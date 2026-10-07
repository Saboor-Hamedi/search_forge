use egui::{Color32, Frame, Margin, Pos2, Rect, Rounding, Stroke, Ui, Vec2};
use crate::utils::config::AppConfig;

pub fn render_settings_menu(
    ui: &mut Ui,
    open: &mut bool,
    config: &mut AppConfig,
    button_rect: Rect,
) {
    if !*open {
        return;
    }

    let menu_width = 240.0;
    let menu_height = 148.0;

    // Position pop-up menu sleekly above the button on the right
    let menu_pos = Pos2::new(
        (button_rect.max.x - menu_width).max(8.0),
        button_rect.min.y - menu_height - 6.0,
    );

    let area_id = ui.make_persistent_id("settings_popup_area");
    egui::Area::new(area_id)
        .fixed_pos(menu_pos)
        .order(egui::Order::Foreground)
        .show(ui.ctx(), |ui| {
            Frame::none()
                .fill(Color32::from_rgb(20, 22, 28))
                .stroke(Stroke::new(1.0, Color32::from_rgb(46, 50, 64)))
                .rounding(Rounding::same(8.0))
                .shadow(egui::epaint::Shadow {
                    offset: [0.0, 4.0].into(),
                    blur: 16.0,
                    spread: 2.0,
                    color: Color32::from_black_alpha(180),
                })
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.set_width(menu_width - 24.0);

                    // Title row
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("⚙ Preferences")
                                .size(13.0)
                                .strong()
                                .color(Color32::WHITE),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let close_btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new("×")
                                        .size(14.0)
                                        .color(Color32::from_rgb(140, 145, 160)),
                                )
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::NONE),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                            if close_btn.clicked() {
                                *open = false;
                            }
                        });
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Toggle 1: Open on Startup
                    let mut autostart = config.autostart;
                    if ui
                        .checkbox(&mut autostart, egui::RichText::new("Launch on Windows startup").size(12.0).color(Color32::from_rgb(220, 225, 235)))
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text("Automatically start SearchForge minimized in background when logging in")
                        .changed()
                    {
                        config.set_autostart(autostart);
                    }

                    ui.add_space(6.0);

                    // Toggle 2: Spotlight Hide instead of Close
                    let mut hide_on_close = config.hide_on_close;
                    if ui
                        .checkbox(&mut hide_on_close, egui::RichText::new("Spotlight Mode (Hide on close)").size(12.0).color(Color32::from_rgb(220, 225, 235)))
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text("Hides window into background on Close/Ctrl+K so hotkeys summon instantly")
                        .changed()
                    {
                        config.set_hide_on_close(hide_on_close);
                    }

                    ui.add_space(10.0);

                    // Quit SearchForge completely button
                    ui.horizontal(|ui| {
                        let quit_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new("Quit SearchForge")
                                    .size(11.0)
                                    .color(Color32::from_rgb(240, 110, 110)),
                            )
                            .fill(Color32::from_rgb(34, 22, 24))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(60, 32, 36)))
                            .rounding(Rounding::same(4.0))
                            .min_size(Vec2::new(110.0, 22.0)),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text("Completely exit and close SearchForge process");

                        if quit_btn.clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            std::process::exit(0);
                        }
                    });
                });
        });
}
