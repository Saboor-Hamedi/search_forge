use crate::indexer::scanner::FileRecord;
use crate::ui::results_list::file_badge_info;
use crate::utils::unicode::format_size;
use egui::{Color32, Frame, Margin, Rounding, Stroke, Ui, Vec2};
use std::fs;
use std::io::Read;
use std::path::Path;

fn truncate_preview_string(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let mut truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        truncated.push_str("...");
        truncated
    }
}

pub fn render_preview_panel(
    ui: &mut Ui,
    selected_file: Option<&FileRecord>,
    pdf_renderer: &mut crate::ui::pdf_renderer::PdfRenderer,
) {
    let Some(file) = selected_file else {
        let total_h = ui.available_height();
        let content_approx_h = 90.0;
        let top_pad = ((total_h - content_approx_h) / 2.0).max(20.0);

        ui.add_space(top_pad);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("⚡")
                    .size(22.0)
                    .color(Color32::from_rgb(90, 115, 175)),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Select a file to preview")
                    .size(13.5)
                    .strong()
                    .color(Color32::from_rgb(205, 210, 220)),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Documents, Markdown, Spreadsheets, Code, PDF & Images")
                    .size(11.0)
                    .color(Color32::from_rgb(115, 120, 135)),
            );
        });
        return;
    };

    // Top metadata header (Clean, quiet, untangled, borderless)
    let (badge_text, badge_color) = file_badge_info(&file.name, file.is_dir);
    let is_app = file.name.to_lowercase().ends_with(".lnk");
    let display_name = if is_app {
        file.name[..file.name.len().saturating_sub(4)].to_string()
    } else {
        file.name.clone()
    };

    ui.horizontal(|ui| {
        // Clean typography badge
        Frame::none()
            .fill(badge_color.linear_multiply(0.2))
            .rounding(Rounding::same(4.0))
            .inner_margin(Margin::symmetric(7.0, 4.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(badge_text)
                        .size(11.0)
                        .strong()
                        .color(badge_color),
                );
            });
        ui.add_space(8.0);

        ui.vertical(|ui| {
            let truncated_name = truncate_preview_string(&display_name, 44);
            ui.label(
                egui::RichText::new(truncated_name)
                    .size(14.0)
                    .strong()
                    .color(Color32::WHITE),
            );
            let truncated_path = truncate_preview_string(&file.path, 48);
            ui.label(
                egui::RichText::new(truncated_path)
                    .size(11.0)
                    .color(Color32::from_rgb(130, 135, 150)),
            );
        });

        // Sleek, fully round "Open" / "Launch" button on the right side of preview header - does NOT block content
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let action_label = if is_app { "Launch ↗" } else { "Open ↗" };
            let (bg_fill, border_stroke, text_color) = if is_app {
                (Color32::from_rgb(0, 190, 240), Stroke::NONE, Color32::BLACK)
            } else {
                (Color32::from_rgb(32, 35, 46), Stroke::new(1.0, Color32::from_rgb(52, 56, 70)), Color32::from_rgb(220, 225, 235))
            };

            let open_btn = egui::Button::new(
                egui::RichText::new(action_label)
                    .size(11.5)
                    .strong()
                    .color(text_color),
            )
            .fill(bg_fill)
            .stroke(border_stroke)
            .rounding(Rounding::same(14.0)) // fully round pill button
            .min_size(Vec2::new(72.0, 26.0));

            if ui
                .add(open_btn)
                .on_hover_text(if is_app { "Launch application" } else { "Open with default system program" })
                .clicked()
            {
                let _ = open::that(&file.path);
            }
        });
    });

    ui.add_space(8.0);

    // Main Preview Content Container (Clean, quiet, never blocked by any bottom button)
    Frame::none()
        .fill(Color32::from_rgb(20, 22, 28))
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            if file.is_dir {
                render_directory_preview(ui, &file.path);
                return;
            }

            let ext = Path::new(&file.path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            match ext.as_str() {
                "lnk" => render_app_preview(ui, file),
                "md" | "markdown" => render_markdown_preview(ui, &file.path),
                "xlsx" | "xls" | "ods" => render_excel_preview(ui, &file.path),
                "csv" | "tsv" => render_csv_preview(ui, &file.path, ext == "tsv"),
                "docx" => render_docx_preview(ui, &file.path),
                "pdf" => render_pdf_preview(ui, &file.path, pdf_renderer),
                "json" => render_json_preview(ui, &file.path),
                "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" => {
                    render_image_preview(ui, &file.path)
                }
                _ => render_code_or_text_preview(ui, &file.path),
            }
        });
}

