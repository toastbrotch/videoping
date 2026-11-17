//! videoping — personal live vjing tool. See features.md.

mod app;
mod gfx;
mod isf;
mod ndi;
mod render;
mod state;
mod video;
mod widgets;

fn main() -> eframe::Result {
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
        ..Default::default()
    };
    eframe::run_native("videoping", options, Box::new(|cc| Ok(Box::new(app::App::new(cc)))))
}
