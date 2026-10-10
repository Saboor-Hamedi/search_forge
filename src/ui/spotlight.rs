use crate::indexer::icon_cache::IconCache;
use crate::indexer::scanner::FileRecord;
use crate::ui::results_list::{app_initials, file_badge_info};
use crate::utils::unicode::format_size;
use egui::{Color32, Frame, Margin, Pos2, Rect, Rounding, Stroke, Ui, Vec2};
use std::path::Path;

pub fn render_spotlight_search_bar(
    ui: &mut Ui,
    query: &mut String,
    top_suggestion: Option<&str>,
    _is_glass: bool,
    open_full_requested: &mut bool,
) {
    let input_id = egui::Id::new("spotlight_search_text_edit");

    // Clean suggestion autocompletion suffix
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

    let bar_fill = Color32::from_rgb(20, 22, 28);

    Frame::none()
        .fill(bar_fill)
        .inner_margin(Margin::symmetric(16.0, 14.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;

                // 1. Vector Search Icon with SearchForge yellow accent
                crate::ui::icons::draw_search_icon(ui, 18.0, Color32::from_rgb(255, 230, 0));

                // Right controls width (Expand to Full button + Dismiss hint)
                let right_w = 110.0;
                let text_w = (ui.available_width() - right_w).max(160.0);

                // 2. Spotlight Search Input: Large 18px font, borderless
                let output = egui::TextEdit::singleline(query)
                    .id(input_id)
                    .hint_text("Search files, software, code...")
                    .font(egui::FontId::proportional(18.0))
                    .text_color(Color32::WHITE)
                    .frame(false)
                    .desired_width(text_w)
                    .show(ui);

                let response = output.response;

                // Tab auto-completion
                if completion_suffix.is_some() {
                    if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)) {
                        if let Some(suggestion) = top_suggestion {
                            *query = suggestion.to_string();
                            let char_len = query.chars().count();
                            let cursor = egui::text::CCursor::new(char_len);
                            let mut state =
                                egui::text_edit::TextEditState::load(ui.ctx(), input_id).unwrap_or_default();
                            state.cursor.set_char_range(Some(egui::text::CCursorRange::one(cursor)));
                            state.store(ui.ctx(), input_id);

                            response.request_focus();
                            ui.ctx().memory_mut(|m| m.request_focus(input_id));
                        }
                    }
                }

                // Ghost auto-completion display
                if let Some(ref suffix) = completion_suffix {
                    let font_id = egui::FontId::proportional(18.0);
                    let galley = output.galley;
                    let ghost_x = output.galley_pos.x + galley.size().x + 2.0;
                    let ghost_pos = Pos2::new(ghost_x, output.galley_pos.y);

                    ui.painter().text(
                        ghost_pos,
                        egui::Align2::LEFT_TOP,
                        suffix,
                        font_id,
                        Color32::from_rgb(115, 120, 138),
                    );
                }

                // Ensure Spotlight input receives immediate, reliable focus
                if !response.has_focus() {
                    response.request_focus();
                }

                // Right side: Subtle Full-mode icon button + escape pill
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;

                    // Close button (×) to dismiss Spotlight
                    let (close_rect, close_resp) = ui.allocate_exact_size(Vec2::new(24.0, 24.0), egui::Sense::click());
                    let close_resp = close_resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                    let close_bg = if close_resp.hovered() {
                        Color32::from_rgb(200, 55, 55)
                    } else {
                        Color32::TRANSPARENT
                    };
                    if close_bg != Color32::TRANSPARENT {
                        ui.painter().rect_filled(close_rect, Rounding::same(4.0), close_bg);
                    }
                    let pad = 7.0;
                    let stroke_col = if close_resp.hovered() { Color32::WHITE } else { Color32::from_rgb(140, 145, 160) };
                    ui.painter().line_segment(
                        [Pos2::new(close_rect.min.x + pad, close_rect.min.y + pad), Pos2::new(close_rect.max.x - pad, close_rect.max.y - pad)],
                        Stroke::new(1.3, stroke_col),
                    );
                    ui.painter().line_segment(
                        [Pos2::new(close_rect.min.x + pad, close_rect.max.y - pad), Pos2::new(close_rect.max.x - pad, close_rect.min.y + pad)],
                        Stroke::new(1.3, stroke_col),
                    );
                    if close_resp.on_hover_text("Dismiss Spotlight (Esc)").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Visible(false));
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        #[cfg(target_os = "windows")]
                        unsafe {
                            crate::utils::tray_hotkey::hide_searchforge();
                        }
                    }

                    // Expand to Full SearchForge window button with crisp vector icon
                    let (btn_rect, expand_resp) = ui.allocate_exact_size(Vec2::new(56.0, 24.0), egui::Sense::click());
                    let expand_resp = expand_resp.on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text("Open Full SearchForge with file previews (Ctrl+Shift+E)");
                    let btn_fill = if expand_resp.hovered() {
                        Color32::from_rgb(40, 44, 58)
                    } else {
                        Color32::from_rgb(30, 33, 44)
                    };
                    ui.painter().rect_filled(btn_rect, Rounding::same(4.0), btn_fill);

                    // Draw expand icon
                    let icon_rect = Rect::from_center_size(
                        Pos2::new(btn_rect.min.x + 12.0, btn_rect.center().y),
                        Vec2::splat(10.0),
                    );
                    let icon_col = if expand_resp.hovered() { Color32::WHITE } else { Color32::from_rgb(170, 175, 190) };
                    crate::ui::icons::draw_expand_icon(ui.painter(), icon_rect, Stroke::new(1.2, icon_col));

                    // Text
                    ui.painter().text(
                        Pos2::new(btn_rect.min.x + 24.0, btn_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        "Full",
                        egui::FontId::proportional(11.5),
                        icon_col,
                    );

                    if expand_resp.clicked() {
                        *open_full_requested = true;
                    }
                });
            });
        });
}

