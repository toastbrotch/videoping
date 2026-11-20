//! Recording of the output to an H.264 mp4 (hardware encoder, plays again in videoping).
//!
//! appsrc (NV12, converted on the GPU) -> NVENC / Intel VA H.264 -> h264parse -> mp4mux -> file
//! - short keyframe distance (fps / 4, 8..15 frames): backward play, jumpcut and scrubbing stay smooth
//! - no B-frames (easier backward decoding)
//! - crash-safe: the index is kept in reserved space at the start of the file and updated every
//!   2 s, so a recording interrupted by a crash still plays up to that point
//! - frames are stamped by frame number (round(elapsed * fps)): constant frame rate, compact index,
//!   still in real time when frames were dropped

use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// H.264 encoders in order of preference: (element, description, encoder options).
const ENCODERS: &[(&str, &str)] = &[
    ("nvcudah264enc", "NVIDIA NVENC"),
    ("nvautogpuh264enc", "NVIDIA NVENC"),
    ("vah264lpenc", "Intel / AMD VA-API"),
    ("vaapih264enc", "VA-API (legacy)"),
];

/// Encoder options: bitrate in kbit/s, keyframe distance in frames, no B-frames.
fn encoder_desc(name: &str, kbps: u32, gop: u32) -> String {
    match name {
        "vah264lpenc" => format!("vah264lpenc bitrate={kbps} key-int-max={gop} b-frames=0"),
        "vaapih264enc" => format!("vaapih264enc bitrate={kbps} keyframe-period={gop} max-bframes=0"),
        n => format!("{n} bitrate={kbps} gop-size={gop} b-frames=0"),
    }
}

/// Keyframe distance: a quarter second, 8..15 frames.
pub fn gop(fps: u32) -> u32 {
    (fps / 4).clamp(8, 15)
}

/// 40 Mbit/s at 1080p60, scaled by pixel rate, at least 8 Mbit/s.
pub fn bitrate_kbps(w: i32, h: i32, fps: u32) -> u32 {
    let rate = (w as f64 * h as f64 * fps as f64) / (1920.0 * 1080.0 * 60.0);
    ((40_000.0 * rate) as u32).max(8_000)
}

static ENCODER: OnceLock<Result<(&'static str, &'static str), String>> = OnceLock::new();

/// The first encoder that really works here (tested once with a tiny clip; runs in the background
/// at program start). None = probe still running.
pub fn encoder() -> Option<&'static Result<(&'static str, &'static str), String>> {
    ENCODER.get()
}

pub fn start_probe() {
    std::thread::spawn(|| {
        let r = probe();
        match &r {
            Ok((n, d)) => eprintln!("record: encoder {n} ({d})"),
            Err(e) => eprintln!("record: {e}"),
        }
        let _ = ENCODER.set(r);
    });
}

fn probe() -> Result<(&'static str, &'static str), String> {
    for (name, desc) in ENCODERS {
        if gst::ElementFactory::find(name).is_none() {
            continue;
        }
        let launch = format!(
            "appsrc name=src format=time caps=video/x-raw,format=NV12,width=256,height=144,framerate=30/1 ! {} ! h264parse ! fakesink",
            encoder_desc(name, 1000, 8)
        );
        let Ok(p) = gst::parse::launch(&launch) else { continue };
        let Some(src) = p.downcast_ref::<gst::Bin>().and_then(|b| b.by_name("src")).and_then(|e| e.downcast::<gst_app::AppSrc>().ok()) else {
            continue;
        };
        if p.set_state(gst::State::Playing).is_err() {
            let _ = p.set_state(gst::State::Null);
            continue;
        }
        for i in 0..3u64 {
            let mut b = gst::Buffer::from_mut_slice(vec![16u8; 256 * 144 * 3 / 2]);
            b.get_mut().unwrap().set_pts(gst::ClockTime::from_mseconds(i * 33));
            let _ = src.push_buffer(b);
        }
        let _ = src.end_of_stream();
        let ok = p.bus().and_then(|bus| bus.timed_pop_filtered(gst::ClockTime::from_seconds(5), &[gst::MessageType::Eos, gst::MessageType::Error]))
            .is_some_and(|m| m.type_() == gst::MessageType::Eos);
        let _ = p.set_state(gst::State::Null);
        if ok {
            return Ok((name, desc));
        }
    }
    Err("no hardware H.264 encoder works here (NVENC / VA-API); recording is not available".into())
}

pub struct Recorder {
    pipeline: gst::Element,
    src: gst_app::AppSrc,
    pub path: PathBuf,
    pub encoder: &'static str,
    pub start: Instant,
    pub w: i32,
    pub h: i32,
    fps: u32,
    last_index: Option<u64>,
    pub frames: u64,
    /// frames that did not get into the recording (render too slow, or encoder queue full)
    pub dropped: u64,
    pub error: Option<String>,
    /// smoothed CPU ms per frame on the render thread (copy into the encoder queue)
    pub cpu_ms: f32,
}

