use egui::{Color32, Context, Frame, Id, Margin, Pos2, Rect, Rounding, Stroke, Vec2};

pub fn render_filter_modal(ctx: &Context, open: &mut bool, ignored_patterns: &mut Vec<String>) {
    if !*open {
        return;
    }

    let draft_id = Id::new("new_ignored_pattern_draft");
    let mut draft = ctx.data_mut(|d| d.get_temp::<String>(draft_id).unwrap_or_default());

    egui::Window::new("Exclusion Filters")
        .open(open)
        .collapsible(false)
        .resizable(false)
        .fixed_size([340.0, 310.0])
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(20, 22, 28))
                .shadow(egui::epaint::Shadow {
                    offset: [0.0, 4.0].into(),
                    blur: 16.0,
                    spread: 2.0,
                    color: Color32::from_black_alpha(160),
                })
                .stroke(Stroke::new(1.0, Color32::from_rgb(46, 50, 64)))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(14.0)),
        )
        .show(ctx, |ui| {
            ui.set_width(312.0);

            // 1. Top Custom Exclusion Input (Exact matching height 28px for input and Add button)
            let control_height = 28.0;
            let add_btn_width = 54.0;
            let spacing = 8.0;
            let text_width = (ui.available_width() - add_btn_width - spacing).max(180.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;

                let text_edit = egui::TextEdit::singleline(&mut draft)
                    .hint_text("Enter exclusion pattern...")
                    .font(egui::FontId::proportional(12.0))
                    .text_color(Color32::WHITE)
                    .margin(Margin::symmetric(8.0, 5.0))
                    .desired_width(text_width)
                    .min_size(Vec2::new(text_width, control_height));

                let edit_resp = ui.add(text_edit);

                let add_btn = egui::Button::new(
                    egui::RichText::new("Add")
                        .size(12.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(38, 42, 54))
                .stroke(Stroke::new(1.0, Color32::from_rgb(58, 64, 80)))
                .rounding(Rounding::same(4.0))
                .min_size(Vec2::new(add_btn_width, control_height));

                let add_resp = ui.add(add_btn);

                let enter_pressed = edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if (add_resp.clicked() || enter_pressed) && !draft.trim().is_empty() {
                    let new_pat = draft.trim().to_string();
                    if !ignored_patterns.contains(&new_pat) {
                        ignored_patterns.push(new_pat);
                    }
                    draft.clear();
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            // 2. Clean, lightweight filter rows with native vector checkboxes
            egui::ScrollArea::vertical()
                .id_source("filter_patterns_scroll")
                .max_height(180.0)
                .show(ui, |ui| {
                    let mut remove_index: Option<usize> = None;

                    for (i, pattern) in ignored_patterns.iter().enumerate() {
                        let (row_rect, row_resp) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 26.0),
                            egui::Sense::click(),
                        );

                        // Subtle background change on hover, no heavy card border
                        if row_resp.hovered() {
                            ui.painter().rect_filled(
                                row_rect,
                                Rounding::same(4.0),
                                Color32::from_rgb(26, 28, 36),
                            );
                        }

                        // Native vector checkbox
                        let box_size = 14.0;
                        let box_rect = Rect::from_min_size(
                            Pos2::new(row_rect.min.x + 6.0, row_rect.center().y - (box_size / 2.0)),
                            Vec2::new(box_size, box_size),
                        );

                        // Yellow accent for active checked state
                        ui.painter().rect_filled(
                            box_rect,
                            Rounding::same(3.0),
                            Color32::from_rgb(255, 230, 0),
                        );
                        ui.painter().rect_stroke(
                            box_rect,
                            Rounding::same(3.0),
                            Stroke::new(1.0, Color32::from_rgb(200, 180, 0)),
                        );

                        // Crisp checkmark symbol
                        let check_stroke = Stroke::new(1.8, Color32::BLACK);
                        let p1 = Pos2::new(box_rect.min.x + 3.0, box_rect.min.y + 7.0);
                        let p2 = Pos2::new(box_rect.min.x + 5.5, box_rect.min.y + 10.5);
                        let p3 = Pos2::new(box_rect.min.x + 11.0, box_rect.min.y + 4.0);
                        ui.painter().line_segment([p1, p2], check_stroke);
                        ui.painter().line_segment([p2, p3], check_stroke);

                        // Pattern text
                        let text_pos = Pos2::new(box_rect.max.x + 8.0, row_rect.center().y);
                        ui.painter().text(
                            text_pos,
                            egui::Align2::LEFT_CENTER,
                            pattern,
                            egui::FontId::monospace(12.0),
                            Color32::from_rgb(220, 225, 235),
                        );

                        // Clicking row toggles pattern
                        if row_resp.on_hover_text("Click to remove exclusion filter").clicked() {
                            remove_index = Some(i);
                        }
                    }

                    if let Some(i) = remove_index {
                        ignored_patterns.remove(i);
                    }
                });

            ui.add_space(10.0);

            // 3. Bottom secondary action: Reset Defaults only (Done button removed)
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Reset Defaults")
                                .size(11.0)
                                .color(Color32::from_rgb(160, 165, 180)),
                        )
                        .fill(Color32::from_rgb(30, 32, 40))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 48, 60)))
                        .rounding(Rounding::same(4.0))
                        .min_size(Vec2::new(96.0, 24.0)),
                    )
                    .clicked()
                {
                    *ignored_patterns = crate::indexer::filters::default_ignored_patterns();
                }
            });
        });

    ctx.data_mut(|d| d.insert_temp(draft_id, draft));
}
