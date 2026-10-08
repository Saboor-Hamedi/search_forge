use eframe::App;
use egui::{Color32, Context, Pos2, Rect, Rounding, Stroke, Vec2};
use std::sync::{Arc, RwLock};

use crate::indexer::filters::default_ignored_patterns;
use crate::indexer::icon_cache::IconCache;
use crate::indexer::scanner::{search_records, start_background_scan, FileRecord};
use crate::ui::{filter_modal, footer, preview_panel, results_list, search_bar};
use crate::utils::updater::UpdateManager;

pub struct SearchForgeApp {
    search_query: String,
    last_search_query: String,
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
    is_window_visible: bool,
    keyboard_navigated: bool,
    preview_cache: crate::ui::preview_panel::PreviewCache,
}

impl SearchForgeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Enable inline image rendering in egui
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Apply sleek Neobrutalist / VS Code palette style visuals
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::TRANSPARENT;
        visuals.window_fill = Color32::from_rgb(18, 19, 23);
        visuals.extreme_bg_color = Color32::from_rgb(14, 15, 18);
        visuals.selection.bg_fill = Color32::from_rgb(255, 230, 0);
        visuals.selection.stroke = Stroke::NONE;
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(18, 19, 23);
        visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(24, 25, 30);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(38, 40, 48));
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(30, 32, 40);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(50, 54, 68));
        visuals.widgets.active.bg_fill = Color32::from_rgb(255, 230, 0);
        visuals.window_rounding = Rounding::same(6.0);
        cc.egui_ctx.set_visuals(visuals);

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

        // Pre-extract icons for top 10 initial applications synchronously so they appear on frame 0
        for app in discovered_apps.iter().take(10) {
            let path_to_extract = if let Some(ref sc) = app.shortcut_path {
                std::path::Path::new(sc)
            } else {
                std::path::Path::new(&app.icon_path)
            };
            if let Some((w, h, rgba)) = crate::indexer::win_icon::extract_icon(path_to_extract) {
                icon_cache.insert_raw_icon(&app.icon_path, w, h, rgba);
            }
        }

        let initial_10: Vec<FileRecord> = app_records.iter().take(10).cloned().collect();
        let initial_selected = if initial_10.is_empty() { None } else { Some(0) };

        let store = Arc::new(RwLock::new(app_records));
        let ignored_patterns = default_ignored_patterns();
        start_background_scan(ignored_patterns.clone(), store.clone(), icon_cache.clone());

        let config = crate::utils::config::AppConfig::load();
        let tray_manager = crate::utils::tray_hotkey::TrayHotkeyManager::new(cc.egui_ctx.clone());

        Self {
            search_query: String::new(),
            last_search_query: String::new(),
            store,
            results: initial_10,
            selected_index: initial_selected,
            open_settings: false,
            ignored_patterns,
            left_panel_width: 280.0, // Sleek compact left panel by default
            pdf_renderer: crate::ui::pdf_renderer::PdfRenderer::new(),
            updater: UpdateManager::new(),
            icon_cache,
            initial_apps_loaded: true,
            config,
            open_preferences: false,
            tray_manager,
            is_window_visible: true,
            keyboard_navigated: false,
            preview_cache: crate::ui::preview_panel::PreviewCache::new(),
        }
    }

    pub fn show_window(&mut self, ctx: &Context) {
        self.is_window_visible = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        // Reset focus state so search bar gets focus immediately
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("search_input_initial_focused"), false));
        ctx.request_repaint();

        #[cfg(target_os = "windows")]
        if let Some(hwnd) = crate::utils::tray_hotkey::find_searchforge_window() {
            crate::utils::tray_hotkey::bring_window_to_foreground(hwnd);
        }
    }

    pub fn hide_window(&mut self, ctx: &Context) {
        self.is_window_visible = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        ctx.request_repaint();

        #[cfg(target_os = "windows")]
        unsafe {
            crate::utils::tray_hotkey::hide_searchforge();
        }
    }
}

impl App for SearchForgeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Poll background updater events on each frame
        self.updater.poll_updates();

        // Process OS tray and global Alt+K hotkey events
        while let Some(event) = self.tray_manager.try_recv() {
            match event {
                crate::utils::tray_hotkey::TrayEvent::Show => {
                    self.show_window(ctx);
                }
                crate::utils::tray_hotkey::TrayEvent::Hide => {
                    self.hide_window(ctx);
                }
                crate::utils::tray_hotkey::TrayEvent::Quit => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    std::process::exit(0);
                }
            }
        }

        // In-app Alt + K shortcut: toggles/hides window to tray
        let alt_k_pressed = ctx.input_mut(|i| i.consume_key(egui::Modifiers::ALT, egui::Key::K));
        if alt_k_pressed {
            self.hide_window(ctx);
        }

        // High-priority Escape key handling: works even when the search bar is focused!
        let esc_pressed = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if esc_pressed {
            if !self.search_query.is_empty() {
                self.search_query.clear();
                self.results.clear();
                self.selected_index = None;
            } else if self.open_preferences {
                self.open_preferences = false;
            } else if self.open_settings {
                self.open_settings = false;
            } else {
                // When search is empty and Esc is pressed, hide to tray!
                self.hide_window(ctx);
            }
        }

        // Keyboard navigation (VS Code Ctrl+P / Spotlight style) - only active when modals are closed
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
                    }
                }
            }
        }

        let query_changed = self.search_query != self.last_search_query;
        let needs_initial_population = !self.initial_apps_loaded && self.search_query.trim().is_empty();
        if query_changed || needs_initial_population {
            if let Ok(store_lock) = self.store.read() {
                if !store_lock.is_empty() {
                    let limit = 10;
                    self.results = search_records(&store_lock, &self.search_query, limit);
                    if query_changed {
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
        }

        // Unified rounded window container with native 6px rounding and 1px border
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(18, 19, 23))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(42, 46, 58)))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ctx, |ui| {
                let available_total_w = ui.available_width();

                // 1. Top Search Bar (Flush borderless input, cleanly placed right action buttons, with auto-completion)
                let top_suggestion = self.results.first().map(|r| r.display_name().to_string());
                search_bar::render_search_bar(
                    ui,
                    &mut self.search_query,
                    &mut self.open_settings,
                    self.results.len(),
                    top_suggestion.as_deref(),
                    self.config.hide_on_close,
                );

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

                        // Neobrutalist Centered Draggable Resize Knob / Separator
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

                        // Sleek divider line
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

                        // Sleek tactile knob handle
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

                        // Right preview panel (dynamically centered)
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

                // 3. Subtle Bottom Footer
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

        // Render filter modal on top of CentralPanel
        filter_modal::render_filter_modal(ctx, &mut self.open_settings, &mut self.ignored_patterns);
    }
}


