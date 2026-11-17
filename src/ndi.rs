//! NDI output, discovery and input. libndi.so is loaded at runtime (NDI runtime / SDK is not
//! bundled) and only once per process; sender, finder and receivers share it.

use std::ffi::{c_char, c_void, CStr, CString};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

// ------------------------------------------------------------------- C types

#[repr(C)]
struct SendCreate {
    p_ndi_name: *const c_char,
    p_groups: *const c_char,
    clock_video: bool,
    clock_audio: bool,
}

#[repr(C)]
struct VideoFrameV2 {
    xres: i32,
    yres: i32,
    fourcc: u32,
    frame_rate_n: i32,
    frame_rate_d: i32,
    picture_aspect_ratio: f32,
    frame_format_type: i32,
    timecode: i64,
    p_data: *mut u8,
    line_stride_in_bytes: i32,
    p_metadata: *const c_char,
    timestamp: i64,
}

impl VideoFrameV2 {
    fn empty() -> Self {
        VideoFrameV2 {
            xres: 0,
            yres: 0,
            fourcc: 0,
            frame_rate_n: 0,
            frame_rate_d: 0,
            picture_aspect_ratio: 0.0,
            frame_format_type: 0,
            timecode: 0,
            p_data: std::ptr::null_mut(),
            line_stride_in_bytes: 0,
            p_metadata: std::ptr::null(),
            timestamp: 0,
        }
    }
}

#[repr(C)]
struct FindCreate {
    show_local_sources: bool,
    p_groups: *const c_char,
    p_extra_ips: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Source {
    p_ndi_name: *const c_char,
    p_url_address: *const c_char,
}

#[repr(C)]
struct RecvCreateV3 {
    source_to_connect_to: Source,
    color_format: i32,
    bandwidth: i32,
    allow_video_fields: bool,
    p_ndi_recv_name: *const c_char,
}

const fn fourcc(a: u8, b: u8, c: u8, d: u8) -> u32 {
    (a as u32) | ((b as u32) << 8) | ((c as u32) << 16) | ((d as u32) << 24)
}
const FOURCC_RGBA: u32 = fourcc(b'R', b'G', b'B', b'A');
const FOURCC_RGBX: u32 = fourcc(b'R', b'G', b'B', b'X');
const FORMAT_PROGRESSIVE: i32 = 1;
const TIMECODE_SYNTHESIZE: i64 = i64::MAX;
const RECV_COLOR_RGBX_RGBA: i32 = 2;
const RECV_BANDWIDTH_HIGHEST: i32 = 100;
const FRAME_TYPE_VIDEO: i32 = 1;

type FnInit = unsafe extern "C" fn() -> bool;
type FnSendCreate = unsafe extern "C" fn(*const SendCreate) -> *mut c_void;
type FnSendDestroy = unsafe extern "C" fn(*mut c_void);
type FnSendVideo = unsafe extern "C" fn(*mut c_void, *const VideoFrameV2);
type FnFindCreate = unsafe extern "C" fn(*const FindCreate) -> *mut c_void;
type FnFindDestroy = unsafe extern "C" fn(*mut c_void);
type FnFindWait = unsafe extern "C" fn(*mut c_void, u32) -> bool;
type FnFindSources = unsafe extern "C" fn(*mut c_void, *mut u32) -> *const Source;
type FnRecvCreate = unsafe extern "C" fn(*const RecvCreateV3) -> *mut c_void;
type FnRecvDestroy = unsafe extern "C" fn(*mut c_void);
type FnRecvCapture = unsafe extern "C" fn(*mut c_void, *mut VideoFrameV2, *mut c_void, *mut c_void, u32) -> i32;
type FnRecvFreeVideo = unsafe extern "C" fn(*mut c_void, *const VideoFrameV2);

pub struct Lib {
    _lib: libloading::Library,
    pub path: String,
    send_create: FnSendCreate,
    send_destroy: FnSendDestroy,
    send_video: FnSendVideo,
    find_create: FnFindCreate,
    find_destroy: FnFindDestroy,
    find_wait: FnFindWait,
    find_sources: FnFindSources,
    recv_create: FnRecvCreate,
    recv_destroy: FnRecvDestroy,
    recv_capture: FnRecvCapture,
    recv_free_video: FnRecvFreeVideo,
}

// ------------------------------------------------------------------- loading

/// Folders searched for libndi.so.6 (in this order): the folder set in the ui, $NDI_RUNTIME_DIR_V6,
/// common install folders, then the system library path.
fn lib_candidates(lib_dir: &str) -> Vec<String> {
    let mut dirs: Vec<String> = vec![];
    if !lib_dir.trim().is_empty() {
        dirs.push(lib_dir.trim().trim_end_matches('/').to_string());
    }
    if let Ok(d) = std::env::var("NDI_RUNTIME_DIR_V6") {
        dirs.push(d.trim_end_matches('/').to_string());
    }
    for d in ["/usr/local/lib", "/usr/lib", "/usr/lib/x86_64-linux-gnu", "/opt/ndi/lib"] {
        dirs.push(d.to_string());
    }
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/lib").display().to_string());
    }
    let mut c: Vec<String> = dirs.iter().flat_map(|d| [format!("{d}/libndi.so.6"), format!("{d}/libndi.so")]).collect();
    c.push("libndi.so.6".into());
    c.push("libndi.so".into());
    c
}

