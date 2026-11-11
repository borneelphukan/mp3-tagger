#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use crate::ui::XurPorisoyApp;
use std::sync::Arc;

mod models;
mod services;
mod ui;

// Two worker threads are plenty for a couple of HTTP requests and avoid the
// default "one worker per CPU" stack/allocator footprint.
#[tokio::main(worker_threads = 2)]
async fn main() -> Result<(), eframe::Error> {
    let icon = load_icon();
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 800.0])
        .with_min_inner_size([800.0, 600.0])
        .with_title("XurPorisoy");

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
        "XurPorisoy",
        options,
        Box::new(|cc| Ok(Box::new(XurPorisoyApp::new(cc)))),
    )
}

fn load_icon() -> Option<Arc<egui::IconData>> {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon_bytes).ok()?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    Some(Arc::new(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    }))
}
