use egui::{Color32, Context, Frame, Id, Margin, Pos2, Rect, Rounding, Stroke, Vec2};

pub fn render_filter_modal(ctx: &Context, open: &mut bool, ignored_patterns: &mut Vec<String>) {
    if !*open {
        return;
    }

    let draft_id = Id::new("new_ignored_pattern_draft");
    let mut draft = ctx.data_mut(|d| d.get_temp::<String>(draft_id).unwrap_or_default());

    let mut should_close = false;
    egui::Window::new("Exclusion Filters")
        .open(open)
        .collapsible(false)
        .resizable(false)
        .fixed_size([340.0, 310.0])
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(20, 22, 28))
                .stroke(Stroke::new(1.0, Color32::from_rgb(44, 48, 60)))
                .rounding(Rounding::same(5.0))
                .inner_margin(Margin::same(14.0)),
        )
        .show(ctx, |ui| {
            ui.set_width(312.0);
            ui.label(
                egui::RichText::new("Paths matching any of these patterns will be skipped:")
                    .size(11.5)
                    .color(Color32::from_rgb(140, 145, 160)),
            );
            ui.add_space(8.0);

            // Clean, lightweight filter rows with reliable native vector checkboxes
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

                        // Very subtle background change on hover, no heavy card border
                        if row_resp.hovered() {
                            ui.painter().rect_filled(
                                row_rect,
                                Rounding::same(4.0),
                                Color32::from_rgb(26, 28, 36),
                            );
                        }

                        // Native vector checkbox (Always checked for active patterns; clicking unchecks/removes)
                        let box_size = 14.0;
                        let box_rect = Rect::from_min_size(
                            Pos2::new(row_rect.min.x + 6.0, row_rect.center().y - (box_size / 2.0)),
                            Vec2::new(box_size, box_size),
                        );

                        // Subtle yellow accent for active checked state
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

                        // Crisp checkmark symbol (dark black tick mark)
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

                        // Clicking row toggles pattern (unchecks and removes from active filters)
                        if row_resp.on_hover_text("Click to disable/remove filter").clicked() {
                            remove_index = Some(i);
                        }
                    }

                    if let Some(i) = remove_index {
                        ignored_patterns.remove(i);
                    }
                });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);

            // Add custom pattern input row
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut draft)
                        .hint_text("Enter custom pattern...")
                        .desired_width(240.0),
                );

                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Add")
                                .size(11.5)
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(38, 42, 54))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(55, 60, 75)))
                        .rounding(Rounding::same(4.0))
                        .min_size(Vec2::new(50.0, 24.0)),
                    )
                    .clicked()
                    && !draft.trim().is_empty()
                {
                    ignored_patterns.push(draft.trim().to_string());
                    draft.clear();
                }
            });

            ui.add_space(12.0);

            // Bottom action row: Reset Defaults on left, Done on right
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
                        .rounding(Rounding::same(4.0)),
                    )
                    .clicked()
                {
                    *ignored_patterns = crate::indexer::filters::default_ignored_patterns();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Done")
                                    .size(11.5)
                                    .strong()
                                    .color(Color32::BLACK),
                            )
                            .fill(Color32::from_rgb(255, 230, 0))
                            .stroke(Stroke::NONE)
                            .rounding(Rounding::same(4.0))
                            .min_size(Vec2::new(60.0, 24.0)),
                        )
                        .clicked()
                    {
                        should_close = true;
                    }
                });
            });
        });

    if should_close {
        *open = false;
    }
    ctx.data_mut(|d| d.insert_temp(draft_id, draft));
}