unsafe fn sym<T: Copy>(lib: &libloading::Library, name: &[u8]) -> Result<T, String> {
    unsafe { lib.get::<T>(name).map(|s| *s).map_err(|e| e.to_string()) }
}

fn load(path: &str) -> Result<Lib, String> {
    unsafe {
        let lib = libloading::Library::new(path).map_err(|e| format!("{path}: {e}"))?;
        let init: FnInit = sym(&lib, b"NDIlib_initialize\0")?;
        let l = Lib {
            path: path.to_string(),
            send_create: sym(&lib, b"NDIlib_send_create\0")?,
            send_destroy: sym(&lib, b"NDIlib_send_destroy\0")?,
            send_video: sym(&lib, b"NDIlib_send_send_video_v2\0")?,
            find_create: sym(&lib, b"NDIlib_find_create_v2\0")?,
            find_destroy: sym(&lib, b"NDIlib_find_destroy\0")?,
            find_wait: sym(&lib, b"NDIlib_find_wait_for_sources\0")?,
            find_sources: sym(&lib, b"NDIlib_find_get_current_sources\0")?,
            recv_create: sym(&lib, b"NDIlib_recv_create_v3\0")?,
            recv_destroy: sym(&lib, b"NDIlib_recv_destroy\0")?,
            recv_capture: sym(&lib, b"NDIlib_recv_capture_v2\0")?,
            recv_free_video: sym(&lib, b"NDIlib_recv_free_video_v2\0")?,
            _lib: lib,
        };
        if !init() {
            return Err("NDIlib_initialize failed (unsupported CPU?)".into());
        }
        Ok(l)
    }
}

static LIB: Mutex<Option<Arc<Lib>>> = Mutex::new(None);

/// The process-wide libndi. Loaded on first success and then kept (it is never unloaded).
pub fn lib(lib_dir: &str) -> Result<Arc<Lib>, String> {
    let mut g = LIB.lock().map_err(|_| "ndi lock poisoned".to_string())?;
    if let Some(l) = g.as_ref() {
        return Ok(l.clone());
    }
    for c in lib_candidates(lib_dir) {
        if c.starts_with('/') && !std::path::Path::new(&c).exists() {
            continue;
        }
        match load(&c) {
            Ok(l) => {
                let l = Arc::new(l);
                *g = Some(l.clone());
                return Ok(l);
            }
            Err(e) if c.starts_with('/') => return Err(e),
            Err(_) => {}
        }
    }
    Err("libndi.so.6 not found: set \"NDI library folder\" in the output options (folder that contains libndi.so.6) or install the NDI runtime (see install.md)".into())
}

/// Already loaded (without trying to load it)?
pub fn loaded() -> Option<Arc<Lib>> {
    LIB.lock().ok().and_then(|g| g.clone())
}

// -------------------------------------------------------------------- sender

pub struct FrameMsg {
    pub w: i32,
    pub h: i32,
    pub data: Vec<u8>,
}

