use egui::{Color32, Frame, Margin, Pos2, Rect, Rounding, Stroke, Ui, Vec2};

pub fn render_search_bar(
    ui: &mut Ui,
    query: &mut String,
    open_settings: &mut bool,
    _result_count: usize,
    top_suggestion: Option<&str>,
    _hide_on_close: bool,
    return_to_spotlight: &mut bool,
) {
    let input_id = egui::Id::new("main_search_text_edit");

    // Check if auto-completion applies
    let completion_suffix = if !query.trim().is_empty() {
        if let Some(suggestion) = top_suggestion {
            let lower_q = query.to_lowercase();
            let lower_s = suggestion.to_lowercase();
            if lower_s.starts_with(&lower_q) && lower_s.chars().count() > lower_q.chars().count() {
                let char_skip = query.chars().count();
                let suf: String = suggestion.chars().skip(char_skip).collect();
                Some(suf)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Clean, borderless edge-to-edge container matching body background
    Frame::none()
        .fill(Color32::from_rgb(18, 19, 23))
        .inner_margin(Margin::symmetric(14.0, 11.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                // 1. Vector Search Icon (size 17.0px)
                crate::ui::icons::draw_search_icon(ui, 17.0, Color32::from_rgb(160, 165, 180));

                // Right side controls reserve width: Spotlight (~82px) + Filter (~72px) + Close (26px) + gaps = ~205px
                let right_controls_width = 205.0;
                let text_width = (ui.available_width() - right_controls_width).max(120.0);

                // 2. Large, borderless, clean search input (+2px font size: 17.0px)
                let output = egui::TextEdit::singleline(query)
                    .id(input_id)
                    .hint_text("Search files, software, code...")
                    .font(egui::FontId::proportional(17.0))
                    .text_color(Color32::WHITE)
                    .frame(false)
                    .desired_width(text_width)
                    .show(ui);

                let response = output.response;

                // If Tab key was pressed and auto-completion is available, complete it and place cursor at end
                if completion_suffix.is_some() {
                    if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)) {
                        if let Some(suggestion) = top_suggestion {
                            *query = suggestion.to_string();
                            // Move text cursor to the very end of completed text
                            let char_len = query.chars().count();
                            let cursor = egui::text::CCursor::new(char_len);
                            let mut state = egui::text_edit::TextEditState::load(ui.ctx(), input_id).unwrap_or_default();
                            state.cursor.set_char_range(Some(egui::text::CCursorRange::one(cursor)));
                            state.store(ui.ctx(), input_id);

                            // Keep focus on input so caret never disappears!
                            response.request_focus();
                            ui.ctx().memory_mut(|m| m.request_focus(input_id));
                        }
                    }
                }

                // Ghost auto-completion display positioned directly matching text baseline with precise offset
                if let Some(ref suffix) = completion_suffix {
                    let font_id = egui::FontId::proportional(17.0);
                    let galley = output.galley;
                    // output.galley_pos is the exact top-left coordinate where egui draws the typed text!
                    let ghost_x = output.galley_pos.x + galley.size().x + 2.0;
                    let ghost_pos = Pos2::new(ghost_x, output.galley_pos.y);

                    ui.painter().text(
                        ghost_pos,
                        egui::Align2::LEFT_TOP,
                        suffix,
                        font_id,
                        Color32::from_rgb(105, 110, 125), // Elegant readable ghost gray
                    );
                }

                // Keep focus on search input permanently unless filter settings modal is open
                if !*open_settings && !response.has_focus() {
                    response.request_focus();
                }

                // 3. Right side controls: cleanly positioned on right with ample gap, flat styling
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;

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

                    crate::ui::icons::draw_close_icon(ui.painter(), close_rect, Stroke::new(1.4, stroke_col));

                    if close_resp.on_hover_text("Close to tray (Alt+K to open)").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Visible(false));
                        #[cfg(target_os = "windows")]
                        unsafe {
                            crate::utils::tray_hotkey::hide_searchforge();
                        }
                    }

                    // Spotlight launcher toggle button - Returns cleanly to compact launcher mode
                    let (spotlight_rect, spotlight_resp) = ui.allocate_exact_size(Vec2::new(76.0, 26.0), egui::Sense::click());
                    let spotlight_resp = spotlight_resp
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text("Switch to compact Spotlight launcher (Ctrl+Shift+E)");
                    let spotlight_fill = if spotlight_resp.hovered() {
                        Color32::from_rgb(34, 38, 50)
                    } else {
                        Color32::from_rgb(26, 29, 38)
                    };
                    ui.painter().rect_filled(spotlight_rect, Rounding::same(4.0), spotlight_fill);

                    let spotlight_col = if spotlight_resp.hovered() { Color32::WHITE } else { Color32::from_rgb(175, 180, 195) };
                    let spotlight_icon_rect = Rect::from_center_size(
                        Pos2::new(spotlight_rect.min.x + 13.0, spotlight_rect.center().y),
                        Vec2::splat(10.0),
                    );
                    crate::ui::icons::draw_expand_icon(ui.painter(), spotlight_icon_rect, Stroke::new(1.2, spotlight_col));
                    ui.painter().text(
                        Pos2::new(spotlight_rect.min.x + 25.0, spotlight_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        "Spotlight",
                        egui::FontId::proportional(12.0),
                        spotlight_col,
                    );

                    if spotlight_resp.clicked() {
                        *return_to_spotlight = true;
                    }

                    // Filter toggle button with vector sliders icon
                    let (filter_rect, filter_resp) = ui.allocate_exact_size(Vec2::new(64.0, 26.0), egui::Sense::click());
                    let filter_resp = filter_resp
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text("Configure folder filters & exclusions");

                    let filter_fill = if *open_settings {
                        Color32::from_rgb(34, 38, 52)
                    } else if filter_resp.hovered() {
                        Color32::from_rgb(26, 28, 36)
                    } else {
                        Color32::TRANSPARENT
                    };
                    if filter_fill != Color32::TRANSPARENT {
                        ui.painter().rect_filled(filter_rect, Rounding::same(4.0), filter_fill);
                    }

                    let filter_col = if *open_settings {
                        Color32::from_rgb(255, 230, 0)
                    } else if filter_resp.hovered() {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(160, 165, 180)
                    };

                    let filter_icon_rect = Rect::from_center_size(
                        Pos2::new(filter_rect.min.x + 12.0, filter_rect.center().y),
                        Vec2::splat(11.0),
                    );
                    crate::ui::icons::draw_sliders_icon(ui.painter(), filter_icon_rect, Stroke::new(1.2, filter_col));
                    ui.painter().text(
                        Pos2::new(filter_rect.min.x + 23.0, filter_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        "Filter",
                        egui::FontId::proportional(12.5),
                        filter_col,
                    );

                    if filter_resp.clicked() {
                        *open_settings = !*open_settings;
                    }
                });
            });
        });
}
