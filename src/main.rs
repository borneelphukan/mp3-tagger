use crate::ui::Mp3TaggerApp;
use std::sync::Arc;

mod models;
mod services;
mod ui;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let icon = load_icon();
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 800.0])
        .with_min_inner_size([800.0, 600.0])
        .with_title("MP3 Tagger");

    if let Some(icon_data) = icon {
        viewport = viewport.with_icon(icon_data);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    // Set the tokio handle for async tasks
    crate::services::llm::set_tokio_handle(tokio::runtime::Handle::current());

    // Run the app
    eframe::run_native(
        "MP3 Tagger",
        options,
        Box::new(|cc| Ok(Box::new(Mp3TaggerApp::new(cc)))),
    )
}

fn load_icon() -> Option<Arc<egui::IconData>> {
    // Try to load icon from assets
    if let Ok(icon_bytes) = std::fs::read("assets/icon.png") {
        if let Ok(image) = image::load_from_memory(&icon_bytes) {
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();
            return Some(Arc::new(egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            }));
        }
    }
    None
}
