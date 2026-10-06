use crate::indexer::scanner::FileRecord;
use crate::utils::unicode::format_size;
use egui::{Color32, Frame, Margin, Rounding, Ui};
use std::path::Path;

pub fn file_badge_info(name: &str, is_dir: bool) -> (&'static str, Color32) {
    if is_dir {
        return ("DIR", Color32::from_rgb(235, 175, 45));
    }
    let ext = Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "lnk" => ("APP", Color32::from_rgb(0, 195, 240)),
        "md" | "markdown" => ("MD", Color32::from_rgb(180, 140, 240)),
        "txt" | "rtf" | "log" => ("TXT", Color32::from_rgb(160, 165, 180)),
        "pdf" => ("PDF", Color32::from_rgb(240, 75, 75)),
        "xlsx" | "xls" | "ods" => ("XLS", Color32::from_rgb(40, 180, 100)),
        "csv" | "tsv" => ("CSV", Color32::from_rgb(35, 165, 120)),
        "docx" | "doc" => ("DOC", Color32::from_rgb(50, 140, 245)),
        "json" => ("JSON", Color32::from_rgb(235, 140, 50)),
        "toml" | "yaml" | "yml" | "xml" | "ini" | "env" => ("CFG", Color32::from_rgb(220, 130, 60)),
        "rs" => ("RS", Color32::from_rgb(230, 90, 40)),
        "py" => ("PY", Color32::from_rgb(55, 135, 230)),
        "js" | "jsx" => ("JS", Color32::from_rgb(240, 210, 50)),
        "ts" | "tsx" => ("TS", Color32::from_rgb(45, 130, 225)),
        "html" | "htm" => ("HTML", Color32::from_rgb(235, 95, 45)),
        "css" | "scss" => ("CSS", Color32::from_rgb(60, 150, 240)),
        "c" | "cpp" | "h" | "hpp" | "cs" | "go" | "java" => ("DEV", Color32::from_rgb(0, 160, 235)),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" => ("IMG", Color32::from_rgb(45, 195, 185)),
        "zip" | "tar" | "gz" | "7z" | "rar" => ("ZIP", Color32::from_rgb(210, 170, 50)),
        _ => ("FILE", Color32::from_rgb(140, 145, 160)),
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let mut truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        truncated.push_str("...");
        truncated
    }
}

pub fn render_results_list(
    ui: &mut Ui,
    results: &[FileRecord],
    selected_index: &mut Option<usize>,
) {
    if results.is_empty() {
        let total_h = ui.available_height();
        let content_approx_h = 90.0;
        let top_pad = ((total_h - content_approx_h) / 2.0).max(20.0);

        ui.add_space(top_pad);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("🔍")
                    .size(22.0)
                    .color(Color32::from_rgb(85, 90, 105)),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("No files found")
                    .color(Color32::from_rgb(205, 210, 220))
                    .size(13.5)
                    .strong(),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Type to search files and installed apps")
                    .color(Color32::from_rgb(115, 120, 135))
                    .size(11.0),
            );
        });
        return;
    }

    ui.add_space(4.0);
    // Display maximum of 5 results as requested
    for (idx, record) in results.iter().take(5).enumerate() {
        let is_selected = *selected_index == Some(idx);
        let (badge_text, badge_color) = file_badge_info(&record.name, record.is_dir);

        let row_id = ui.make_persistent_id(format!("result_row_{}", idx));
        let was_hovered = ui.data(|d| d.get_temp::<bool>(row_id).unwrap_or(false));

        let bg_color = if is_selected {
            Color32::from_rgb(32, 35, 46)
        } else if was_hovered {
            Color32::from_rgb(24, 26, 33) // Almost imperceptible surface lightening
        } else {
            Color32::TRANSPARENT
        };

        let row_frame = Frame::none()
            .fill(bg_color)
            .rounding(Rounding::same(5.0))
            .inner_margin(Margin::symmetric(8.0, 7.0));

        let response = row_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                // Subtle selection indicator bar
                let bar_color = if is_selected {
                    Color32::from_rgb(255, 230, 0)
                } else {
                    Color32::TRANSPARENT
                };
                let (bar_rect, _) = ui.allocate_exact_size(egui::Vec2::new(3.0, 24.0), egui::Sense::hover());
                ui.painter().rect_filled(bar_rect, Rounding::same(1.5), bar_color);
                ui.add_space(4.0);

                // Clean typography badge
                Frame::none()
                    .fill(badge_color.linear_multiply(0.2))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::symmetric(6.0, 3.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(badge_text)
                                .size(10.5)
                                .strong()
                                .color(badge_color),
                        );
                    });

                ui.add_space(8.0);

                // File name & breadcrumb path
                ui.vertical(|ui| {
                    let display_name = if record.name.to_lowercase().ends_with(".lnk") {
                        record.name[..record.name.len().saturating_sub(4)].to_string()
                    } else {
                        record.name.clone()
                    };
                    let truncated_name = truncate_string(&display_name, 34);
                    ui.label(
                        egui::RichText::new(truncated_name)
                            .size(13.0)
                            .strong()
                            .color(if is_selected {
                                Color32::WHITE
                            } else {
                                Color32::from_rgb(220, 225, 235)
                            }),
                    );

                    let parent = Path::new(&record.path)
                        .parent()
                        .and_then(|p| p.to_str())
                        .unwrap_or("");
                    let truncated_parent = truncate_string(parent, 36);
                    ui.label(
                        egui::RichText::new(truncated_parent)
                            .size(11.0)
                            .color(if is_selected {
                                Color32::from_rgb(150, 155, 175)
                            } else {
                                Color32::from_rgb(130, 135, 150)
                            }),
                    );
                });

                // Right side size indicator pill
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let size_text = if record.is_dir {
                        "DIR".to_string()
                    } else if record.name.to_lowercase().ends_with(".lnk") {
                        "APP".to_string()
                    } else {
                        format_size(record.size)
                    };

                    Frame::none()
                        .fill(Color32::from_rgb(28, 30, 38))
                        .rounding(Rounding::same(3.0))
                        .inner_margin(Margin::symmetric(5.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(size_text)
                                .size(10.5)
                                .color(Color32::from_rgb(140, 145, 160)),
                            );
                        });
                });
            });
        });

        let interactive = response.response.interact(egui::Sense::click());
        let is_now_hovered = interactive.hovered();
        ui.data_mut(|d| d.insert_temp(row_id, is_now_hovered));

        // Click selection
        if interactive.clicked() {
            *selected_index = Some(idx);
        }

        ui.add_space(5.0);
    }
}