impl Recorder {
    pub fn start(path: &Path, w: i32, h: i32, fps: u32) -> Result<Recorder, String> {
        let (enc, _) = match encoder() {
            Some(Ok(e)) => *e,
            Some(Err(e)) => return Err(e.clone()),
            None => return Err("still testing the encoder, try again in a moment".into()),
        };
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
        }
        let launch = format!(
            "appsrc name=src format=time is-live=true max-buffers=8 leaky-type=downstream \
             caps=video/x-raw,format=NV12,width={w},height={h},framerate={fps}/1,colorimetry=bt709 \
             ! {} ! h264parse ! mp4mux name=mux reserved-max-duration={} reserved-moov-update-period={} \
             ! filesink name=sink",
            encoder_desc(enc, bitrate_kbps(w, h, fps), gop(fps)),
            // index space for 1 hour reserved at the start (about 2 MB); longer recordings are
            // finished normally on stop, only the crash safety ends after that hour
            gst::ClockTime::from_seconds(3600).nseconds(),
            gst::ClockTime::from_seconds(2).nseconds(),
        );
        let pipeline = gst::parse::launch(&launch).map_err(|e| format!("record pipeline: {e}"))?;
        let bin = pipeline.downcast_ref::<gst::Bin>().ok_or("record pipeline: not a bin")?;
        let sink = bin.by_name("sink").ok_or("record pipeline: no sink")?;
        sink.set_property("location", path.to_string_lossy().to_string());
        let src = bin.by_name("src").and_then(|e| e.downcast::<gst_app::AppSrc>().ok()).ok_or("record pipeline: no appsrc")?;
        pipeline.set_state(gst::State::Playing).map_err(|e| format!("record: {e:?}"))?;
        Ok(Recorder {
            pipeline,
            src,
            path: path.to_path_buf(),
            encoder: enc,
            start: Instant::now(),
            w,
            h,
            fps,
            last_index: None,
            frames: 0,
            dropped: 0,
            error: None,
            cpu_ms: 0.0,
        })
    }

    /// One NV12 frame (w * h * 3/2 bytes, top row first).
    pub fn push(&mut self, data: &[u8]) {
        if self.error.is_some() {
            return;
        }
        let fps = self.fps.max(1) as f64;
        let index = (self.start.elapsed().as_secs_f64() * fps).round() as u64;
        if let Some(last) = self.last_index {
            if index <= last {
                return; // two frames in the same slot (rendered early): keep the first
            }
            self.dropped += index - last - 1;
        }
        self.last_index = Some(index);
        let t0 = Instant::now();
        let mut b = gst::Buffer::from_mut_slice(data.to_vec());
        {
            let b = b.get_mut().unwrap();
            b.set_pts(gst::ClockTime::from_nseconds((index as f64 * 1e9 / fps) as u64));
            b.set_duration(gst::ClockTime::from_nseconds((1e9 / fps) as u64));
        }
        if self.src.push_buffer(b).is_err() {
            self.dropped += 1;
        } else {
            self.frames += 1;
        }
        let ms = t0.elapsed().as_secs_f32() * 1000.0;
        self.cpu_ms = if self.frames <= 1 { ms } else { self.cpu_ms * 0.9 + ms * 0.1 };
    }

    /// Frames waiting for the encoder (max 8; full = the encoder does not keep up, frames get dropped).
    pub fn queue(&self) -> u64 {
        self.src.property::<u64>("current-level-buffers")
    }

    pub const QUEUE_MAX: u64 = 8;

    /// Errors from the pipeline (disk full, encoder failure).
    pub fn poll(&mut self) {
        let Some(bus) = self.pipeline.bus() else { return };
        while let Some(m) = bus.pop() {
            if let gst::MessageView::Error(e) = m.view() {
                self.error = Some(format!("recording stopped: {}", e.error()));
            }
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Finish the file in the background (the render thread does not wait); join the handle
    /// only when quitting.
    pub fn stop(self) -> std::thread::JoinHandle<()> {
        let _ = self.src.end_of_stream();
        let p = self.pipeline;
        let path = self.path;
        std::thread::spawn(move || {
            let eos = p.bus().and_then(|bus| bus.timed_pop_filtered(gst::ClockTime::from_seconds(15), &[gst::MessageType::Eos, gst::MessageType::Error]));
            if !eos.is_some_and(|m| m.type_() == gst::MessageType::Eos) {
                eprintln!("record: {} was not finished cleanly (still playable up to the last index update)", path.display());
            }
            let _ = p.set_state(gst::State::Null);
        })
    }
}

/// videoping-2026-11-20_22-53-07.mp4 in `dir`.
pub fn new_file_name(dir: &Path) -> PathBuf {
    let stamp = gst::glib::DateTime::now_local()
        .ok()
        .and_then(|d| d.format("%Y-%m-%d_%H-%M-%S").ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)));
    dir.join(format!("videoping-{stamp}.mp4"))
}