fn render_directory_preview(ui: &mut Ui, path: &str) {
    ui.heading("📁 Directory Contents");
    ui.add_space(6.0);

    let entries = match fs::read_dir(path) {
        Ok(read_dir) => read_dir.filter_map(|e| e.ok()).collect::<Vec<_>>(),
        Err(err) => {
            ui.label(format!("Could not read folder: {}", err));
            return;
        }
    };

    ui.label(
        egui::RichText::new(format!("{} item(s) in folder", entries.len()))
            .size(12.0)
            .color(Color32::from_rgb(140, 145, 160)),
    );
    ui.add_space(6.0);

    egui::ScrollArea::vertical()
        .id_source("folder_preview_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for entry in entries.iter().take(200) {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let (icon, color) = file_badge_info(&name, is_dir);
                let size_str = entry
                    .metadata()
                    .map(|m| format_size(m.len()))
                    .unwrap_or_default();

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(icon).color(color));
                    ui.label(egui::RichText::new(&name).color(Color32::from_rgb(220, 225, 235)));
                    if !is_dir {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(size_str)
                                    .size(11.0)
                                    .color(Color32::from_rgb(120, 125, 140)),
                            );
                        });
                    }
                });
                ui.add_space(2.0);
            }
        });
}

fn render_app_preview(ui: &mut Ui, file: &FileRecord) {
    let app_name = if file.name.to_lowercase().ends_with(".lnk") {
        file.name[..file.name.len().saturating_sub(4)].to_string()
    } else {
        file.name.clone()
    };

    ui.vertical_centered(|ui| {
        ui.add_space(20.0);

        // App Icon anchor (clean, subtle, no giant banner)
        ui.label(
            egui::RichText::new("🚀")
                .size(32.0),
        );

        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(&app_name)
                .size(17.0)
                .strong()
                .color(Color32::WHITE),
        );

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Windows desktop application")
                .size(11.5)
                .color(Color32::from_rgb(140, 145, 160)),
        );

        ui.add_space(24.0);
    });

    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new("Location")
                    .size(11.0)
                    .strong()
                    .color(Color32::from_rgb(160, 165, 180)),
            );
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(&file.path)
                    .size(11.0)
                    .color(Color32::from_rgb(120, 125, 140)),
            );
        });
    });
}

