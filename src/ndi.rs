//! NDI output. libndi.so is loaded at runtime (NDI runtime / SDK is not bundled):
//! search order: $NDI_RUNTIME_DIR_V6, then the system library path.

use std::ffi::{c_char, c_void, CString};
use std::sync::mpsc::{sync_channel, SyncSender, TrySendError};
use std::sync::Arc;
use std::thread::JoinHandle;

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

const fn fourcc(a: u8, b: u8, c: u8, d: u8) -> u32 {
    (a as u32) | ((b as u32) << 8) | ((c as u32) << 16) | ((d as u32) << 24)
}
const FOURCC_RGBA: u32 = fourcc(b'R', b'G', b'B', b'A');
const FORMAT_PROGRESSIVE: i32 = 1;
const TIMECODE_SYNTHESIZE: i64 = i64::MAX;

type FnInit = unsafe extern "C" fn() -> bool;
type FnSendCreate = unsafe extern "C" fn(*const SendCreate) -> *mut c_void;
type FnSendDestroy = unsafe extern "C" fn(*mut c_void);
type FnSendVideo = unsafe extern "C" fn(*mut c_void, *const VideoFrameV2);

struct Lib {
    _lib: libloading::Library,
    send_create: FnSendCreate,
    send_destroy: FnSendDestroy,
    send_video: FnSendVideo,
}

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

fn load_lib(lib_dir: &str) -> Result<(Lib, String), String> {
    for c in lib_candidates(lib_dir) {
        if c.starts_with('/') && !std::path::Path::new(&c).exists() {
            continue;
        }
        match unsafe { libloading::Library::new(&c) } {
            Ok(lib) => unsafe {
                let init: FnInit = *lib.get::<FnInit>(b"NDIlib_initialize\0").map_err(|e| e.to_string())?;
                let send_create = *lib.get::<FnSendCreate>(b"NDIlib_send_create\0").map_err(|e| e.to_string())?;
                let send_destroy = *lib.get::<FnSendDestroy>(b"NDIlib_send_destroy\0").map_err(|e| e.to_string())?;
                let send_video =
                    *lib.get::<FnSendVideo>(b"NDIlib_send_send_video_v2\0").map_err(|e| e.to_string())?;
                if !init() {
                    return Err("NDIlib_initialize failed (unsupported CPU?)".into());
                }
                return Ok((Lib { _lib: lib, send_create, send_destroy, send_video }, c));
            },
            Err(e) => {
                if c.starts_with('/') {
                    return Err(format!("{c}: {e}"));
                }
            }
        }
    }
    Err("libndi.so.6 not found: set \"NDI library folder\" in the output options (folder that contains libndi.so.6) or install the NDI runtime (see install.md)".into())
}

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
    pub pool: Arc<std::sync::Mutex<Vec<Vec<u8>>>>,
}

impl NdiSender {
    pub fn new(name: &str, fps: i32, lib_dir: &str) -> Result<NdiSender, String> {
        let (lib, lib_path) = load_lib(lib_dir)?;
        let cname = CString::new(name).map_err(|e| e.to_string())?;
        let (tx, rx) = sync_channel::<FrameMsg>(1);
        let pool: Arc<std::sync::Mutex<Vec<Vec<u8>>>> = Default::default();
        let pool2 = pool.clone();
        let (ready_tx, ready_rx) = sync_channel::<Result<(), String>>(1);
        let thread = std::thread::Builder::new()
            .name("ndi-send".into())
            .stack_size(16 << 20)
            .spawn(move || {
                // capture the whole Lib (not only its fn pointers): otherwise the
                // Library handle is dropped in new() and libndi gets unloaded
                let lib = lib;
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
