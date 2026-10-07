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

    let icon_data = eframe::icon_data::from_png_bytes(include_bytes!("assets/icons/image_64.png")).ok();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([780.0, 480.0])
        .with_resizable(false)
        .with_maximize_button(false)
        .with_title("SearchForge")
        .with_decorations(false)
        .with_transparent(true)
        .with_active(true);

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
        Box::new(|cc| Box::new(SearchForgeApp::new(cc))),
    )
}