fn render_markdown_inline(ui: &mut Ui, text: &str, base_size: f32, base_color: Color32) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '`' {
                // Inline code snippet
                let mut code = String::new();
                while let Some(&nc) = chars.peek() {
                    chars.next();
                    if nc == '`' {
                        break;
                    }
                    code.push(nc);
                }
                if !code.is_empty() {
                    Frame::none()
                        .fill(Color32::from_rgb(34, 38, 48))
                        .rounding(Rounding::same(3.0))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(55, 60, 75)))
                        .inner_margin(Margin::symmetric(4.0, 1.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(&code)
                                    .monospace()
                                    .size(base_size - 1.0)
                                    .color(Color32::from_rgb(255, 230, 0)),
                            );
                        });
                }
            } else if c == '*' || c == '_' {
                // Check if double (** or __)
                let is_double = chars.peek() == Some(&c);
                if is_double {
                    chars.next(); // consume second marker
                    let mut bold_text = String::new();
                    let mut closed = false;
                    while let Some(nc) = chars.next() {
                        if nc == c && chars.peek() == Some(&c) {
                            chars.next();
                            closed = true;
                            break;
                        }
                        bold_text.push(nc);
                    }
                    if closed || !bold_text.is_empty() {
                        ui.label(
                            egui::RichText::new(&bold_text)
                                .size(base_size)
                                .strong()
                                .color(Color32::WHITE),
                        );
                    }
                } else {
                    // Single (* or _) -> italics
                    let mut italic_text = String::new();
                    let mut closed = false;
                    while let Some(nc) = chars.next() {
                        if nc == c {
                            closed = true;
                            break;
                        }
                        italic_text.push(nc);
                    }
                    if closed || !italic_text.is_empty() {
                        ui.label(
                            egui::RichText::new(&italic_text)
                                .size(base_size)
                                .italics()
                                .color(base_color),
                        );
                    }
                }
            } else if c == '[' {
                // Markdown link [text](url) -> display clean link text
                let mut link_text = String::new();
                let mut has_close_bracket = false;
                while let Some(nc) = chars.next() {
                    if nc == ']' {
                        has_close_bracket = true;
                        break;
                    }
                    link_text.push(nc);
                }
                if has_close_bracket && chars.peek() == Some(&'(') {
                    chars.next(); // '('
                    let mut url = String::new();
                    while let Some(nc) = chars.next() {
                        if nc == ')' {
                            break;
                        }
                        url.push(nc);
                    }
                    if ui.link(egui::RichText::new(&link_text).size(base_size).color(Color32::from_rgb(0, 180, 255))).clicked() {
                        let _ = open::that(&url);
                    }
                } else {
                    let mut s = String::new();
                    s.push('[');
                    s.push_str(&link_text);
                    if has_close_bracket {
                        s.push(']');
                    }
                    ui.label(egui::RichText::new(s).size(base_size).color(base_color));
                }
            } else if c.is_whitespace() {
                // Let spaces naturally space items or wrap
                ui.add_space(3.0);
            } else {
                // Plain word / text accumulator
                let mut word = String::new();
                word.push(c);
                while let Some(&nc) = chars.peek() {
                    if nc == '`' || nc == '*' || nc == '_' || nc == '[' || nc.is_whitespace() {
                        break;
                    }
                    chars.next();
                    word.push(nc);
                }
                ui.label(egui::RichText::new(word).size(base_size).color(base_color));
            }
        }
    });
}

