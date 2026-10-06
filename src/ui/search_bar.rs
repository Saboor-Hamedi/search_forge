use egui::{Color32, Frame, Margin, Pos2, Rounding, Stroke, Ui, Vec2};

pub fn render_search_bar(
    ui: &mut Ui,
    query: &mut String,
    open_settings: &mut bool,
    _result_count: usize,
) {
    // Edge-to-edge container: matching body background (no border)
    Frame::none()
        .fill(Color32::from_rgb(18, 19, 23))
        .stroke(Stroke::NONE)
        .inner_margin(Margin::symmetric(14.0, 11.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Lucide-style search icon badge
                ui.label(
                    egui::RichText::new("🔍")
                        .size(16.0)
                        .color(Color32::from_rgb(180, 185, 200)),
                );
                ui.add_space(8.0);

                // Large, clean, borderless search input matching body
                let input_id = egui::Id::new("main_search_text_edit");
                let text_edit = egui::TextEdit::singleline(query)
                    .id(input_id)
                    .hint_text("Type to search files, code, docs...")
                    .font(egui::FontId::proportional(16.0))
                    .text_color(Color32::WHITE)
                    .frame(false)
                    .desired_width((ui.available_width() - 120.0).max(100.0));

                let response = ui.add(text_edit);
                let has_focused_id = ui.make_persistent_id("search_input_initial_focused");
                let has_focused = ui.data(|d| d.get_temp::<bool>(has_focused_id).unwrap_or(false));
                if !has_focused {
                    response.request_focus();
                    ui.data_mut(|d| d.insert_temp(has_focused_id, true));
                }

                if !query.is_empty() {
                    let (clear_rect, clear_resp) = ui.allocate_exact_size(Vec2::new(20.0, 20.0), egui::Sense::click());
                    let cross_col = if clear_resp.hovered() {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(150, 155, 170)
                    };
                    let pad = 5.0;
                    ui.painter().line_segment(
                        [
                            Pos2::new(clear_rect.min.x + pad, clear_rect.min.y + pad),
                            Pos2::new(clear_rect.max.x - pad, clear_rect.max.y - pad),
                        ],
                        Stroke::new(1.5, cross_col),
                    );
                    ui.painter().line_segment(
                        [
                            Pos2::new(clear_rect.min.x + pad, clear_rect.max.y - pad),
                            Pos2::new(clear_rect.max.x - pad, clear_rect.min.y + pad),
                        ],
                        Stroke::new(1.5, cross_col),
                    );
                    if clear_resp.on_hover_text("Clear search").clicked() {
                        query.clear();
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Crisp native custom vector close button (never broken Unicode)
                    let (close_rect, close_resp) = ui.allocate_exact_size(Vec2::new(26.0, 26.0), egui::Sense::click());
                    let close_bg = if close_resp.is_pointer_button_down_on() {
                        Color32::from_rgb(180, 45, 45)
                    } else if close_resp.hovered() {
                        Color32::from_rgb(200, 55, 55)
                    } else {
                        Color32::from_rgb(28, 30, 38)
                    };
                    let stroke_col = if close_resp.hovered() {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(190, 195, 210)
                    };

                    ui.painter().rect_filled(close_rect, Rounding::same(5.0), close_bg);
                    let pad = 8.0;
                    ui.painter().line_segment(
                        [
                            Pos2::new(close_rect.min.x + pad, close_rect.min.y + pad),
                            Pos2::new(close_rect.max.x - pad, close_rect.max.y - pad),
                        ],
                        Stroke::new(1.5, stroke_col),
                    );
                    ui.painter().line_segment(
                        [
                            Pos2::new(close_rect.min.x + pad, close_rect.max.y - pad),
                            Pos2::new(close_rect.max.x - pad, close_rect.min.y + pad),
                        ],
                        Stroke::new(1.5, stroke_col),
                    );

                    if close_resp.on_hover_text("Close SearchForge").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        std::process::exit(0);
                    }

                    // Sleek filter toggle button
                    let (filter_fill, filter_stroke, filter_text_col) = if *open_settings {
                        (
                            Color32::from_rgb(255, 230, 0),
                            Stroke::NONE,
                            Color32::BLACK,
                        )
                    } else {
                        (
                            Color32::from_rgb(28, 30, 38),
                            Stroke::new(1.0, Color32::from_rgb(48, 52, 64)),
                            Color32::from_rgb(180, 185, 200),
                        )
                    };

                    let filter_btn = egui::Button::new(
                        egui::RichText::new("⚙ Filter")
                            .size(12.0)
                            .strong()
                            .color(filter_text_col),
                    )
                    .fill(filter_fill)
                    .stroke(filter_stroke)
                    .rounding(Rounding::same(5.0))
                    .min_size(Vec2::new(65.0, 26.0));

                    if ui.add(filter_btn).clicked() {
                        *open_settings = !*open_settings;
                    }
                });
            });
        });
}



