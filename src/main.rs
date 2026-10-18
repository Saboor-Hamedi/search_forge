#![windows_subsystem = "windows"]

mod app;
mod db;
mod indexer;
mod ui;
mod utils;

use app::SearchForgeApp;
use eframe::NativeOptions;

#[cfg(target_os = "windows")]
fn detach_any_console() {
    #[link(name = "kernel32")]
    extern "system" {
        fn FreeConsole() -> i32;
    }
    unsafe {
        let _ = FreeConsole();
    }
}

fn main() -> Result<(), eframe::Error> {
    #[cfg(target_os = "windows")]
    detach_any_console();

    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    let is_autostart = args
        .iter()
        .any(|a| a == "--autostart" || a == "-a" || a == "--silent");
    let force_show = args.iter().any(|a| a == "--show");

    // If an instance of SearchForge is already running in the background:
    // - on autostart/install/update (--autostart), exit quietly so no duplicate process is created
    // - on manual launch (without --autostart), ask the running instance to show Spotlight and exit
    if utils::tray_hotkey::notify_existing_instance(!is_autostart) {
        return Ok(());
    }

    // Start hidden in the system tray by default on startup, installation, and updates
    // so Spotlight only opens when summoned via shortcut (Alt+K) or tray icon.
    let start_hidden = !force_show;

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
        .with_active(!start_hidden)
        .with_visible(!start_hidden);

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
        Box::new(move |cc| Box::new(SearchForgeApp::new(cc, start_hidden))),
    )
}
