//! Resource meters (cheap by design):
//! - GPU time per source / effect: `TIME_ELAPSED` queries, never nested, read back a few frames
//!   later without waiting for the GPU (no stall); only every SAMPLE_EVERY-th video frame,
//!   because each query costs some CPU in the driver (measured: ~1.2 ms per frame on Intel/Mesa
//!   when every frame is measured)
//! - VRAM: bookkeeping of what videoping allocates itself (no GPU call)
//! - process CPU / RAM: `/proc/self` once per second
//!
//! Levels 1..3 for the grey bar indicators: relative to the frame budget (1000 / fps ms).

use glow::HasContext;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

type Gl = glow::Context;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Key {
    /// source of an instance: clip upload, NDI upload, generator shader
    Src(u64),
    /// one effect (shader passes + mix)
    Fx(u64),
    /// mixer of the 3 layers
    Mixer,
    /// freeze, master brightness, NDI readback
    Master,
    /// recording: NV12 conversion + readback
    Record,
}

/// Frames in flight before measuring pauses (the GPU is far behind: do not add more work).
const MAX_PENDING: usize = 6;
/// Measure every n-th video frame only (60 fps: 7.5 samples per second and item).
const SAMPLE_EVERY: u64 = 8;
/// A key that was not measured for this long is not active any more.
const STALE_SECS: f32 = 1.0;

pub struct GpuTimer {
    pub enabled: bool,
    pool: Vec<glow::Query>,
    open: Option<(Key, glow::Query)>,
    frame: Vec<(Key, glow::Query)>,
    pending: VecDeque<Vec<(Key, glow::Query)>>,
    /// smoothed ms per key, time of the last sample
    ms: HashMap<Key, (f32, Instant)>,
    /// smoothed GPU ms of all measured work per frame
    pub total_ms: f32,
    frame_no: u64,
}

impl GpuTimer {
    pub fn new() -> GpuTimer {
        GpuTimer { enabled: true, pool: vec![], open: None, frame: vec![], pending: VecDeque::new(), ms: HashMap::new(), total_ms: 0.0, frame_no: 0 }
    }

    /// Start measuring `key` (ends a span that is still open: spans never nest).
    pub fn begin(&mut self, gl: &Gl, key: Key) {
        self.end(gl);
        if !self.enabled || !self.frame_no.is_multiple_of(SAMPLE_EVERY) || self.pending.len() >= MAX_PENDING {
            return;
        }
        let q = match self.pool.pop() {
            Some(q) => q,
            None => match unsafe { gl.create_query() } {
                Ok(q) => q,
                Err(_) => return,
            },
        };
        unsafe { gl.begin_query(glow::TIME_ELAPSED, q) };
        self.open = Some((key, q));
    }

    pub fn end(&mut self, gl: &Gl) {
        if let Some((k, q)) = self.open.take() {
            unsafe { gl.end_query(glow::TIME_ELAPSED) };
            self.frame.push((k, q));
        }
    }

    /// Once per video frame: queue this frame's queries, collect finished ones (non-blocking).
    pub fn end_frame(&mut self, gl: &Gl) {
        self.end(gl);
        self.frame_no += 1;
        if !self.frame.is_empty() {
            self.pending.push_back(std::mem::take(&mut self.frame));
        }
        let now = Instant::now();
        while let Some(front) = self.pending.front() {
            // results arrive in order: the last query of a frame done = the whole frame done
            let done = front.last().is_none_or(|(_, q)| unsafe { gl.get_query_parameter_u32(*q, glow::QUERY_RESULT_AVAILABLE) != 0 });
            if !done {
                break;
            }
            let spans = self.pending.pop_front().unwrap();
            let mut sum: HashMap<Key, f32> = HashMap::new();
            let mut total = 0.0;
            for (k, q) in spans {
                let ns = unsafe { gl.get_query_parameter_u64(q, glow::QUERY_RESULT) };
                let ms = ns as f32 / 1e6;
                *sum.entry(k).or_default() += ms;
                total += ms;
                self.pool.push(q);
            }
            for (k, v) in sum {
                let e = self.ms.entry(k).or_insert((v, now));
                // smoothed over a few samples: steady numbers, still reacts within about a second
                e.0 = e.0 * 0.7 + v * 0.3;
                e.1 = now;
            }
            self.total_ms = self.total_ms * 0.7 + total * 0.3;
        }
        self.ms.retain(|_, (_, t)| now.duration_since(*t).as_secs_f32() < STALE_SECS * 4.0);
        if !self.enabled {
            self.total_ms = 0.0;
        }
    }

