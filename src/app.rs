use eframe::App;
use egui::{Color32, Context, Pos2, Rect, Rounding, Stroke, Vec2};
use std::sync::{Arc, RwLock};

use crate::indexer::filters::default_ignored_patterns;
use crate::indexer::icon_cache::IconCache;
use crate::indexer::scanner::{search_records, start_background_scan, FileRecord};
use crate::ui::{filter_modal, footer, preview_panel, results_list, search_bar, spotlight};
use crate::utils::lifecycle::WindowLifecycle;
use crate::utils::updater::UpdateManager;

const SPOTLIGHT_SIZE: [f32; 2] = [680.0, 420.0];
const FULL_WINDOW_SIZE: [f32; 2] = [840.0, 520.0];

pub struct SearchForgeApp {
    search_query: String,
    last_search_query: String,
    last_ignored_patterns: Vec<String>,
    store: Arc<RwLock<Vec<FileRecord>>>,
    results: Vec<FileRecord>,
    selected_index: Option<usize>,
    open_settings: bool,
    ignored_patterns: Vec<String>,
    left_panel_width: f32,
    pdf_renderer: crate::ui::pdf_renderer::PdfRenderer,
    updater: UpdateManager,
    icon_cache: IconCache,
    initial_apps_loaded: bool,
    config: crate::utils::config::AppConfig,
    open_preferences: bool,
    tray_manager: crate::utils::tray_hotkey::TrayHotkeyManager,
    lifecycle: WindowLifecycle,
    keyboard_navigated: bool,
    preview_cache: crate::ui::preview_panel::PreviewCache,
    suppress_k_frames: u8,
    applied_theme_init: bool,
    spotlight_focus_grace_frames: u8,
    spotlight_has_focused: bool,
}

impl SearchForgeApp {
    pub fn new(cc: &eframe::CreationContext<'_>, is_autostart: bool) -> Self {
        // Enable inline image rendering in egui
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Configure system fallback fonts for multilingual support (Arabic, Persian, CJK, Cyrillic, symbols)
        let mut fonts = egui::FontDefinitions::default();
        crate::utils::font::setup_multilingual_fonts(&mut fonts);
        cc.egui_ctx.set_fonts(fonts);

        let config = crate::utils::config::AppConfig::load();

        // Apply sleek dark / glass theme base visuals
        Self::apply_theme_visuals(&cc.egui_ctx, config.theme);

        let icon_cache = IconCache::new();
        icon_cache.set_context(cc.egui_ctx.clone());

        // Pre-load top installed applications synchronously so they appear instantly on frame 0
        let discovered_apps = crate::indexer::app_scanner::discover_installed_applications();
        let mut app_records = Vec::with_capacity(discovered_apps.len());
        for app in &discovered_apps {
            let norm_name = crate::utils::unicode::normalize_for_search(&app.display_name);
            let record = FileRecord {
                name: app.name.clone(),
                path: app.target_path.clone(),
                size: 0,
                is_dir: false,
                item_type: crate::indexer::app_scanner::SearchResultType::Application,
                app_metadata: Some(crate::indexer::app_scanner::AppMetadata {
                    name: app.name.clone(),
                    display_name: app.display_name.clone(),
                    publisher: app.publisher.clone(),
                    version: app.version.clone(),
                    target_path: app.target_path.clone(),
                    shortcut_path: app.shortcut_path.clone(),
                    icon_path: app.icon_path.clone(),
                    source: app.source.clone(),
                }),
                norm_name,
            };
            icon_cache.request_icon(&app.icon_path, &app.icon_path);
            app_records.push(record);
        }

        // Pre-extract icons for top initial applications synchronously so they appear on frame 0
        for app in discovered_apps.iter().take(12) {
            let path_to_extract = if let Some(ref sc) = app.shortcut_path {
                std::path::Path::new(sc)
            } else {
                std::path::Path::new(&app.icon_path)
            };
            if let Some((w, h, rgba)) = crate::indexer::win_icon::extract_icon(path_to_extract) {
                icon_cache.insert_raw_icon(&app.icon_path, w, h, rgba);
            }
        }

        let initial_records: Vec<FileRecord> = app_records.iter().take(10).cloned().collect();
        let initial_selected = if initial_records.is_empty() { None } else { Some(0) };

        let store = Arc::new(RwLock::new(app_records));
        let ignored_patterns = default_ignored_patterns();
        start_background_scan(ignored_patterns.clone(), store.clone(), icon_cache.clone());

        let tray_manager = crate::utils::tray_hotkey::TrayHotkeyManager::new(cc.egui_ctx.clone());

        let initial_lifecycle = if is_autostart {
            WindowLifecycle::Hidden
        } else {
            WindowLifecycle::SpotlightVisible
        };

        Self {
            search_query: String::new(),
            last_search_query: String::new(),
            last_ignored_patterns: ignored_patterns.clone(),
            store,
            results: initial_records,
            selected_index: initial_selected,
            open_settings: false,
            ignored_patterns,
            left_panel_width: 280.0,
            pdf_renderer: crate::ui::pdf_renderer::PdfRenderer::new(),
            updater: UpdateManager::new(),
            icon_cache,
            initial_apps_loaded: true,
            config,
            open_preferences: false,
            tray_manager,
            lifecycle: initial_lifecycle,
            keyboard_navigated: false,
            preview_cache: crate::ui::preview_panel::PreviewCache::new(),
            suppress_k_frames: 0,
            applied_theme_init: false,
            spotlight_focus_grace_frames: 30,
            spotlight_has_focused: false,
        }
    }

