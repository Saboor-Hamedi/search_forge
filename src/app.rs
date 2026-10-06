use eframe::App;
use egui::{Color32, Context, Pos2, Rect, Rounding, Stroke, Vec2};
use std::sync::{Arc, RwLock};

use crate::indexer::filters::default_ignored_patterns;
use crate::indexer::scanner::{search_records, start_background_scan, FileRecord};
use crate::ui::{filter_modal, preview_panel, results_list, search_bar};

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
}

impl SearchForgeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Enable inline image rendering in egui
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Apply sleek Neobrutalist / VS Code palette style visuals
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(18, 19, 23);
        visuals.window_fill = Color32::from_rgb(22, 23, 28);
        visuals.extreme_bg_color = Color32::from_rgb(14, 15, 18);
        visuals.selection.bg_fill = Color32::from_rgb(255, 230, 0);
        visuals.selection.stroke = Stroke::NONE;
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(18, 19, 23);
        visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(26, 27, 32);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(42, 44, 52));
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(34, 36, 44);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, Color32::from_rgb(255, 230, 0));
        visuals.widgets.active.bg_fill = Color32::from_rgb(255, 230, 0);
        visuals.window_rounding = Rounding::same(6.0);
        cc.egui_ctx.set_visuals(visuals);

        let store = Arc::new(RwLock::new(Vec::new()));
        let ignored_patterns = default_ignored_patterns();

        // Target user content folders: Downloads, Documents, Desktop, Pictures, Videos, Music, OneDrive
        let mut roots = Vec::new();
        if let Some(dl) = dirs::download_dir() {
            roots.push(dl);
        }
        if let Some(docs) = dirs::document_dir() {
            roots.push(docs);
        }
        if let Some(dt) = dirs::desktop_dir() {
            roots.push(dt);
        }
        if let Some(pic) = dirs::picture_dir() {
            roots.push(pic);
        }
        if let Some(vid) = dirs::video_dir() {
            roots.push(vid);
        }
        if let Some(mus) = dirs::audio_dir() {
            roots.push(mus);
        }
        if let Some(home) = dirs::home_dir() {
            let one_drive = home.join("OneDrive");
            if one_drive.exists() {
                roots.push(one_drive);
            }
        }

        start_background_scan(roots, ignored_patterns.clone(), store.clone());

        Self {
            search_query: String::new(),
            last_search_query: String::new(),
            store,
            results: Vec::new(),
            selected_index: None,
            open_settings: false,
            ignored_patterns,
            left_panel_width: 440.0, // Centered by default (half of 880.0)
            pdf_renderer: crate::ui::pdf_renderer::PdfRenderer::new(),
        }
    }
}

impl App for SearchForgeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // High-priority Escape key handling: works even when the search bar is focused!
        let esc_pressed = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if esc_pressed {
            if !self.search_query.is_empty() {
                self.search_query.clear();
                self.results.clear();
                self.selected_index = None;
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                std::process::exit(0);
            }
        }

        // Keyboard navigation (VS Code Ctrl+P / Spotlight style)
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)) {
            if !self.results.is_empty() {
                let next = match self.selected_index {
                    Some(idx) => (idx + 1).min(self.results.len().min(5) - 1),
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
                    let _ = open::that(&file.path);
                }
            }
        }

        let query_changed = self.search_query != self.last_search_query;
        if query_changed {
            if !self.search_query.trim().is_empty() {
                // Maximum 5 results: blazing fast in-memory search
                if let Ok(store_lock) = self.store.read() {
                    self.results = search_records(&store_lock, &self.search_query, 5);
                    self.selected_index = if self.results.is_empty() { None } else { Some(0) };
                }
            } else {
                // When deleting search query, clear results and preview instantly
                self.results.clear();
                self.selected_index = None;
            }
            self.last_search_query = self.search_query.clone();
        }


        filter_modal::render_filter_modal(ctx, &mut self.open_settings, &mut self.ignored_patterns);

        // Top Search Bar (Flush edge-to-edge: NO gap from left, right, top)
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::none().fill(Color32::from_rgb(22, 23, 29)).inner_margin(egui::Margin::ZERO))
            .show(ctx, |ui| {
                search_bar::render_search_bar(ui, &mut self.search_query, &mut self.open_settings, self.results.len());
            });

        // Main Content Area with Centered Draggable Splitter Knob
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(18, 19, 23)).inner_margin(egui::Margin::symmetric(10.0, 8.0)))
            .show(ctx, |ui| {
                let available_width = ui.available_width();
                let available_height = ui.available_height();

                // Clamp panel width
                let min_left = 220.0;
                let max_left = (available_width - 240.0).max(min_left);
                self.left_panel_width = self.left_panel_width.clamp(min_left, max_left);

                ui.horizontal(|ui| {
                    // Left results list panel (max 5 items)
                    ui.allocate_ui_with_layout(
                        Vec2::new(self.left_panel_width, available_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            results_list::render_results_list(ui, &self.results, &mut self.selected_index);
                        },
                    );

                    // Neobrutalist Centered Draggable Resize Knob / Separator
                    let knob_width = 10.0;
                    let (knob_rect, knob_response) = ui.allocate_exact_size(
                        Vec2::new(knob_width, available_height),
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
                        Color32::from_rgb(100, 140, 240) // Subtle sleek blue on hover/drag
                    } else {
                        Color32::from_rgb(38, 41, 52)
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
                        Color32::from_rgb(55, 60, 75)
                    };
                    ui.painter().rect_filled(handle_rect, Rounding::same(2.0), handle_color);

                    // Right preview panel (centered next to knob)
                    let preview_width = ui.available_width();
                    self.pdf_renderer.receive_rendered_textures(ctx);
                    ui.allocate_ui_with_layout(
                        Vec2::new(preview_width, available_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            let selected_file = self.selected_index.and_then(|idx| self.results.get(idx));
                            preview_panel::render_preview_panel(ui, selected_file, &mut self.pdf_renderer);
                        },
                    );
                });
            });
    }
}


