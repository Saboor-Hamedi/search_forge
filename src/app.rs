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

        let store = Arc::new(RwLock::new(Vec::new()));
        let ignored_patterns = default_ignored_patterns();
        let icon_cache = IconCache::new();
        start_background_scan(ignored_patterns.clone(), store.clone(), icon_cache.clone());

        let config = crate::utils::config::AppConfig::load();

        Self {
            search_query: String::new(),
            last_search_query: String::new(),
            store,
            results: Vec::new(),
            selected_index: None,
            open_settings: false,
            ignored_patterns,
            left_panel_width: 280.0, // Sleek compact left panel by default
            pdf_renderer: crate::ui::pdf_renderer::PdfRenderer::new(),
            updater: UpdateManager::new(),
            icon_cache,
            initial_apps_loaded: false,
            config,
            open_preferences: false,
        }
    }
}

impl App for SearchForgeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Poll background updater events on each frame
        self.updater.poll_updates();

        // Global Spotlight shortcuts: Alt + K opens/focuses, Ctrl + K closes or hides
        let alt_k_pressed = ctx.input_mut(|i| i.consume_key(egui::Modifiers::ALT, egui::Key::K));
        let ctrl_k_pressed = ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::COMMAND, egui::Key::K)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::K)
        });

        if alt_k_pressed {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        } else if ctrl_k_pressed {
            if self.config.hide_on_close {
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                std::process::exit(0);
            }
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
                if self.config.hide_on_close {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    std::process::exit(0);
                }
            }
        }

        // Keyboard navigation (VS Code Ctrl+P / Spotlight style)
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)) {
            if !self.results.is_empty() {
                let next = match self.selected_index {
                    Some(idx) => (idx + 1).min(self.results.len().min(10) - 1),
                    None => 0,
                };
                self.selected_index = Some(next);
            }
        } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)) {
            if !self.results.is_empty() {
                let prev = match self.selected_index {
                    Some(idx) => idx.saturating_sub(1),
                    None => 0,
                };
                self.selected_index = Some(prev);
            }
        } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
            if let Some(idx) = self.selected_index {
                if let Some(file) = self.results.get(idx) {
                    let _ = open::that(file.launch_target());
                }
            }
        }

        let query_changed = self.search_query != self.last_search_query;
        let needs_initial_population = !self.initial_apps_loaded && self.search_query.trim().is_empty();
        if query_changed || needs_initial_population {
            if let Ok(store_lock) = self.store.read() {
                if !store_lock.is_empty() {
                    self.results = search_records(&store_lock, &self.search_query, 10);
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

        filter_modal::render_filter_modal(ctx, &mut self.open_settings, &mut self.ignored_patterns);

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
                                results_list::render_results_list(ui, &self.results, &mut self.selected_index, &self.icon_cache);
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
                                preview_panel::render_preview_panel(ui, selected_file, &mut self.pdf_renderer, &self.icon_cache);
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
    }
}