pub fn render_spotlight_results(
    ui: &mut Ui,
    results: &[FileRecord],
    selected_index: &mut Option<usize>,
    icon_cache: &IconCache,
    keyboard_navigated: bool,
    _is_glass: bool,
) {
    if results.is_empty() {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("No files or software found")
                    .color(Color32::from_rgb(180, 185, 200))
                    .size(13.0),
            );
            ui.add_space(3.0);
            ui.label(
                egui::RichText::new("Press Esc to dismiss · Type to search")
                    .color(Color32::from_rgb(110, 115, 130))
                    .size(11.0),
            );
        });
        ui.add_space(16.0);
        return;
    }

    egui::ScrollArea::vertical()
        .id_source("spotlight_results_scroll")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(6.0);
            let display_count = results.len().min(8);

            for (idx, record) in results.iter().take(display_count).enumerate() {
                let is_selected = *selected_index == Some(idx);
                let is_app = record.is_app();

                let row_id = ui.make_persistent_id(format!("spotlight_row_{}", idx));
                let was_hovered = ui.data(|d| d.get_temp::<bool>(row_id).unwrap_or(false));

                let bg_color = if is_selected {
                    Color32::from_rgb(34, 38, 50)
                } else if was_hovered {
                    Color32::from_rgb(26, 29, 38)
                } else {
                    Color32::TRANSPARENT
                };

                let row_frame = Frame::none()
                    .fill(bg_color)
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::symmetric(10.0, 7.0));

                let response = row_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Signature yellow selection indicator bar
                        let bar_color = if is_selected {
                            Color32::from_rgb(255, 230, 0)
                        } else {
                            Color32::TRANSPARENT
                        };
                        let (bar_rect, _) =
                            ui.allocate_exact_size(Vec2::new(3.0, 26.0), egui::Sense::hover());
                        ui.painter().rect_filled(bar_rect, Rounding::same(1.5), bar_color);
                        ui.add_space(6.0);

                        if is_app {
                            let icon_key = record.icon_cache_key();
                            let texture_opt =
                                icon_cache.get_or_load_texture(ui.ctx(), &icon_key, Some(&record.path));

                            if let Some(texture) = texture_opt {
                                ui.add(
                                    egui::Image::new(&texture)
                                        .fit_to_exact_size(Vec2::new(26.0, 26.0))
                                        .rounding(Rounding::same(4.0)),
                                );
                            } else {
                                let (icon_rect, _) =
                                    ui.allocate_exact_size(Vec2::new(26.0, 26.0), egui::Sense::hover());
                                ui.painter().rect_filled(
                                    icon_rect,
                                    Rounding::same(5.0),
                                    Color32::from_rgb(26, 30, 42),
                                );
                                ui.painter().rect_stroke(
                                    icon_rect,
                                    Rounding::same(5.0),
                                    Stroke::new(1.0, Color32::from_rgb(45, 55, 78)),
                                );
                                let initials = app_initials(record.display_name());
                                ui.painter().text(
                                    icon_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    initials,
                                    egui::FontId::proportional(10.5),
                                    Color32::from_rgb(0, 195, 240),
                                );
                            }

                            ui.add_space(8.0);

                            // App name and category
                            let app_w = (ui.available_width() - 80.0).max(100.0);
                            ui.allocate_ui_with_layout(
                                Vec2::new(app_w, 28.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(app_w);
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(record.display_name())
                                                .size(13.5)
                                                .strong()
                                                .color(if is_selected {
                                                    Color32::WHITE
                                                } else {
                                                    Color32::from_rgb(225, 230, 240)
                                                }),
                                        )
                                        .truncate(true),
                                    );

                                    let subtitle = if let Some(pub_name) = record.publisher() {
                                        format!("{} · Application", pub_name)
                                    } else {
                                        "Application".to_string()
                                    };
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(subtitle)
                                                .size(11.0)
                                                .color(if is_selected {
                                                    Color32::from_rgb(160, 165, 185)
                                                } else {
                                                    Color32::from_rgb(125, 130, 145)
                                                }),
                                        )
                                        .truncate(true),
                                    );
                                },
                            );

                            // Subtle enter/action badge
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if is_selected {
                                    let (badge_rect, _) = ui.allocate_exact_size(Vec2::new(58.0, 19.0), egui::Sense::hover());
                                    ui.painter().rect_filled(badge_rect, Rounding::same(3.5), Color32::from_rgb(36, 40, 54));
                                    ui.painter().rect_stroke(badge_rect, Rounding::same(3.5), Stroke::new(1.0, Color32::from_rgb(56, 62, 82)));

                                    let arrow_rect = Rect::from_center_size(
                                        Pos2::new(badge_rect.min.x + 9.0, badge_rect.center().y),
                                        Vec2::splat(9.0),
                                    );
                                    crate::ui::icons::draw_return_key_icon(ui.painter(), arrow_rect, Stroke::new(1.2, Color32::from_rgb(220, 225, 238)));

                                    ui.painter().text(
                                        Pos2::new(badge_rect.min.x + 19.0, badge_rect.center().y),
                                        egui::Align2::LEFT_CENTER,
                                        "Launch",
                                        egui::FontId::proportional(10.0),
                                        Color32::from_rgb(220, 225, 238),
                                    );
                                } else {
                                    Frame::none()
                                        .fill(Color32::from_rgba_premultiplied(24, 28, 38, 120))
                                        .rounding(Rounding::same(3.0))
                                        .inner_margin(Margin::symmetric(5.0, 2.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new("APP")
                                                    .size(9.5)
                                                    .color(Color32::from_rgb(0, 195, 240)),
                                            );
                                        });
                                }
                            });
                        } else {
                            // File / Folder row
                            let (badge_text, badge_color) = file_badge_info(&record.name, record.is_dir);

                            Frame::none()
                                .fill(badge_color.linear_multiply(0.18))
                                .rounding(Rounding::same(3.0))
                                .inner_margin(Margin::symmetric(5.0, 2.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(badge_text)
                                            .size(10.0)
                                            .strong()
                                            .color(badge_color),
                                    );
                                });

                            ui.add_space(8.0);

                            let size_pill_w = 60.0;
                            let text_w = (ui.available_width() - size_pill_w - 10.0).max(80.0);

                            ui.allocate_ui_with_layout(
                                Vec2::new(text_w, 28.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(text_w);
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(&record.name)
                                                .size(13.0)
                                                .strong()
                                                .color(if is_selected {
                                                    Color32::WHITE
                                                } else {
                                                    Color32::from_rgb(220, 225, 235)
                                                }),
                                        )
                                        .truncate(true),
                                    );

                                    let parent = Path::new(&record.path)
                                        .parent()
                                        .and_then(|p| p.to_str())
                                        .unwrap_or("");
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(parent)
                                                .size(10.5)
                                                .color(if is_selected {
                                                    Color32::from_rgb(155, 160, 180)
                                                } else {
                                                    Color32::from_rgb(125, 130, 145)
                                                }),
                                        )
                                        .truncate(true),
                                    );
                                },
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if is_selected {
                                    let (badge_rect, _) = ui.allocate_exact_size(Vec2::new(52.0, 19.0), egui::Sense::hover());
                                    ui.painter().rect_filled(badge_rect, Rounding::same(3.5), Color32::from_rgb(36, 40, 54));
                                    ui.painter().rect_stroke(badge_rect, Rounding::same(3.5), Stroke::new(1.0, Color32::from_rgb(56, 62, 82)));

                                    let arrow_rect = Rect::from_center_size(
                                        Pos2::new(badge_rect.min.x + 9.0, badge_rect.center().y),
                                        Vec2::splat(9.0),
                                    );
                                    crate::ui::icons::draw_return_key_icon(ui.painter(), arrow_rect, Stroke::new(1.2, Color32::from_rgb(220, 225, 238)));

                                    ui.painter().text(
                                        Pos2::new(badge_rect.min.x + 19.0, badge_rect.center().y),
                                        egui::Align2::LEFT_CENTER,
                                        "Open",
                                        egui::FontId::proportional(10.0),
                                        Color32::from_rgb(220, 225, 238),
                                    );
                                } else {
                                    let size_str = if record.is_dir {
                                        "DIR".to_string()
                                    } else {
                                        format_size(record.size)
                                    };
                                    Frame::none()
                                        .fill(Color32::from_rgba_premultiplied(24, 28, 38, 120))
                                        .rounding(Rounding::same(3.0))
                                        .inner_margin(Margin::symmetric(5.0, 2.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new(size_str)
                                                    .size(10.0)
                                                    .color(Color32::from_rgb(135, 140, 155)),
                                            );
                                        });
                                }
                            });
                        }
                    });
                });

                let interactive = response.response.interact(egui::Sense::click());
                let interactive = interactive.on_hover_cursor(egui::CursorIcon::PointingHand);
                let is_now_hovered = interactive.hovered();
                ui.data_mut(|d| d.insert_temp(row_id, is_now_hovered));

                if interactive.clicked() {
                    *selected_index = Some(idx);
                }

                if is_selected && keyboard_navigated {
                    response.response.scroll_to_me(Some(egui::Align::Center));
                }

                ui.add_space(4.0);
            }
        });
}
