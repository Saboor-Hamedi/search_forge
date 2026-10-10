#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod db;
mod indexer;
mod ui;
mod utils;

use app::SearchForgeApp;
use eframe::NativeOptions;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    let is_autostart = args.iter().any(|a| a == "--autostart" || a == "-a");

    let icon_data = eframe::icon_data::from_png_bytes(include_bytes!("assets/icons/image_64.png")).ok();

    // Spotlight default size: [680.0, 420.0]
    let initial_size = [680.0, 420.0];

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size(initial_size)
        .with_resizable(false)
        .with_maximize_button(false)
        .with_title("SearchForge")
        .with_decorations(false)
        .with_transparent(true)
        .with_active(!is_autostart)
        .with_visible(!is_autostart);

    if let Some(icon) = icon_data {
        viewport = viewport.with_icon(icon);
    }

    let options = NativeOptions {
        viewport,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "SearchForge",
        options,
        Box::new(move |cc| Box::new(SearchForgeApp::new(cc, is_autostart))),
    )
}