fn render_markdown_preview(ui: &mut Ui, path: &str) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            ui.label(format!("Could not read markdown: {}", e));
            return;
        }
    };

    let max_w = ui.available_width();
    egui::ScrollArea::vertical()
        .id_source("markdown_preview_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_max_width(max_w);

            let mut in_code_block = false;
            let mut code_lang = String::new();
            let mut code_accumulator = String::new();

            let lines: Vec<&str> = content.lines().collect();
            let mut line_idx = 0;

            while line_idx < lines.len() {
                let line = lines[line_idx];
                let trimmed = line.trim();

                // Check for Markdown table row (| col1 | col2 |)
                if !in_code_block && trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.len() > 1 {
                    // Collect all consecutive table rows
                    let mut current_table: Vec<Vec<String>> = Vec::new();
                    while line_idx < lines.len() {
                        let t_line = lines[line_idx].trim();
                        if t_line.starts_with('|') && t_line.ends_with('|') && t_line.len() > 1 {
                            // Check if this is a separator row like |---|---|
                            let is_separator = t_line
                                .trim_matches('|')
                                .split('|')
                                .all(|cell| cell.trim().chars().all(|c| c == '-' || c == ':' || c.is_whitespace()));

                            if !is_separator {
                                let cells: Vec<String> = t_line
                                    .trim_matches('|')
                                    .split('|')
                                    .map(|c| c.trim().to_string())
                                    .collect();
                                current_table.push(cells);
                            }
                            line_idx += 1;
                        } else {
                            break;
                        }
                    }

                    if !current_table.is_empty() {
                        ui.add_space(6.0);
                        render_markdown_table(ui, &current_table);
                        ui.add_space(6.0);
                    }
                    continue;
                }

                if trimmed.starts_with("```") {
                    if in_code_block {
                        // Render flat modern code wrapper with Copy button
                        render_flat_code_wrapper(ui, &code_lang, &code_accumulator);
                        code_accumulator.clear();
                        code_lang.clear();
                        in_code_block = false;
                    } else {
                        in_code_block = true;
                        code_lang = trimmed.strip_prefix("```").unwrap_or("").trim().to_string();
                    }
                    line_idx += 1;
                    continue;
                }

                if in_code_block {
                    code_accumulator.push_str(line);
                    code_accumulator.push('\n');
                    line_idx += 1;
                    continue;
                }

                if let Some(h1) = trimmed.strip_prefix("# ") {
                    ui.add_space(8.0);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(h1.trim()).size(20.0).strong().color(Color32::WHITE),
                        )
                        .wrap(true),
                    );
                    ui.separator();
                } else if let Some(h2) = trimmed.strip_prefix("## ") {
                    ui.add_space(6.0);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(h2.trim())
                                .size(16.0)
                                .strong()
                                .color(Color32::from_rgb(230, 235, 245)),
                        )
                        .wrap(true),
                    );
                } else if let Some(h3) = trimmed.strip_prefix("### ") {
                    ui.add_space(4.0);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(h3.trim())
                                .size(14.0)
                                .strong()
                                .color(Color32::from_rgb(180, 200, 235)),
                        )
                        .wrap(true),
                    );
                } else if let Some(bullet) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        ui.label(egui::RichText::new("•").size(14.0).strong().color(Color32::from_rgb(0, 180, 255)));
                        render_markdown_inline(ui, bullet.trim(), 13.0, Color32::from_rgb(220, 225, 235));
                    });
                } else if let Some(quote) = trimmed.strip_prefix("> ") {
                    Frame::none()
                        .fill(Color32::from_rgb(28, 32, 40))
                        .stroke(Stroke::new(1.5, Color32::from_rgb(0, 180, 255)))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::symmetric(10.0, 6.0))
                        .show(ui, |ui| {
                            ui.set_max_width(ui.available_width());
                            render_markdown_inline(ui, quote.trim(), 13.0, Color32::from_rgb(190, 205, 230));
                        });
                } else if trimmed == "---" || trimmed == "***" || trimmed == "___" {
                    ui.separator();
                } else if !trimmed.is_empty() {
                    render_markdown_inline(ui, trimmed, 13.0, Color32::from_rgb(215, 220, 230));
                    ui.add_space(2.0);
                }

                line_idx += 1;
            }

            if in_code_block && !code_accumulator.is_empty() {
                render_flat_code_wrapper(ui, &code_lang, &code_accumulator);
            }
        });
}

fn render_flat_code_wrapper(ui: &mut Ui, lang: &str, code: &str) {
    let clean_code = code.trim_end();
    let display_lang = if lang.is_empty() { "code" } else { lang };

    // Flat sleek code container
    Frame::none()
        .fill(Color32::from_rgb(18, 20, 25))
        .stroke(Stroke::new(1.0, Color32::from_rgb(38, 42, 54)))
        .rounding(Rounding::same(4.0))
        .inner_margin(Margin::same(0.0))
        .show(ui, |ui| {
            // Flat code header toolbar with language badge and Copy button
            Frame::none()
                .fill(Color32::from_rgb(25, 28, 36))
                .inner_margin(Margin::symmetric(10.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(display_lang)
                                .size(11.0)
                                .monospace()
                                .color(Color32::from_rgb(140, 145, 165)),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("📋 Copy")
                                            .size(11.0)
                                            .color(Color32::from_rgb(200, 205, 220)),
                                    )
                                    .fill(Color32::from_rgb(35, 38, 50))
                                    .stroke(Stroke::new(1.0, Color32::from_rgb(50, 54, 70)))
                                    .rounding(Rounding::same(4.0)),
                                )
                                .on_hover_text("Copy code to clipboard")
                                .clicked()
                            {
                                ui.output_mut(|o| o.copied_text = clean_code.to_string());
                            }
                        });
                    });
                });

            // Flat code content with horizontal scrolling for long source lines so preview pane is never pushed
            Frame::none()
                .inner_margin(Margin::symmetric(10.0, 8.0))
                .show(ui, |ui| {
                    egui::ScrollArea::horizontal()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(clean_code)
                                    .monospace()
                                    .size(12.0)
                                    .color(Color32::from_rgb(220, 225, 235)),
                            );
                        });
                });
        });
    ui.add_space(6.0);
}