    fn apply_theme_visuals(ctx: &Context, theme: crate::utils::config::ThemeMode) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::TRANSPARENT;
        visuals.extreme_bg_color = Color32::from_rgb(14, 15, 18);
        visuals.selection.bg_fill = Color32::from_rgb(255, 230, 0);
        visuals.selection.stroke = Stroke::NONE;
        visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;

        match theme {
            crate::utils::config::ThemeMode::Glass => {
                visuals.window_fill = Color32::from_rgba_premultiplied(18, 20, 26, 210);
                visuals.widgets.noninteractive.bg_fill = Color32::from_rgba_premultiplied(18, 20, 26, 180);
                visuals.widgets.inactive.bg_fill = Color32::from_rgba_premultiplied(26, 28, 36, 170);
                visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgba_premultiplied(48, 52, 64, 180));
                visuals.widgets.hovered.bg_fill = Color32::from_rgba_premultiplied(36, 40, 52, 210);
                visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgba_premultiplied(65, 72, 90, 220));
            }
            crate::utils::config::ThemeMode::Dark => {
                visuals.window_fill = Color32::from_rgb(18, 19, 23);
                visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(18, 19, 23);
                visuals.widgets.inactive.bg_fill = Color32::from_rgb(24, 25, 30);
                visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(38, 40, 48));
                visuals.widgets.hovered.bg_fill = Color32::from_rgb(30, 32, 40);
                visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(50, 54, 68));
            }
        }
        visuals.widgets.active.bg_fill = Color32::from_rgb(255, 230, 0);
        visuals.window_rounding = Rounding::same(5.0);
        ctx.set_visuals(visuals);
    }

    pub fn show_spotlight(&mut self, ctx: &Context) {
        self.lifecycle = WindowLifecycle::SpotlightVisible;
        self.suppress_k_frames = 3;
        self.spotlight_has_focused = false;
        self.spotlight_focus_grace_frames = 30;
        crate::utils::window_effects::resize_and_center_window(SPOTLIGHT_SIZE[0], SPOTLIGHT_SIZE[1]);
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::from(SPOTLIGHT_SIZE)));
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        ctx.request_repaint();

        crate::utils::window_effects::set_window_taskbar_presence(false);

        #[cfg(target_os = "windows")]
        if let Some(hwnd) = crate::utils::tray_hotkey::find_searchforge_window() {
            crate::utils::tray_hotkey::bring_window_to_foreground(hwnd);
        }
    }

    pub fn show_full_window(&mut self, ctx: &Context) {
        self.lifecycle = WindowLifecycle::FullWindowVisible;
        self.suppress_k_frames = 3;
        crate::utils::window_effects::resize_and_center_window(FULL_WINDOW_SIZE[0], FULL_WINDOW_SIZE[1]);
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::from(FULL_WINDOW_SIZE)));
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        ctx.request_repaint();

        crate::utils::window_effects::set_window_taskbar_presence(true);

        #[cfg(target_os = "windows")]
        if let Some(hwnd) = crate::utils::tray_hotkey::find_searchforge_window() {
            crate::utils::tray_hotkey::bring_window_to_foreground(hwnd);
        }
    }

    pub fn hide_to_tray(&mut self, ctx: &Context) {
        self.lifecycle = WindowLifecycle::Hidden;
        self.open_settings = false;
        self.open_preferences = false;
        self.spotlight_has_focused = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        ctx.request_repaint();

        #[cfg(target_os = "windows")]
        if let Some(hwnd) = crate::utils::tray_hotkey::find_searchforge_window() {
            crate::utils::tray_hotkey::hide_searchforge_window(hwnd);
        }
    }
}

