use egui::{Color32, Frame, Margin, Rounding, Stroke, Ui, Vec2};

pub fn render_search_bar(
    ui: &mut Ui,
    query: &mut String,
    open_settings: &mut bool,
    result_count: usize,
) {
    // Edge-to-edge Neobrutalist container: zero gap from left, right, top
    Frame::none()
        .fill(Color32::from_rgb(22, 23, 29))
        .stroke(Stroke::new(1.5, Color32::from_rgb(45, 48, 60)))
        .inner_margin(Margin::symmetric(14.0, 11.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Lucide-style search icon badge
                ui.label(
                    egui::RichText::new("🔍")
                        .size(16.0)
                        .color(Color32::from_rgb(255, 230, 0)), // Neobrutalist vivid yellow
                );
                ui.add_space(8.0);

                // Large, clean, borderless search input
                let text_edit = egui::TextEdit::singleline(query)
                    .hint_text("Type to search files, code, docs...")
                    .font(egui::FontId::proportional(16.0))
                    .text_color(Color32::WHITE)
                    .frame(false)
                    .desired_width((ui.available_width() - 170.0).max(100.0));

                ui.add(text_edit);

                if !query.is_empty() {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("✕")
                                    .size(12.0)
                                    .color(Color32::from_rgb(180, 185, 200)),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Clear search")
                        .clicked()
                    {
                        query.clear();
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Sleek, modern window close button
                    let close_btn = egui::Button::new(
                        egui::RichText::new("✕")
                            .size(13.0)
                            .strong()
                            .color(Color32::from_rgb(200, 205, 215)),
                    )
                    .fill(Color32::from_rgb(34, 36, 44))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(50, 54, 66)))
                    .rounding(Rounding::same(5.0))
                    .min_size(Vec2::new(26.0, 26.0));

                    if ui
                        .add(close_btn)
                        .on_hover_text("Close SearchForge")
                        .clicked()
                    {
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

                    if !query.trim().is_empty() {
                        ui.label(
                            egui::RichText::new(format!("{} found", result_count))
                                .size(11.0)
                                .color(Color32::from_rgb(140, 145, 165)),
                        );
                    }
                });
            });
        });
}