fn render_markdown_table(ui: &mut Ui, rows: &[Vec<String>]) {
    if rows.is_empty() {
        return;
    }
    let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if max_cols == 0 {
        return;
    }

    // Flat sleek table container with equal column widths & horizontal scrolling if wide
    let col_w = 110.0;
    Frame::none()
        .fill(Color32::from_rgb(18, 20, 25))
        .stroke(Stroke::new(1.0, Color32::from_rgb(38, 42, 54)))
        .rounding(Rounding::same(4.0))
        .inner_margin(Margin::same(8.0))
        .show(ui, |ui| {
            egui::ScrollArea::horizontal()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new("md_table_grid")
                        .striped(true)
                        .num_columns(max_cols)
                        .min_col_width(col_w)
                        .max_col_width(col_w)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            for (row_idx, row) in rows.iter().enumerate() {
                                let is_header = row_idx == 0;
                                for col_idx in 0..max_cols {
                                    let text = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                                    ui.label(
                                        egui::RichText::new(text)
                                            .size(12.0)
                                            .strong()
                                            .color(if is_header {
                                                Color32::WHITE
                                            } else {
                                                Color32::from_rgb(210, 215, 225)
                                            }),
                                    );
                                }
                                ui.end_row();
                            }
                        });
                });
        });
}

fn render_excel_preview(ui: &mut Ui, path: &str) {
    use calamine::{open_workbook_auto, Reader};

    let mut workbook = match open_workbook_auto(path) {
        Ok(wb) => wb,
        Err(e) => {
            ui.label(format!("Could not read Excel workbook: {}", e));
            return;
        }
    };

    let sheet_names = workbook.sheet_names().to_owned();
    if sheet_names.is_empty() {
        ui.label("Empty Excel spreadsheet");
        return;
    }

    let first_sheet = &sheet_names[0];
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("📊 Sheet: ")
                .size(13.0)
                .color(Color32::from_rgb(140, 145, 160)),
        );
        ui.label(
            egui::RichText::new(first_sheet)
                .strong()
                .size(13.0)
                .color(Color32::from_rgb(34, 160, 85)),
        );
        if sheet_names.len() > 1 {
            ui.label(
                egui::RichText::new(format!("(+{} other sheets)", sheet_names.len() - 1))
                    .size(11.0)
                    .color(Color32::from_rgb(120, 125, 140)),
            );
        }
    });
    ui.add_space(6.0);

    if let Ok(range) = workbook.worksheet_range(first_sheet) {
        let rows: Vec<Vec<String>> = range
            .rows()
            .take(60)
            .map(|row| row.iter().take(15).map(|c| c.to_string()).collect())
            .collect();

        render_spreadsheet_grid(ui, &rows, "excel_grid");
    } else {
        ui.label("Could not parse sheet data");
    }
}

fn render_csv_preview(ui: &mut Ui, path: &str, is_tsv: bool) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            ui.label(format!("Could not read file: {}", e));
            return;
        }
    };

    let delimiter = if is_tsv { '\t' } else { ',' };
    let rows: Vec<Vec<String>> = content
        .lines()
        .take(80)
        .map(|line| {
            line.split(delimiter)
                .take(15)
                .map(|cell| cell.trim_matches('"').trim().to_string())
                .collect()
        })
        .collect();

    ui.label(
        egui::RichText::new(format!(
            "📊 {} rows previewed",
            rows.len()
        ))
        .size(12.0)
        .color(Color32::from_rgb(140, 145, 160)),
    );
    ui.add_space(6.0);

    render_spreadsheet_grid(ui, &rows, "csv_grid");
}

