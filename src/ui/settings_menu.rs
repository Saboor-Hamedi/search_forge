use egui::{Color32, Frame, Margin, Pos2, Rect, Rounding, Stroke, Ui, Vec2};
use crate::utils::config::AppConfig;

/// Modern iOS/macOS-style pill toggle switch with smooth animated visual states
pub fn render_toggle_switch(ui: &mut Ui, on: &mut bool) -> bool {
    let desired_size = Vec2::new(34.0, 18.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

    let changed = if response.clicked() {
        *on = !*on;
        true
    } else {
        false
    };

    let bg_color = if *on {
        Color32::from_rgb(255, 230, 0) // SearchForge signature yellow
    } else if response.hovered() {
        Color32::from_rgb(48, 52, 66)
    } else {
        Color32::from_rgb(34, 38, 48)
    };

    // Draw pill track
    let radius = rect.height() / 2.0;
    ui.painter().rect_filled(rect, Rounding::same(radius), bg_color);

    // Draw switch knob
    let knob_radius = radius - 2.0;
    let knob_center_x = if *on {
        rect.max.x - radius
    } else {
        rect.min.x + radius
    };
    let knob_color = if *on {
        Color32::BLACK
    } else {
        Color32::from_rgb(200, 205, 220)
    };
    ui.painter().circle_filled(
        Pos2::new(knob_center_x, rect.center().y),
        knob_radius,
        knob_color,
    );

    changed
}

pub fn render_settings_menu(
    ui: &mut Ui,
    open: &mut bool,
    config: &mut AppConfig,
    button_rect: Rect,
) {
    if !*open {
        return;
    }

    let menu_width = 280.0;
    let menu_height = 224.0;

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
                                .size(13.5)
                                .strong()
                                .color(Color32::WHITE),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let close_btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new("×")
                                        .size(15.0)
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

                    // 1. Primary Toggle: Autostart on Boot
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("Start on Login")
                                    .size(12.5)
                                    .strong()
                                    .color(Color32::WHITE),
                            );
                            ui.label(
                                egui::RichText::new("Launch SearchForge automatically")
                                    .size(11.0)
                                    .color(Color32::from_rgb(130, 135, 150)),
                            );
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let mut autostart_enabled = config.autostart;
                            if render_toggle_switch(ui, &mut autostart_enabled) {
                                config.set_autostart(autostart_enabled);
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // 2. Appearance Theme: Translucent Glass vs Classic Dark
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("Theme Appearance")
                                    .size(12.5)
                                    .strong()
                                    .color(Color32::WHITE),
                            );
                            ui.label(
                                egui::RichText::new(match config.theme {
                                    crate::utils::config::ThemeMode::Glass => "Translucent Acrylic Glass",
                                    crate::utils::config::ThemeMode::Dark => "Classic Dark Solid",
                                })
                                .size(11.0)
                                .color(Color32::from_rgb(130, 135, 150)),
                            );
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let mut is_glass = config.theme == crate::utils::config::ThemeMode::Glass;
                            if render_toggle_switch(ui, &mut is_glass) {
                                let new_theme = if is_glass {
                                    crate::utils::config::ThemeMode::Glass
                                } else {
                                    crate::utils::config::ThemeMode::Dark
                                };
                                config.set_theme(new_theme);
                                crate::utils::window_effects::apply_native_glass_to_window(is_glass);
                            }
                        });
                    });

                    ui.add_space(10.0);

                    // Shortcut prompt info
                    Frame::none()
                        .fill(Color32::from_rgb(15, 16, 21))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(32, 36, 46)))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::symmetric(8.0, 5.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Spotlight Shortcut:")
                                        .size(11.0)
                                        .color(Color32::from_rgb(140, 145, 160)),
                                );
                                ui.label(
                                    egui::RichText::new("Alt + K")
                                        .size(11.0)
                                        .strong()
                                        .color(Color32::from_rgb(255, 230, 0)),
                                );
                            });
                        });

                    ui.add_space(10.0);

                    // Quit SearchForge completely button
                    ui.horizontal(|ui| {
                        let quit_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new("Quit SearchForge")
                                    .size(11.5)
                                    .color(Color32::from_rgb(240, 110, 110)),
                            )
                            .fill(Color32::from_rgb(34, 22, 24))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(60, 32, 36)))
                            .rounding(Rounding::same(4.0))
                            .min_size(Vec2::new(110.0, 24.0)),
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
