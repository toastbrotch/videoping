//! videoping — personal live vjing tool. See features.md.

mod ai;
mod app;
mod control;
mod gfx;
mod isf;
mod mcp;
mod mapping;
mod meter;
mod midi;
mod ndi;
mod record;
mod render;
mod shader_doc;
mod state;
mod video;
mod videopong;
mod widgets;

pub const NVIDIA_EGL_VENDOR: &str = "/usr/share/glvnd/egl_vendor.d/10_nvidia.json";

/// Hybrid laptops: start on the discrete NVIDIA GPU (PRIME render offload) when asked with
/// `--nvidia` or the saved option "render on NVIDIA GPU". The GL driver is chosen when the
/// process starts, so videoping restarts itself once with the offload variables set.
fn maybe_restart_on_nvidia() {
    use std::os::unix::process::CommandExt;
    if std::env::var_os("__NV_PRIME_RENDER_OFFLOAD").is_some() || std::env::var_os("VIDEOPING_SELFTEST").is_some() {
        return; // already offloaded (or self-test)
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = args.iter().any(|a| a == "--nvidia");
    let saved = dirs::data_dir()
        .map(|d| d.join("videoping").join("autosave.json"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.pointer("/output/prefer_nvidia").and_then(|b| b.as_bool()))
        .unwrap_or(false);
    if !(flag || saved) {
        return;
    }
    if !std::path::Path::new(NVIDIA_EGL_VENDOR).exists() {
        eprintln!("nvidia: no NVIDIA EGL driver found ({NVIDIA_EGL_VENDOR}), staying on the default gpu");
        return;
    }
    let Ok(exe) = std::env::current_exe() else { return };
    let err = std::process::Command::new(exe)
        .args(args.iter().filter(|a| *a != "--nvidia"))
        .env("__NV_PRIME_RENDER_OFFLOAD", "1")
        .env("__GLX_VENDOR_LIBRARY_NAME", "nvidia")
        .env("__EGL_VENDOR_LIBRARY_FILENAMES", NVIDIA_EGL_VENDOR)
        .exec(); // only returns on error
    eprintln!("nvidia: restart failed: {err}");
}

/// Window icon: the v2 (glitch) logo, built into the binary. Wayland desktops take the icon from
/// the installed videoping.desktop / icon theme instead (see install.md).
pub(crate) fn app_icon() -> std::sync::Arc<egui::IconData> {
    let png = include_bytes!("../website/logo/videoping-v2-icon-256.png");
    std::sync::Arc::new(eframe::icon_data::from_png_bytes(png).unwrap_or_default())
}

/// Only one videoping at a time: two would fight over the MIDI controllers (leds), the output
/// window, NDI and the autosave. An exclusive lock on a file, released by the system when the
/// process ends (also after a crash). The self-test is exempt (it can run beside a session).
fn single_instance() -> Option<std::fs::File> {
    if std::env::var_os("VIDEOPING_SELFTEST").is_some() {
        return None;
    }
    let dir = dirs::runtime_dir().or_else(dirs::data_dir).unwrap_or_else(std::env::temp_dir);
    let path = dir.join("videoping.lock");
    let f = std::fs::OpenOptions::new().create(true).truncate(false).write(true).open(&path).ok()?;
    match f.try_lock() {
        Ok(()) => Some(f),
        Err(std::fs::TryLockError::WouldBlock) => {
            let msg = "videoping is already running";
            eprintln!("{msg} (lock {})", path.display());
            // started from the launcher there is no terminal: a desktop notification, if available
            let _ = std::process::Command::new("notify-send").args(["-a", "videoping", "-i", "videoping", msg]).status();
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("videoping: lock {}: {e} (starting anyway)", path.display());
            None
        }
    }
}

fn main() -> eframe::Result {
    maybe_restart_on_nvidia();
    // after the NVIDIA restart decision: the restarted process takes the lock
    let _lock = single_instance();
    if let Err(e) = video::init() {
        eprintln!("gstreamer init failed: {e}");
    } else {
        // find a working hardware H.264 encoder for recording (background, ~1 s)
        record::start_probe();
    }
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default()
            .with_title("videoping")
            .with_app_id("videoping")
            .with_icon(app_icon())
            .with_maximized(true)
            .with_min_inner_size([1000.0, 600.0]),
        // the self-test must not touch the stored window state / ui layout
        persist_window: std::env::var("VIDEOPING_SELFTEST").is_err(),
        persistence_path: std::env::var("VIDEOPING_SELFTEST")
            .ok()
            .map(|d| std::path::PathBuf::from(d).join("selftest-app.ron")),
        ..Default::default()
    };
    eframe::run_native("videoping", options, Box::new(|cc| Ok(Box::new(app::App::new(cc)))))
}