fn render_spreadsheet_grid(ui: &mut Ui, rows: &[Vec<String>], id: &'static str) {
    if rows.is_empty() {
        ui.label("No tabular data found");
        return;
    }

    egui::ScrollArea::both()
        .id_source(id)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
            if max_cols == 0 {
                return;
            }

            // Flat sleek table with uniform column width
            let col_w = 110.0;
            egui::Grid::new(format!("{}_inner", id))
                .striped(true)
                .num_columns(max_cols + 1)
                .min_col_width(col_w)
                .max_col_width(col_w)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    // Header row (A, B, C...)
                    ui.label(egui::RichText::new("#").size(11.0).color(Color32::from_rgb(110, 115, 130)));
                    for col_idx in 0..max_cols {
                        let col_letter = (b'A' + (col_idx as u8 % 26)) as char;
                        ui.label(
                            egui::RichText::new(col_letter.to_string())
                                .strong()
                                .size(11.5)
                                .color(Color32::from_rgb(150, 155, 175)),
                        );
                    }
                    ui.end_row();

                    // Data rows
                    for (row_idx, row) in rows.iter().enumerate() {
                        ui.label(
                            egui::RichText::new((row_idx + 1).to_string())
                                .size(11.0)
                                .color(Color32::from_rgb(110, 115, 130)),
                        );

                        for col_idx in 0..max_cols {
                            let text = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                            let is_header_row = row_idx == 0;
                            ui.label(
                                egui::RichText::new(text)
                                    .size(12.0)
                                    .strong()
                                    .color(if is_header_row {
                                        Color32::WHITE
                                    } else {
                                        Color32::from_rgb(210, 215, 225)
                                    }),
                            );
                        }
                        ui.end_row();
                    }
                });
        });
}

fn render_docx_preview(ui: &mut Ui, path: &str) {
    ui.heading("📘 Word Document Preview");
    ui.add_space(6.0);

    let doc_text = match extract_docx_text(path) {
        Some(t) if !t.is_empty() => t,
        _ => {
            ui.label("Could not extract readable text from Word document.");
            return;
        }
    };

    egui::ScrollArea::vertical()
        .id_source("docx_preview_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for paragraph in doc_text.lines() {
                let trimmed = paragraph.trim();
                if !trimmed.is_empty() {
                    ui.label(
                        egui::RichText::new(trimmed)
                            .size(13.0)
                            .color(Color32::from_rgb(220, 225, 235)),
                    );
                    ui.add_space(4.0);
                }
            }
        });
}

fn extract_docx_text(path: &str) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    let mut document_xml = archive.by_name("word/document.xml").ok()?;
    let mut xml_content = String::new();
    document_xml.read_to_string(&mut xml_content).ok()?;

    let mut paragraphs = Vec::new();
    let mut current_p = String::new();
    let mut in_wt = false;

    let mut chars = xml_content.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '<' {
            let mut tag = String::new();
            while let Some(&next) = chars.peek() {
                chars.next();
                if next == '>' {
                    break;
                }
                tag.push(next);
            }
            if tag.starts_with("w:t") {
                in_wt = true;
            } else if tag.starts_with("/w:t") {
                in_wt = false;
            } else if tag.starts_with("/w:p") {
                if !current_p.trim().is_empty() {
                    paragraphs.push(current_p.trim().to_string());
                }
                current_p.clear();
            }
        } else if in_wt {
            current_p.push(c);
        }
    }
    if !current_p.trim().is_empty() {
        paragraphs.push(current_p.trim().to_string());
    }

    Some(paragraphs.join("\n\n"))
}

