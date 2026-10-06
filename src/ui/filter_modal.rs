use egui::{Color32, Context, Frame, Id, Margin, Rounding, Stroke, Vec2};

pub fn render_filter_modal(ctx: &Context, open: &mut bool, ignored_patterns: &mut Vec<String>) {
    if !*open {
        return;
    }

    let draft_id = Id::new("new_ignored_pattern_draft");
    let mut draft = ctx.data_mut(|d| d.get_temp::<String>(draft_id).unwrap_or_default());

    let mut should_close = false;
    egui::Window::new("⚙ Exclusion Filters")
        .open(open)
        .collapsible(false)
        .resizable(false)
        .default_width(320.0)
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(24, 26, 33))
                .stroke(Stroke::new(1.0, Color32::from_rgb(48, 52, 65)))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(14.0)),
        )
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("Paths matching any of these patterns will be skipped:")
                    .size(11.5)
                    .color(Color32::from_rgb(150, 155, 170)),
            );
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .max_height(140.0)
                .show(ui, |ui| {
                    let mut remove: Option<usize> = None;
                    for (i, pattern) in ignored_patterns.iter().enumerate() {
                        ui.horizontal(|ui| {
                            Frame::none()
                                .fill(Color32::from_rgb(32, 35, 45))
                                .rounding(Rounding::same(4.0))
                                .inner_margin(Margin::symmetric(6.0, 3.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(pattern)
                                            .monospace()
                                            .size(11.0)
                                            .color(Color32::from_rgb(220, 225, 235)),
                                    );
                                });

                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("✕")
                                            .size(10.0)
                                            .color(Color32::from_rgb(180, 185, 200)),
                                    )
                                    .fill(Color32::TRANSPARENT),
                                )
                                .clicked()
                            {
                                remove = Some(i);
                            }
                        });
                        ui.add_space(2.0);
                    }
                    if let Some(i) = remove {
                        ignored_patterns.remove(i);
                    }
                });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut draft)
                        .hint_text("e.g. /build/ or .log")
                        .desired_width(200.0),
                );
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("+ Add")
                                .size(11.5)
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(45, 50, 65))
                        .rounding(Rounding::same(4.0)),
                    )
                    .clicked()
                    && !draft.trim().is_empty()
                {
                    ignored_patterns.push(draft.trim().to_string());
                    draft.clear();
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("↺ Reset Defaults")
                                .size(11.0)
                                .color(Color32::from_rgb(160, 165, 180)),
                        )
                        .fill(Color32::from_rgb(32, 34, 42))
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
                            .rounding(Rounding::same(4.0))
                            .min_size(Vec2::new(60.0, 22.0)),
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
