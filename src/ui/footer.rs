use egui::{Color32, Rect, Rounding, Stroke, Ui, Vec2};
use crate::utils::updater::{UpdateManager, UpdateState, CURRENT_VERSION};

pub fn render_footer(
    ui: &mut Ui,
    updater: &mut UpdateManager,
    indexed_count: usize,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;

        // Left side: Clean status line
        let status_color = Color32::from_rgb(115, 120, 138);
        ui.label(
            egui::RichText::new(format!("SearchForge v{}", CURRENT_VERSION))
                .size(12.0)
                .color(status_color),
        );

        ui.label(
            egui::RichText::new("·")
                .size(12.0)
                .color(Color32::from_rgb(70, 75, 90)),
        );

        let files_msg = if indexed_count > 0 {
            format!("{} items indexed", indexed_count)
        } else {
            "Indexing workspace...".to_string()
        };
        ui.label(
            egui::RichText::new(files_msg)
                .size(12.0)
                .color(status_color),
        );

        // Right side: Interactive multi-state Update / Download / Restart button
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            match updater.state() {
                UpdateState::Idle => {
                    let btn_id = ui.make_persistent_id("update_idle_btn");
                    let was_hovered = ui.data(|d| d.get_temp::<bool>(btn_id).unwrap_or(false));
                    let bg_fill = if was_hovered {
                        Color32::from_rgb(26, 28, 36)
                    } else {
                        Color32::TRANSPARENT
                    };
                    let text_color = if was_hovered {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(160, 165, 185)
                    };

                    let check_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new("Check for Updates")
                                .size(11.5)
                                .color(text_color),
                        )
                        .fill(bg_fill)
                        .stroke(Stroke::NONE) // Flat
                        .rounding(Rounding::same(4.0))
                        .min_size(Vec2::new(110.0, 22.0)),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand);

                    ui.data_mut(|d| d.insert_temp(btn_id, check_btn.hovered()));

                    if check_btn.clicked() {
                        updater.check_for_updates();
                    }
                }
                UpdateState::Checking => {
                    ui.add(
                        egui::Spinner::new().size(12.0)
                    );
                    ui.label(
                        egui::RichText::new("Checking...")
                            .size(11.5)
                            .color(Color32::from_rgb(170, 175, 195)),
                    );
                }
                UpdateState::UpToDate => {
                    let btn_id = ui.make_persistent_id("update_uptodate_btn");
                    let was_hovered = ui.data(|d| d.get_temp::<bool>(btn_id).unwrap_or(false));
                    let bg_fill = if was_hovered {
                        Color32::from_rgb(24, 30, 26)
                    } else {
                        Color32::TRANSPARENT
                    };

                    let btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new("Up to Date")
                                .size(11.5)
                                .color(Color32::from_rgb(120, 200, 140)),
                        )
                        .fill(bg_fill)
                        .stroke(Stroke::NONE) // Flat
                        .rounding(Rounding::same(4.0))
                        .min_size(Vec2::new(85.0, 22.0)),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand);

                    ui.data_mut(|d| d.insert_temp(btn_id, btn.hovered()));

                    if btn.clicked() {
                        updater.check_for_updates();
                    }
                }
                UpdateState::UpdateAvailable { version, .. } => {
                    let download_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(format!("Update to v{} ↓", version))
                                .size(11.0)
                                .strong()
                                .color(Color32::BLACK),
                        )
                        .fill(Color32::from_rgb(255, 230, 0)) // SearchForge signature yellow
                        .stroke(Stroke::NONE)
                        .rounding(Rounding::same(4.0))
                        .min_size(Vec2::new(125.0, 22.0)),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if download_btn.clicked() {
                        updater.start_download();
                    }
                }
                UpdateState::Downloading { progress, downloaded, total, .. } => {
                    let progress_val = *progress;
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(150.0, 22.0), egui::Sense::hover());
                    
                    // Background bar
                    ui.painter().rect_filled(
                        rect,
                        Rounding::same(4.0),
                        Color32::from_rgb(26, 28, 36),
                    );
                    ui.painter().rect_stroke(
                        rect,
                        Rounding::same(4.0),
                        Stroke::new(1.0, Color32::from_rgb(50, 54, 70)),
                    );

                    // Fill progress bar
                    let fill_w = rect.width() * progress_val;
                    if fill_w > 0.0 {
                        let fill_rect = Rect::from_min_size(rect.min, Vec2::new(fill_w, rect.height()));
                        ui.painter().rect_filled(
                            fill_rect,
                            Rounding::same(4.0),
                            Color32::from_rgb(255, 230, 0),
                        );
                    }

                    // Text overlay
                    let pct_text = if *total > 0 {
                        format!("Downloading {:.0}%", progress_val * 100.0)
                    } else {
                        format!("{} KB", downloaded / 1024)
                    };
                    let text_color = if progress_val > 0.5 {
                        Color32::BLACK
                    } else {
                        Color32::from_rgb(220, 225, 235)
                    };
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        pct_text,
                        egui::FontId::proportional(11.0),
                        text_color,
                    );
                }
                UpdateState::ReadyToRestart { version, .. } => {
                    let restart_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(format!("Restart & Install v{} ↻", version))
                                .size(11.0)
                                .strong()
                                .color(Color32::BLACK),
                        )
                        .fill(Color32::from_rgb(100, 225, 120)) // Vibrant restart green
                        .stroke(Stroke::NONE)
                        .rounding(Rounding::same(4.0))
                        .min_size(Vec2::new(155.0, 22.0)),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if restart_btn.clicked() {
                        updater.restart_and_install();
                    }
                }
                UpdateState::Error(err) => {
                    let err_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new("Retry Update")
                                .size(11.0)
                                .color(Color32::from_rgb(255, 120, 120)),
                        )
                        .fill(Color32::from_rgb(38, 24, 26))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(70, 36, 40)))
                        .rounding(Rounding::same(4.0))
                        .min_size(Vec2::new(95.0, 22.0)),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if err_btn.on_hover_text(err).clicked() {
                        updater.check_for_updates();
                    }
                }
            }
        });
    });
}