/// NDI sender running on its own thread. Frames are dropped if the sender is busy.
pub struct NdiSender {
    /// the libndi that was loaded
    pub lib_path: String,
    tx: Option<SyncSender<FrameMsg>>,
    thread: Option<JoinHandle<()>>,
    /// recycled buffers from the sender thread
    pub pool: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl NdiSender {
    pub fn new(name: &str, fps: i32, lib_dir: &str) -> Result<NdiSender, String> {
        let lib = lib(lib_dir)?;
        let lib_path = lib.path.clone();
        let cname = CString::new(name).map_err(|e| e.to_string())?;
        let (tx, rx) = sync_channel::<FrameMsg>(1);
        let pool: Arc<Mutex<Vec<Vec<u8>>>> = Default::default();
        let pool2 = pool.clone();
        let (ready_tx, ready_rx) = sync_channel::<Result<(), String>>(1);
        let thread = std::thread::Builder::new()
            .name("ndi-send".into())
            .spawn(move || {
                let lib = lib; // keep the whole Arc<Lib> in this thread
                let create = SendCreate {
                    p_ndi_name: cname.as_ptr(),
                    p_groups: std::ptr::null(),
                    clock_video: false,
                    clock_audio: false,
                };
                let inst = unsafe { (lib.send_create)(&create) };
                if inst.is_null() {
                    let _ = ready_tx.send(Err("NDIlib_send_create failed".into()));
                    return;
                }
                let _ = ready_tx.send(Ok(()));
                while let Ok(mut f) = rx.recv() {
                    let frame = VideoFrameV2 {
                        xres: f.w,
                        yres: f.h,
                        fourcc: FOURCC_RGBA,
                        frame_rate_n: fps * 1000,
                        frame_rate_d: 1000,
                        picture_aspect_ratio: f.w as f32 / f.h as f32,
                        frame_format_type: FORMAT_PROGRESSIVE,
                        timecode: TIMECODE_SYNTHESIZE,
                        p_data: f.data.as_mut_ptr(),
                        line_stride_in_bytes: f.w * 4,
                        p_metadata: std::ptr::null(),
                        timestamp: 0,
                    };
                    unsafe { (lib.send_video)(inst, &frame) };
                    if let Ok(mut p) = pool2.lock() {
                        if p.len() < 3 {
                            p.push(f.data);
                        }
                    }
                }
                unsafe { (lib.send_destroy)(inst) };
            })
            .map_err(|e| e.to_string())?;
        ready_rx.recv().map_err(|e| e.to_string())??;
        Ok(NdiSender { lib_path, tx: Some(tx), thread: Some(thread), pool })
    }

    pub fn buffer(&self, size: usize) -> Vec<u8> {
        let mut b = self.pool.lock().ok().and_then(|mut p| p.pop()).unwrap_or_default();
        b.resize(size, 0);
        b
    }

    pub fn send(&self, f: FrameMsg) {
        if let Some(tx) = &self.tx {
            match tx.try_send(f) {
                Ok(()) | Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {}
            }
        }
    }
}

impl Drop for NdiSender {
    fn drop(&mut self) {
        self.tx.take();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

// -------------------------------------------------------------------- finder

/// Discovers NDI sources on the network (own thread, list refreshed continuously).
pub struct NdiFinder {
    sources: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl NdiFinder {
    pub fn new(lib: Arc<Lib>) -> NdiFinder {
        let sources: Arc<Mutex<Vec<String>>> = Default::default();
        let stop = Arc::new(AtomicBool::new(false));
        let (s2, stop2) = (sources.clone(), stop.clone());
        let thread = std::thread::Builder::new()
            .name("ndi-find".into())
            .spawn(move || {
                let lib = lib;
                let create = FindCreate { show_local_sources: true, p_groups: std::ptr::null(), p_extra_ips: std::ptr::null() };
                let inst = unsafe { (lib.find_create)(&create) };
                if inst.is_null() {
                    return;
                }
                while !stop2.load(Ordering::Relaxed) {
                    unsafe { (lib.find_wait)(inst, 500) };
                    let mut n = 0u32;
                    let ptr = unsafe { (lib.find_sources)(inst, &mut n) };
                    let mut names = vec![];
                    if !ptr.is_null() {
                        for i in 0..n as usize {
                            let s = unsafe { *ptr.add(i) };
                            if !s.p_ndi_name.is_null() {
                                names.push(unsafe { CStr::from_ptr(s.p_ndi_name) }.to_string_lossy().to_string());
                            }
                        }
                    }
                    names.sort();
                    if let Ok(mut g) = s2.lock() {
                        *g = names;
                    }
                }
                unsafe { (lib.find_destroy)(inst) };
            })
            .ok();
        NdiFinder { sources, stop, thread }
    }

    pub fn sources(&self) -> Vec<String> {
        self.sources.lock().map(|g| g.clone()).unwrap_or_default()
    }
}

impl Drop for NdiFinder {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

// ------------------------------------------------------------------ receiver

/// One received frame: RGBA rows top-down.
pub struct Frame {
    pub w: i32,
    pub h: i32,
    pub stride: i32,
    pub data: Vec<u8>,
}

#[derive(Default)]
struct RecvShared {
    frame: Mutex<Option<Frame>>,
    seq: AtomicU64,
    fps: Mutex<f32>,
}

/// Receives one NDI source on its own thread (RGBA frames, latest frame wins).
pub struct NdiReceiver {
    shared: Arc<RecvShared>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    last_seq: u64,
    last_frame: Instant,
}

impl NdiReceiver {
    pub fn new(lib: Arc<Lib>, source: &str) -> Result<NdiReceiver, String> {
        let cname = CString::new(source).map_err(|e| e.to_string())?;
        let recv_name = CString::new("videoping").unwrap();
        let shared = Arc::new(RecvShared::default());
        let stop = Arc::new(AtomicBool::new(false));
        let (sh, stop2) = (shared.clone(), stop.clone());
        let thread = std::thread::Builder::new()
            .name("ndi-recv".into())
            .spawn(move || {
                let lib = lib;
                let create = RecvCreateV3 {
                    source_to_connect_to: Source { p_ndi_name: cname.as_ptr(), p_url_address: std::ptr::null() },
                    color_format: RECV_COLOR_RGBX_RGBA,
                    bandwidth: RECV_BANDWIDTH_HIGHEST,
                    allow_video_fields: false,
                    p_ndi_recv_name: recv_name.as_ptr(),
                };
                let inst = unsafe { (lib.recv_create)(&create) };
                if inst.is_null() {
                    return;
                }
                while !stop2.load(Ordering::Relaxed) {
                    let mut vf = VideoFrameV2::empty();
                    let ty = unsafe { (lib.recv_capture)(inst, &mut vf, std::ptr::null_mut(), std::ptr::null_mut(), 200) };
                    if ty != FRAME_TYPE_VIDEO {
                        continue;
                    }
                    if !vf.p_data.is_null() && (vf.fourcc == FOURCC_RGBA || vf.fourcc == FOURCC_RGBX) && vf.xres > 0 && vf.yres > 0 {
                        let stride = vf.line_stride_in_bytes.max(vf.xres * 4);
                        let need = (stride * vf.yres) as usize;
                        let src = unsafe { std::slice::from_raw_parts(vf.p_data, need) };
                        if let Ok(mut g) = sh.frame.lock() {
                            match g.as_mut() {
                                Some(f) if f.data.len() == need => {
                                    f.data.copy_from_slice(src);
                                    f.w = vf.xres;
                                    f.h = vf.yres;
                                    f.stride = stride;
                                }
                                _ => *g = Some(Frame { w: vf.xres, h: vf.yres, stride, data: src.to_vec() }),
                            }
                        }
                        if vf.frame_rate_d > 0 {
                            if let Ok(mut f) = sh.fps.lock() {
                                *f = vf.frame_rate_n as f32 / vf.frame_rate_d as f32;
                            }
                        }
                        sh.seq.fetch_add(1, Ordering::Release);
                    }
                    unsafe { (lib.recv_free_video)(inst, &vf) };
                }
                unsafe { (lib.recv_destroy)(inst) };
            })
            .map_err(|e| e.to_string())?;
        Ok(NdiReceiver { shared, stop, thread: Some(thread), last_seq: 0, last_frame: Instant::now() })
    }

    /// New frame since the last call?
    pub fn take_frame<R>(&mut self, f: impl FnOnce(&Frame) -> R) -> Option<R> {
        let seq = self.shared.seq.load(Ordering::Acquire);
        if seq == self.last_seq {
            return None;
        }
        self.last_seq = seq;
        self.last_frame = Instant::now();
        let g = self.shared.frame.lock().ok()?;
        g.as_ref().map(f)
    }

    /// (width, height, fps) of the last frame; None = no signal for 2 s
    pub fn status(&self) -> Option<(i32, i32, f32)> {
        if self.last_seq == 0 || self.last_frame.elapsed() > Duration::from_secs(2) {
            return None;
        }
        let g = self.shared.frame.lock().ok()?;
        let fps = self.shared.fps.lock().map(|f| *f).unwrap_or(0.0);
        g.as_ref().map(|f| (f.w, f.h, fps))
    }
}

impl Drop for NdiReceiver {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