    /// Smoothed GPU ms of `key`, None when it was not rendered recently.
    pub fn ms(&self, key: Key) -> Option<f32> {
        self.ms.get(&key).filter(|(_, t)| t.elapsed().as_secs_f32() < STALE_SECS).map(|(v, _)| *v)
    }
}

/// Clip decoding info (CPU side of a clip).
#[derive(Clone, Debug, Default)]
pub struct DecodeInfo {
    pub decoder: String,
    pub hardware: bool,
    pub w: i32,
    pub h: i32,
    pub fps: f64,
    /// current playback rate (0 = paused)
    pub rate: f64,
    /// frames per second actually delivered by the decoder (last second)
    pub delivered: f64,
}

impl DecodeInfo {
    /// Frames per second the clip should deliver right now.
    pub fn expected(&self) -> f64 {
        self.fps * self.rate.abs()
    }
    /// Decoder does not keep up (with a little slack for measuring jitter).
    pub fn behind(&self) -> bool {
        let e = self.expected();
        e > 1.0 && self.delivered < e * 0.85
    }
    /// 1..3: hardware decoding is light; software decoding by pixel rate (1080p30 ≈ 62 Mpx/s).
    pub fn level(&self) -> u8 {
        if self.behind() {
            return 3;
        }
        if self.hardware {
            return 1;
        }
        let mpx = self.w as f64 * self.h as f64 * self.expected() / 1e6;
        if mpx < 95.0 {
            1
        } else if mpx < 380.0 {
            2
        } else {
            3
        }
    }
}

/// 1..3 from the share of the frame budget: < 15 % light, < 40 % medium, else heavy.
pub fn gpu_level(ms: f32, budget_ms: f32) -> u8 {
    let share = ms / budget_ms.max(0.1);
    if share < 0.15 {
        1
    } else if share < 0.40 {
        2
    } else {
        3
    }
}

/// 1..3 from the own VRAM of an item: < 64 MB light, < 256 MB medium, else heavy.
pub fn vram_level(bytes: u64) -> u8 {
    let mb = bytes / (1024 * 1024);
    if mb < 64 {
        1
    } else if mb < 256 {
        2
    } else {
        3
    }
}

pub fn mb(bytes: u64) -> String {
    format!("{} MB", (bytes + 512 * 1024) / (1024 * 1024))
}

/// Process CPU (% of one core) and RAM (resident), sampled once per second.
pub struct ProcStats {
    last: Option<(Instant, u64)>,
    pub cpu_percent: f32,
    pub rss_bytes: u64,
    pub cores: usize,
    /// NVIDIA only (GL_NVX_gpu_memory_info): (used, total) of the whole GPU
    pub gpu_mem: Option<(u64, u64)>,
}

impl ProcStats {
    pub fn new() -> ProcStats {
        let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
        ProcStats { last: None, cpu_percent: 0.0, rss_bytes: 0, cores, gpu_mem: None }
    }

    pub fn sample(&mut self, gl: &Gl) {
        let now = Instant::now();
        if self.last.is_some_and(|(t, _)| now.duration_since(t).as_secs_f32() < 1.0) {
            return;
        }
        // utime + stime (fields 14, 15; field 2 can contain spaces, so count after the ')')
        let ticks = std::fs::read_to_string("/proc/self/stat").ok().and_then(|s| {
            let rest = &s[s.rfind(')')? + 2..];
            let f: Vec<&str> = rest.split_whitespace().collect();
            Some(f.get(11)?.parse::<u64>().ok()? + f.get(12)?.parse::<u64>().ok()?)
        });
        if let Some(ticks) = ticks {
            if let Some((t0, k0)) = self.last {
                // clock ticks are 100 / s on Linux
                let secs = now.duration_since(t0).as_secs_f32();
                self.cpu_percent = (ticks.saturating_sub(k0)) as f32 / 100.0 / secs * 100.0;
            }
            self.last = Some((now, ticks));
        }
        if let Ok(s) = std::fs::read_to_string("/proc/self/status") {
            if let Some(kb) = s.lines().find(|l| l.starts_with("VmRSS:")).and_then(|l| l.split_whitespace().nth(1)?.parse::<u64>().ok()) {
                self.rss_bytes = kb * 1024;
            }
        }
        if gl.supported_extensions().contains("GL_NVX_gpu_memory_info") {
            const TOTAL: u32 = 0x9048;
            const AVAILABLE: u32 = 0x9049;
            let (total, avail) = unsafe { (gl.get_parameter_i32(TOTAL), gl.get_parameter_i32(AVAILABLE)) };
            if total > 0 {
                self.gpu_mem = Some((((total - avail).max(0) as u64) * 1024, total as u64 * 1024));
            }
        }
    }
}