impl App for SearchForgeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Initialize native backdrop blur on first frame
        if !self.applied_theme_init {
            self.applied_theme_init = true;
            let is_glass = self.config.theme == crate::utils::config::ThemeMode::Glass;
            crate::utils::window_effects::apply_native_glass_to_window(is_glass);

            // If initial mode is Hidden (launched on startup), ensure window stays hidden
            if self.lifecycle == WindowLifecycle::Hidden {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                #[cfg(target_os = "windows")]
                if let Some(hwnd) = crate::utils::tray_hotkey::find_searchforge_window() {
                    crate::utils::tray_hotkey::hide_searchforge_window(hwnd);
                }
            } else if self.lifecycle == WindowLifecycle::SpotlightVisible {
                crate::utils::window_effects::resize_and_center_window(SPOTLIGHT_SIZE[0], SPOTLIGHT_SIZE[1]);
                crate::utils::window_effects::set_window_taskbar_presence(false);
            } else if self.lifecycle == WindowLifecycle::FullWindowVisible {
                crate::utils::window_effects::resize_and_center_window(FULL_WINDOW_SIZE[0], FULL_WINDOW_SIZE[1]);
                crate::utils::window_effects::set_window_taskbar_presence(true);
            }
        }

        // Suppress stray 'k' key leakage when waking up via Alt+K
        if self.suppress_k_frames > 0 {
            self.suppress_k_frames -= 1;
            ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::ALT, egui::Key::K);
                i.consume_key(egui::Modifiers::NONE, egui::Key::K);
                i.events.retain(|e| match e {
                    egui::Event::Key { key: egui::Key::K, .. } => false,
                    egui::Event::Text(t) if t.eq_ignore_ascii_case("k") => false,
                    _ => true,
                });
            });
        }

        // Poll background updater events on each frame
        self.updater.poll_updates();

        // Process OS tray and global Alt+K hotkey events
        while let Some(event) = self.tray_manager.try_recv() {
            match event {
                crate::utils::tray_hotkey::TrayEvent::ShowSpotlight => {
                    self.show_spotlight(ctx);
                }
                crate::utils::tray_hotkey::TrayEvent::ShowFull => {
                    self.show_full_window(ctx);
                }
                crate::utils::tray_hotkey::TrayEvent::Hide => {
                    self.hide_to_tray(ctx);
                }
                crate::utils::tray_hotkey::TrayEvent::Quit => {
                    self.lifecycle = WindowLifecycle::ShuttingDown;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    std::process::exit(0);
                }
            }
        }

        // Spotlight auto-dismiss on focus lost (exact PowerToys Run behavior)
        if self.lifecycle.is_spotlight() {
            let is_focused = crate::utils::window_effects::is_searchforge_focused()
                || ctx.input(|i| i.viewport().focused.unwrap_or(false));

            if is_focused {
                // Confirm that Spotlight has successfully gained focus
                self.spotlight_has_focused = true;
                self.spotlight_focus_grace_frames = 15;
            } else if self.spotlight_has_focused {
                // Only dismiss once Spotlight has gained focus and then subsequently lost it
                if self.spotlight_focus_grace_frames > 0 {
                    self.spotlight_focus_grace_frames -= 1;
                } else {
                    // User switched to another app or clicked outside -> cleanly dismiss Spotlight to tray
                    self.search_query.clear();
                    self.selected_index = None;
                    self.hide_to_tray(ctx);
                }
            }
        }

        // In-app Alt + K shortcut: if pressed when already open and not suppressed, toggle closed to tray
        let alt_k_pressed = ctx.input_mut(|i| i.consume_key(egui::Modifiers::ALT, egui::Key::K));
        if alt_k_pressed && self.suppress_k_frames == 0 {
            if self.lifecycle.is_visible() {
                self.hide_to_tray(ctx);
            }
        }

        // Ctrl + Shift + E shortcut: toggle between Spotlight launcher and Full window mode
        let ctrl_shift_e_pressed = ctx.input_mut(|i| {
            i.consume_key(
                egui::Modifiers {
                    ctrl: true,
                    shift: true,
                    ..Default::default()
                },
                egui::Key::E,
            ) || i.consume_key(
                egui::Modifiers {
                    command: true,
                    shift: true,
                    ..Default::default()
                },
                egui::Key::E,
            )
        });
        if ctrl_shift_e_pressed {
            if self.lifecycle.is_spotlight() {
                self.show_full_window(ctx);
            } else if self.lifecycle.is_full() {
                self.show_spotlight(ctx);
            } else {
                self.show_full_window(ctx);
            }
        }

        // High-priority Escape key handling
        let esc_pressed = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if esc_pressed {
            if self.open_preferences {
                self.open_preferences = false;
            } else if self.open_settings {
                self.open_settings = false;
            } else if self.lifecycle.is_spotlight() {
                // In Spotlight launcher mode: Esc immediately dismisses Spotlight!
                self.search_query.clear();
                self.selected_index = None;
                self.hide_to_tray(ctx);
            } else if !self.search_query.is_empty() {
                self.search_query.clear();
                self.results.clear();
                self.selected_index = None;
            } else {
                // In Full window mode when query is empty, Esc returns to Spotlight launcher mode
                self.show_spotlight(ctx);
            }
        }

        // Keyboard navigation (Arrow keys + Enter)
        if !self.open_settings && !self.open_preferences {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)) {
                if !self.results.is_empty() {
                    let next = match self.selected_index {
                        Some(idx) => (idx + 1).min(self.results.len().saturating_sub(1)),
                        None => 0,
                    };
                    self.selected_index = Some(next);
                    self.keyboard_navigated = true;
                }
            } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)) {
                if !self.results.is_empty() {
                    let prev = match self.selected_index {
                        Some(idx) => idx.saturating_sub(1),
                        None => 0,
                    };
                    self.selected_index = Some(prev);
                    self.keyboard_navigated = true;
                }
            } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                if let Some(idx) = self.selected_index {
                    if let Some(file) = self.results.get(idx) {
                        let _ = open::that(file.launch_target());
                        // If in Spotlight mode, dismiss launcher upon launching
                        if self.lifecycle.is_spotlight() {
                            self.hide_to_tray(ctx);
                        }
                    }
                }
            }
        }

        // Update search results smoothly
        let query_changed = self.search_query != self.last_search_query;
        let patterns_changed = self.ignored_patterns != self.last_ignored_patterns;
        let needs_initial_population = !self.initial_apps_loaded && self.search_query.trim().is_empty();
        if query_changed || patterns_changed || needs_initial_population {
            if let Ok(store_lock) = self.store.read() {
                if !store_lock.is_empty() {
                    let limit = if self.lifecycle.is_spotlight() { 8 } else { 12 };
                    self.results = search_records(&store_lock, &self.search_query, limit, &self.ignored_patterns);
                    if query_changed || patterns_changed {
                        self.selected_index = if self.results.is_empty() { None } else { Some(0) };
                    } else if self.selected_index.is_none() && !self.results.is_empty() {
                        self.selected_index = Some(0);
                    }
                    if !self.results.is_empty() {
                        self.initial_apps_loaded = true;
                    }
                }
            }
            self.last_search_query = self.search_query.clone();
            self.last_ignored_patterns = self.ignored_patterns.clone();
        }

        // Check theme mode
        let is_glass = self.config.theme == crate::utils::config::ThemeMode::Glass;

        // Container frame: Sleek rounded container with 5px radius border
        let container_frame = egui::Frame::none()
            .fill(Color32::from_rgb(18, 19, 23))
            .stroke(Stroke::new(1.0, Color32::from_rgb(42, 46, 58)))
            .rounding(Rounding::same(5.0))
            .inner_margin(egui::Margin::ZERO);

        match self.lifecycle {
            WindowLifecycle::Hidden | WindowLifecycle::Initializing | WindowLifecycle::ShuttingDown => {
                // Keep background services and indexer alive while hidden without repainting
            }

            WindowLifecycle::SpotlightVisible => {
                egui::CentralPanel::default().frame(container_frame).show(ctx, |ui| {
                    let available_total_w = ui.available_width();
                    let top_suggestion = self.results.first().map(|r| r.display_name().to_string());

                    let mut open_full_requested = false;

                    // 1. Spotlight Search Input Bar
                    spotlight::render_spotlight_search_bar(
                        ui,
                        &mut self.search_query,
                        top_suggestion.as_deref(),
                        is_glass,
                        &mut open_full_requested,
                    );

                    if open_full_requested {
                        self.show_full_window(ctx);
                        return;
                    }

                    // Divider below search bar
                    let divider_y = ui.cursor().top();
                    let divider_color = if is_glass {
                        Color32::from_rgba_premultiplied(45, 52, 70, 160)
                    } else {
                        Color32::from_rgb(30, 33, 42)
                    };
                    ui.painter().line_segment(
                        [Pos2::new(0.0, divider_y), Pos2::new(available_total_w, divider_y)],
                        Stroke::new(1.0, divider_color),
                    );

                    // 2. Spotlight Results List
                    spotlight::render_spotlight_results(
                        ui,
                        &self.results,
                        &mut self.selected_index,
                        &self.icon_cache,
                        self.keyboard_navigated,
                        is_glass,
                    );
                    self.keyboard_navigated = false;
                });
            }

            WindowLifecycle::FullWindowVisible => {
                egui::CentralPanel::default().frame(container_frame).show(ctx, |ui| {
                    let available_total_w = ui.available_width();

                    // 1. Full Search Bar (with filter toggle and close button)
                    let top_suggestion = self.results.first().map(|r| r.display_name().to_string());
                    let mut return_to_spotlight = false;
                    search_bar::render_search_bar(
                        ui,
                        &mut self.search_query,
                        &mut self.open_settings,
                        self.results.len(),
                        top_suggestion.as_deref(),
                        self.config.hide_on_close,
                        &mut return_to_spotlight,
                    );

                    if return_to_spotlight {
                        self.show_spotlight(ctx);
                        return;
                    }

                    // Subtle divider below search bar
                    let divider_y = ui.cursor().top();
                    ui.painter().line_segment(
                        [Pos2::new(0.0, divider_y), Pos2::new(available_total_w, divider_y)],
                        Stroke::new(1.0, Color32::from_rgb(30, 33, 42)),
                    );

                    // 2. Main Middle Area (Results list + Draggable Knob + Centered Preview)
                    let footer_h = 36.0;
                    let middle_h = (ui.available_height() - footer_h).max(100.0);

                    let min_left = 200.0;
                    let max_left = (available_total_w - 240.0).max(min_left);
                    self.left_panel_width = self.left_panel_width.clamp(min_left, max_left);

                    ui.allocate_ui_with_layout(
                        Vec2::new(available_total_w, middle_h),
                        egui::Layout::left_to_right(egui::Align::Min),
                        |ui| {
                            // Left results list panel
                            ui.allocate_ui_with_layout(
                                Vec2::new(self.left_panel_width, middle_h),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(self.left_panel_width);
                                    results_list::render_results_list(
                                        ui,
                                        &self.results,
                                        &mut self.selected_index,
                                        &self.icon_cache,
                                        self.keyboard_navigated,
                                        self.open_settings || self.open_preferences,
                                    );
                                    self.keyboard_navigated = false;
                                },
                            );

                            // Draggable Separator
                            let knob_width = 10.0;
                            let (knob_rect, knob_response) = ui.allocate_exact_size(
                                Vec2::new(knob_width, middle_h),
                                egui::Sense::drag(),
                            );

                            if knob_response.hovered() || knob_response.dragged() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                            }

                            if knob_response.dragged() {
                                self.left_panel_width = (self.left_panel_width + knob_response.drag_delta().x)
                                    .clamp(min_left, max_left);
                            }

                            let center_x = knob_rect.center().x;
                            let line_color = if knob_response.hovered() || knob_response.dragged() {
                                Color32::from_rgb(100, 140, 240)
                            } else {
                                Color32::from_rgb(34, 38, 48)
                            };

                            ui.painter().line_segment(
                                [Pos2::new(center_x, knob_rect.min.y), Pos2::new(center_x, knob_rect.max.y)],
                                Stroke::new(1.0, line_color),
                            );

                            let handle_h = 28.0;
                            let handle_w = 4.0;
                            let handle_rect = Rect::from_center_size(
                                Pos2::new(center_x, knob_rect.center().y),
                                Vec2::new(handle_w, handle_h),
                            );
                            let handle_color = if knob_response.dragged() {
                                Color32::from_rgb(100, 140, 240)
                            } else if knob_response.hovered() {
                                Color32::from_rgb(140, 170, 255)
                            } else {
                                Color32::from_rgb(52, 56, 70)
                            };
                            ui.painter().rect_filled(handle_rect, Rounding::same(2.0), handle_color);

                            // Right preview panel
                            let preview_width = ui.available_width();
                            self.pdf_renderer.receive_rendered_textures(ctx);
                            ui.allocate_ui_with_layout(
                                Vec2::new(preview_width, middle_h),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(preview_width);
                                    let selected_file = self.selected_index.and_then(|idx| self.results.get(idx));
                                    preview_panel::render_preview_panel(
                                        ui,
                                        selected_file,
                                        &mut self.pdf_renderer,
                                        &self.icon_cache,
                                        &mut self.preview_cache,
                                    );
                                },
                            );
                        },
                    );

                    // Divider above footer
                    let footer_y = ui.cursor().top();
                    ui.painter().line_segment(
                        [Pos2::new(0.0, footer_y), Pos2::new(available_total_w, footer_y)],
                        Stroke::new(1.0, Color32::from_rgb(26, 28, 36)),
                    );

                    // 3. Bottom Footer
                    let total_indexed = self.store.read().map(|s| s.len()).unwrap_or(0);
                    egui::Frame::none()
                        .fill(Color32::from_rgb(16, 17, 21))
                        .inner_margin(egui::Margin::symmetric(14.0, 7.0))
                        .show(ui, |ui| {
                            footer::render_footer(
                                ui,
                                &mut self.updater,
                                total_indexed,
                                &mut self.open_preferences,
                                &mut self.config,
                            );
                        });
                });

                // Filter modal
                filter_modal::render_filter_modal(ctx, &mut self.open_settings, &mut self.ignored_patterns);
            }
        }
    }
}
