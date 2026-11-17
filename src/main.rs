//! videoping — personal live vjing tool. See features.md.

mod app;
mod gfx;
mod isf;
mod ndi;
mod render;
mod state;
mod video;
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

fn main() -> eframe::Result {
    maybe_restart_on_nvidia();
    if let Err(e) = video::init() {
        eprintln!("gstreamer init failed: {e}");
    }
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default()
            .with_title("videoping")
            .with_app_id("videoping")
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