fn render_pdf_preview(ui: &mut Ui, path: &str, pdf_renderer: &mut crate::ui::pdf_renderer::PdfRenderer) {
    let page_count = pdf_renderer.get_or_query_page_count(path);
    pdf_renderer.clear_cache_except(path);

    // Breathing room on sides so document does not touch borders
    let available_w = (ui.available_width() - 36.0).max(180.0);
    let target_raster_w = (available_w * 1.5).min(1200.0) as u16;

    egui::ScrollArea::vertical()
        .id_source("pdf_real_preview_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(4.0);
                for page_idx in 0..page_count {
                    if let Some(texture) = pdf_renderer.get_rendered_page(path, page_idx, target_raster_w) {
                        let tex_size = texture.size_vec2();
                        let aspect_ratio = if tex_size.x > 0.0 {
                            tex_size.y / tex_size.x
                        } else {
                            1.414
                        };
                        let draw_h = available_w * aspect_ratio;

                        // Clean paper sheet presentation with subtle separation
                        Frame::none()
                            .fill(Color32::WHITE)
                            .rounding(Rounding::same(2.0))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(40, 44, 54)))
                            .show(ui, |ui| {
                                ui.add(
                                    egui::Image::new(texture)
                                        .fit_to_exact_size(egui::Vec2::new(available_w, draw_h))
                                        .rounding(Rounding::same(2.0)),
                                );
                            });
                    } else {
                        // Sleek minimal placeholder while rasterizing in background
                        let (rect, _) = ui.allocate_exact_size(
                            egui::Vec2::new(available_w, available_w * 1.3),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(
                            rect,
                            Rounding::same(2.0),
                            Color32::from_rgb(26, 28, 36),
                        );
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("Page {} · Rendering...", page_idx + 1),
                            egui::FontId::proportional(12.0),
                            Color32::from_rgb(140, 145, 160),
                        );
                    }

                    ui.add_space(12.0);
                }
            });
        });
}


fn render_json_preview(ui: &mut Ui, path: &str) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            ui.label(format!("Could not read JSON: {}", e));
            return;
        }
    };

    egui::ScrollArea::vertical()
        .id_source("json_preview_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Indented formatting
            let mut indent: usize = 0;
            for line in content.lines().take(500) {
                let trimmed = line.trim();
                if trimmed.starts_with('}') || trimmed.starts_with(']') {
                    indent = indent.saturating_sub(1);
                }

                let indent_spaces = "  ".repeat(indent);
                ui.label(
                    egui::RichText::new(format!("{}{}", indent_spaces, trimmed))
                        .monospace()
                        .size(12.0)
                        .color(Color32::from_rgb(235, 180, 110)),
                );

                if trimmed.ends_with('{') || trimmed.ends_with('[') {
                    indent += 1;
                }
            }
        });
}

fn render_image_preview(ui: &mut Ui, path: &str) {
    ui.heading("🖼 Image Preview");
    ui.add_space(8.0);

    let uri = format!("file://{}", path.replace('\\', "/"));
    egui::ScrollArea::both()
        .id_source("image_preview_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let max_w = (ui.available_width() - 20.0).max(100.0);
            ui.add(
                egui::Image::new(uri)
                    .max_width(max_w)
                    .rounding(Rounding::same(8.0)),
            );
        });
}

fn render_code_or_text_preview(ui: &mut Ui, path: &str) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label("⚠️ Non-UTF8 or Binary Content");
                ui.add_space(6.0);
                if ui
                    .button("↗ Open with External Application")
                    .clicked()
                {
                    let _ = open::that(path);
                }
            });
            return;
        }
    };

    egui::ScrollArea::both()
        .id_source("code_preview_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let lines: Vec<&str> = content.lines().collect();
            let total_digits = format!("{}", lines.len()).len();

            for (idx, line) in lines.iter().enumerate() {
                ui.horizontal(|ui| {
                    let num_str = format!("{:>width$}", idx + 1, width = total_digits);
                    ui.label(
                        egui::RichText::new(num_str)
                            .monospace()
                            .size(12.0)
                            .color(Color32::from_rgb(95, 100, 115)),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(*line)
                            .monospace()
                            .size(12.0)
                            .color(Color32::from_rgb(220, 225, 235)),
                    );
                });
            }
        });
}

