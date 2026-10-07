use egui::{Color32, Frame, Margin, Pos2, Rounding, Stroke, Ui, Vec2};

pub fn render_search_bar(
    ui: &mut Ui,
    query: &mut String,
    open_settings: &mut bool,
    _result_count: usize,
) {
    let input_id = egui::Id::new("main_search_text_edit");

    // Clean, borderless edge-to-edge container matching body background
    Frame::none()
        .fill(Color32::from_rgb(18, 19, 23))
        .inner_margin(Margin::symmetric(14.0, 11.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                // 1. Search Icon (size 17.0px)
                ui.label(
                    egui::RichText::new("🔍")
                        .size(17.0)
                        .color(Color32::from_rgb(160, 165, 180)),
                );

                // Right side controls reserve width: Filter (~66px) + Gap (12.0px) + Close (26px) + Separation gap (16px) = ~120px
                let right_controls_width = 125.0;
                let text_width = (ui.available_width() - right_controls_width).max(120.0);

                // 2. Large, borderless, clean search input (+2px font size: 17.0px)
                let text_edit = egui::TextEdit::singleline(query)
                    .id(input_id)
                    .hint_text("Search files, software, code...")
                    .font(egui::FontId::proportional(17.0))
                    .text_color(Color32::WHITE)
                    .frame(false)
                    .desired_width(text_width);

                let response = ui.add(text_edit);
                let has_focused_id = ui.make_persistent_id("search_input_initial_focused");
                let has_focused = ui.data(|d| d.get_temp::<bool>(has_focused_id).unwrap_or(false));
                if !has_focused {
                    response.request_focus();
                    ui.data_mut(|d| d.insert_temp(has_focused_id, true));
                }

                // 3. Right side controls: cleanly positioned on right with ample gap, flat styling
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;

                    // Close button (×) - Flat neobrutalist icon button
                    let (close_rect, close_resp) = ui.allocate_exact_size(Vec2::new(26.0, 26.0), egui::Sense::click());
                    let close_resp = close_resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                    let close_bg = if close_resp.is_pointer_button_down_on() {
                        Color32::from_rgb(180, 45, 45)
                    } else if close_resp.hovered() {
                        Color32::from_rgb(200, 55, 55)
                    } else {
                        Color32::TRANSPARENT // Flat
                    };
                    let stroke_col = if close_resp.hovered() {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(150, 155, 170)
                    };

                    if close_bg != Color32::TRANSPARENT {
                        ui.painter().rect_filled(close_rect, Rounding::same(4.0), close_bg);
                    }

                    let pad = 8.0;
                    ui.painter().line_segment(
                        [
                            Pos2::new(close_rect.min.x + pad, close_rect.min.y + pad),
                            Pos2::new(close_rect.max.x - pad, close_rect.max.y - pad),
                        ],
                        Stroke::new(1.4, stroke_col),
                    );
                    ui.painter().line_segment(
                        [
                            Pos2::new(close_rect.min.x + pad, close_rect.max.y - pad),
                            Pos2::new(close_rect.max.x - pad, close_rect.min.y + pad),
                        ],
                        Stroke::new(1.4, stroke_col),
                    );

                    if close_resp.on_hover_text("Close SearchForge").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        std::process::exit(0);
                    }

                    // Filter toggle button (⚙ Filter) - Flat styling with hover background matching file row
                    let filter_btn_id = ui.make_persistent_id("filter_toggle_btn");
                    let was_filter_hovered = ui.data(|d| d.get_temp::<bool>(filter_btn_id).unwrap_or(false));

                    let filter_btn_text = if *open_settings {
                        egui::RichText::new("⚙ Filter")
                            .size(13.0)
                            .strong()
                            .color(Color32::from_rgb(255, 230, 0))
                    } else if was_filter_hovered {
                        egui::RichText::new("⚙ Filter")
                            .size(13.0)
                            .color(Color32::WHITE)
                    } else {
                        egui::RichText::new("⚙ Filter")
                            .size(13.0)
                            .color(Color32::from_rgb(160, 165, 180))
                    };

                    let filter_fill = if *open_settings {
                        Color32::from_rgb(32, 35, 46)
                    } else if was_filter_hovered {
                        Color32::from_rgb(26, 28, 36) // Same hover background as file rows on left side
                    } else {
                        Color32::TRANSPARENT // Flat
                    };

                    let filter_btn = egui::Button::new(filter_btn_text)
                        .fill(filter_fill)
                        .stroke(Stroke::NONE) // Flat: no border stroke
                        .rounding(Rounding::same(4.0))
                        .min_size(Vec2::new(64.0, 26.0));

                    let filter_resp = ui.add(filter_btn)
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text("Configure folder filters & exclusions");

                    ui.data_mut(|d| d.insert_temp(filter_btn_id, filter_resp.hovered()));

                    if filter_resp.clicked() {
                        *open_settings = !*open_settings;
                    }
                });
            });
        });
}
