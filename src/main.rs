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

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([880.0, 500.0])
            .with_resizable(false)
            .with_maximize_button(false)
            .with_title("SearchForge")
            .with_decorations(false)
            .with_active(true),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "SearchForge",
        options,
        Box::new(|cc| Box::new(SearchForgeApp::new(cc))),
    )
}

