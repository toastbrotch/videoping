//! mp4 playback with GStreamer (playbin -> appsink RGBA).
//!
//! The transport (forward/backward/jumpcut/pause, loop/once/bounce, free/bpm speed,
//! limiters) is driven from the main thread with rate seeks.

use crate::state::{ClipTransport, LoopMode, PlayMode, Rng};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub fn init() -> Result<(), String> {
    gst::init().map_err(|e| e.to_string())
}

pub struct Frame {
    pub w: i32,
    pub h: i32,
    pub stride: i32,
    pub data: Vec<u8>,
}

#[derive(Default)]
struct Shared {
    frame: Mutex<Option<Frame>>,
    seq: AtomicU64,
}

pub struct Player {
    pipeline: gst::Element,
    shared: Arc<Shared>,
    pub last_seq: u64,
    pub duration: Option<f64>,
    pub position: f64,
    applied_rate: f64,
    playing: bool,
    last_seek: Instant,
    bounce_dir: f64,
    held: bool,
    last_mode: Option<PlayMode>,
    last_loop: Option<LoopMode>,
    pub error: Option<String>,
    eos: bool,
}

impl Player {
    pub fn new(path: &Path) -> Result<Player, String> {
        let abs = std::fs::canonicalize(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let uri = gst::glib::filename_to_uri(&abs, None).map_err(|e| e.to_string())?;
        let caps = gst_video::VideoCapsBuilder::new().format(gst_video::VideoFormat::Rgba).build();
        let sink = gst_app::AppSink::builder().caps(&caps).max_buffers(1).drop(true).sync(true).build();
        let shared = Arc::new(Shared::default());
        let sh = shared.clone();
        let sh2 = shared.clone();
        sink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |s| {
                    let sample = s.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                    store_sample(&sh, &sample);
                    Ok(gst::FlowSuccess::Ok)
                })
                // preroll = first frame while paused (thumbnail, pause mode, scrubbing)
                .new_preroll(move |s| {
                    if let Ok(sample) = s.pull_preroll() {
                        store_sample(&sh2, &sample);
                    }
                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );
        let audio = gst::ElementFactory::make("fakesink").build().map_err(|e| e.to_string())?;
        let pipeline = gst::ElementFactory::make("playbin")
            .property("uri", uri.as_str())
            .property("video-sink", &sink)
            .property("audio-sink", &audio)
            .build()
            .map_err(|e| format!("playbin: {e}"))?;
        pipeline.set_state(gst::State::Paused).map_err(|e| format!("{e:?}"))?;
        Ok(Player {
            pipeline,
            shared,
            last_seq: 0,
            duration: None,
            position: 0.0,
            applied_rate: 0.0,
            playing: false,
            last_seek: Instant::now(),
            bounce_dir: 1.0,
            held: false,
            last_mode: None,
            last_loop: None,
            error: None,
            eos: false,
        })
    }

    /// New frame since the last call?
    pub fn take_frame<R>(&mut self, f: impl FnOnce(&Frame) -> R) -> Option<R> {
        let seq = self.shared.seq.load(Ordering::Acquire);
        if seq == self.last_seq {
            return None;
        }
        self.last_seq = seq;
        let guard = self.shared.frame.lock().ok()?;
        guard.as_ref().map(f)
    }

    fn poll_bus(&mut self) {
        let Some(bus) = self.pipeline.bus() else { return };
        while let Some(msg) = bus.pop() {
            match msg.view() {
                gst::MessageView::Eos(_) => self.eos = true,
                gst::MessageView::Error(e) => {
                    self.error = Some(format!("{}", e.error()));
                }
                _ => {}
            }
        }
    }

    fn seek(&mut self, pos: f64, rate: f64) {
        let pos = pos.max(0.0);
        let p = gst::ClockTime::from_nseconds((pos * 1e9) as u64);
        let flags = gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE;
        let res = if rate >= 0.0 {
            self.pipeline.seek(rate.max(0.01), flags, gst::SeekType::Set, p, gst::SeekType::None, gst::ClockTime::NONE)
        } else {
            self.pipeline.seek(rate.min(-0.01), flags, gst::SeekType::Set, gst::ClockTime::ZERO, gst::SeekType::Set, p)
        };
        if res.is_ok() {
            self.applied_rate = rate;
            self.position = pos;
        }
        self.eos = false;
        self.last_seek = Instant::now();
    }

    fn set_playing(&mut self, on: bool) {
        if on != self.playing {
            let _ = self.pipeline.set_state(if on { gst::State::Playing } else { gst::State::Paused });
            self.playing = on;
        }
    }

    /// Jump to a normalized position (timeline click).
    pub fn scrub(&mut self, norm: f32) {
        if let Some(d) = self.duration {
            let r = if self.applied_rate == 0.0 { 1.0 } else { self.applied_rate };
            self.held = false;
            self.seek(norm as f64 * d, r);
        }
    }

    pub fn update(&mut self, t: &ClipTransport, bpm: f32, jump: bool, rng: &mut Rng) {
        self.poll_bus();
        if self.duration.is_none() {
            if let Some(d) = self.pipeline.query_duration::<gst::ClockTime>() {
                self.duration = Some(d.nseconds() as f64 / 1e9);
            } else {
                return; // still prerolling
            }
        }
        let dur = self.duration.unwrap_or(1.0).max(0.04);
        if let Some(p) = self.pipeline.query_position::<gst::ClockTime>() {
            self.position = p.nseconds() as f64 / 1e9;
        }
        let in_s = (t.in_pt.min(t.out_pt) as f64 * dur).clamp(0.0, dur);
        let out_s = (t.out_pt.max(t.in_pt) as f64 * dur).clamp(in_s + 0.04, dur);
        let len = out_s - in_s;

        let base_rate = if t.speed.bpm_mode {
            len / (t.speed.beats.max(0.0625) as f64 * 60.0 / bpm.max(1.0) as f64)
        } else {
            t.speed.free as f64
        }
        .clamp(0.01, 16.0);

        if self.last_mode != Some(t.mode) {
            self.held = false;
            self.bounce_dir = 1.0;
            self.last_mode = Some(t.mode);
        }
        // stopped at the end in "once" and then switched to loop / bounce: play again
        // (loop restarts from the start, bounce turns around at the end)
        if self.last_loop != Some(t.loop_mode) {
            if t.loop_mode != LoopMode::Once {
                self.held = false;
            }
            self.last_loop = Some(t.loop_mode);
        }

        if t.mode == PlayMode::Pause || self.held {
            self.set_playing(false);
            return;
        }

        let base_dir = if t.mode == PlayMode::Backward { -1.0 } else { 1.0 };
        let dir = if t.loop_mode == LoopMode::Bounce { base_dir * self.bounce_dir } else { base_dir };
        let mut rate = dir * base_rate;
        let eps = 0.03 * base_rate.max(1.0);

        if t.mode == PlayMode::Jumpcut && jump {
            let target = in_s + rng.next_f32() as f64 * len;
            self.seek(target, rate);
            self.set_playing(true);
            return;
        }

        // outside the limiters (limiter moved / first start) -> into the range
        if self.position < in_s - 0.05 || self.position > out_s + 0.05 {
            let start = if rate >= 0.0 { in_s } else { out_s };
            self.seek(start, rate);
            self.set_playing(true);
            return;
        }

        let at_end = rate > 0.0 && (self.position >= out_s - eps || self.eos);
        let at_start = rate < 0.0 && (self.position <= in_s + eps || self.eos);
        if at_end || at_start {
            match t.loop_mode {
                LoopMode::Loop => {
                    let start = if rate > 0.0 { in_s } else { out_s };
                    self.seek(start, rate);
                }
                LoopMode::Once => {
                    self.held = true;
                    self.set_playing(false);
                    return;
                }
                LoopMode::Bounce => {
                    self.bounce_dir = -self.bounce_dir;
                    rate = -rate;
                    let p = self.position.clamp(in_s, out_s);
                    self.seek(p, rate);
                }
            }
        } else if self.applied_rate == 0.0
            || self.applied_rate.signum() != rate.signum()
            || ((self.applied_rate - rate).abs() / rate.abs() > 0.02 && self.last_seek.elapsed().as_millis() > 120)
        {
            self.seek(self.position.clamp(in_s, out_s), rate);
        }
        self.set_playing(true);
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}

fn store_sample(sh: &Shared, sample: &gst::Sample) {
    let Some(buffer) = sample.buffer() else { return };
    let Some(caps) = sample.caps() else { return };
    let Ok(info) = gst_video::VideoInfo::from_caps(caps) else { return };
    let Ok(map) = buffer.map_readable() else { return };
    let w = info.width() as i32;
    let h = info.height() as i32;
    let stride = info.stride()[0];
    let need = (stride * h) as usize;
    if map.len() < need {
        return;
    }
    if let Ok(mut g) = sh.frame.lock() {
        match g.as_mut() {
            Some(f) if f.data.len() == need => {
                f.data.copy_from_slice(&map[..need]);
                f.w = w;
                f.h = h;
                f.stride = stride;
            }
            _ => *g = Some(Frame { w, h, stride, data: map[..need].to_vec() }),
        }
    }
    sh.seq.fetch_add(1, Ordering::Release);
}
