use crate::indexer::icon_cache::IconCache;
use crate::indexer::scanner::FileRecord;
use crate::ui::results_list::file_badge_info;
use crate::utils::unicode::format_size;
use egui::{Color32, Frame, Margin, Rounding, Stroke, Ui, Vec2};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

const MAX_READ_BYTES: u64 = 256 * 1024; // 256 KB max read guard (prevents freezing on huge files)
const MAX_PREVIEW_LINES: usize = 300;   // 300 lines max rendered
const MAX_LINE_CHARS: usize = 300;      // 300 chars max per line (prevents egui font shaping lockups on minified CSS/JSON)

fn truncate_line(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let t: String = s.chars().take(max_chars).collect();
        format!("{}... [truncated]", t)
    }
}

fn read_bounded_text(path: &str) -> Result<(String, bool), String> {
    let file = fs::File::open(path).map_err(|e| format!("Cannot open file: {}", e))?;
    let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let is_truncated = file_len > MAX_READ_BYTES;

    let mut buf = Vec::with_capacity((file_len.min(MAX_READ_BYTES)) as usize);
    file.take(MAX_READ_BYTES).read_to_end(&mut buf).map_err(|e| format!("Read error: {}", e))?;

    // Check if binary (null bytes in first 1024 bytes)
    if buf.iter().take(1024).any(|&b| b == 0) {
        return Err("Binary content".to_string());
    }

    let text = match String::from_utf8(buf) {
        Ok(s) => s,
        Err(e) => {
            let bytes = e.into_bytes();
            String::from_utf8_lossy(&bytes).to_string()
        }
    };
    Ok((text, is_truncated))
}

#[derive(Clone)]
pub enum CachedPreview {
    Loading,
    Text {
        lines: Vec<String>,
        total_digits: usize,
        is_truncated: bool,
    },
    Markdown {
        content: String,
        is_truncated: bool,
    },
    Json {
        lines: Vec<String>,
        is_truncated: bool,
    },
    Docx {
        paragraphs: Vec<String>,
    },
    Excel {
        sheet_names: Vec<String>,
        first_sheet_name: String,
        rows: Vec<Vec<String>>,
    },
    Csv {
        rows: Vec<Vec<String>>,
        total_rows: usize,
    },
    Directory {
        entries: Vec<(String, bool, String)>,
        total_count: usize,
    },
    Image {
        width: usize,
        height: usize,
        rgba: Vec<u8>,
    },
    BinaryOrError(String),
}

use std::collections::HashMap;

pub struct PreviewCache {
    cached_path: String,
    content: Option<CachedPreview>,
    current_texture: Option<egui::TextureHandle>,
    mem_cache: HashMap<String, CachedPreview>,
    texture_cache: HashMap<String, egui::TextureHandle>,
    tx: Sender<(String, bool)>,
    rx: Receiver<(String, CachedPreview)>,
}

impl Default for PreviewCache {
    fn default() -> Self {
        Self::new()
    }
}

impl PreviewCache {
    pub fn new() -> Self {
        let (req_tx, req_rx) = channel::<(String, bool)>();
        let (res_tx, res_rx) = channel::<(String, CachedPreview)>();

        // Background file reading & parsing thread: guarantees main UI thread NEVER blocks on disk I/O!
        thread::spawn(move || {
            while let Ok((path, is_dir)) = req_rx.recv() {
                let path_clone = path.clone();
                let preview = std::panic::catch_unwind(move || {
                    Self::load_entry(&path_clone, is_dir)
                })
                .unwrap_or_else(|_| {
                    CachedPreview::BinaryOrError("Error loading preview".to_string())
                });
                let _ = res_tx.send((path, preview));
            }
        });

        Self {
            cached_path: String::new(),
            content: None,
            current_texture: None,
            mem_cache: HashMap::new(),
            texture_cache: HashMap::new(),
            tx: req_tx,
            rx: res_rx,
        }
    }

    pub fn poll(&mut self, ctx: &egui::Context) {
        let mut got_update = false;
        while let Ok((path, preview)) = self.rx.try_recv() {
            if let CachedPreview::Image { width, height, ref rgba } = preview {
                let color_img = egui::ColorImage::from_rgba_unmultiplied([width, height], rgba);
                let texture = ctx.load_texture(
                    &path,
                    color_img,
                    egui::TextureOptions::LINEAR,
                );
                self.texture_cache.insert(path.clone(), texture);
            }
            if self.cached_path == path {
                if let Some(tex) = self.texture_cache.get(&path) {
                    self.current_texture = Some(tex.clone());
                } else {
                    self.current_texture = None;
                }
                self.content = Some(preview.clone());
                got_update = true;
            } else {
                self.mem_cache.insert(path, preview);
            }
        }
        if got_update {
            ctx.request_repaint();
        }
    }

    pub fn get_or_load(&mut self, path: &str, is_dir: bool) -> &CachedPreview {
        if self.cached_path != path {
            self.cached_path = path.to_string();
            if let Some(existing) = self.mem_cache.get(path) {
                self.content = Some(existing.clone());
                self.current_texture = self.texture_cache.get(path).cloned();
            } else {
                self.content = Some(CachedPreview::Loading);
                self.current_texture = None;
                let _ = self.tx.send((path.to_string(), is_dir));
            }
        }
        self.content.as_ref().unwrap()
    }

    pub fn current_texture(&self) -> Option<&egui::TextureHandle> {
        self.current_texture.as_ref()
    }

    fn load_entry(path: &str, is_dir: bool) -> CachedPreview {
        if is_dir {
            return match fs::read_dir(path) {
                Ok(read_dir) => {
                    let mut entries = Vec::new();
                    for entry in read_dir.filter_map(|e| e.ok()).take(200) {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                        let size_str = entry.metadata().map(|m| format_size(m.len())).unwrap_or_default();
                        entries.push((name, is_dir, size_str));
                    }
                    let total_count = entries.len();
                    CachedPreview::Directory { entries, total_count }
                }
                Err(e) => CachedPreview::BinaryOrError(format!("Could not read directory: {}", e)),
            };
        }

        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "md" | "markdown" => {
                match read_bounded_text(path) {
                    Ok((content, is_truncated)) => CachedPreview::Markdown { content, is_truncated },
                    Err(e) => CachedPreview::BinaryOrError(e),
                }
            }
            "json" => {
                match read_bounded_text(path) {
                    Ok((text, is_truncated)) => {
                        let lines = if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                            let pretty = serde_json::to_string_pretty(&val).unwrap_or_else(|_| text.clone());
                            pretty.lines()
                                .take(MAX_PREVIEW_LINES)
                                .map(|l| truncate_line(l, MAX_LINE_CHARS))
                                .collect()
                        } else {
                            let mut formatted = Vec::new();
                            let mut indent: usize = 0;
                            for raw_line in text.lines().take(MAX_PREVIEW_LINES) {
                                let trimmed = raw_line.trim();
                                if trimmed.starts_with('}') || trimmed.starts_with(']') {
                                    indent = indent.saturating_sub(1);
                                }
                                let indent_spaces = "  ".repeat(indent.min(10));
                                let combined = format!("{}{}", indent_spaces, trimmed);
                                formatted.push(truncate_line(&combined, MAX_LINE_CHARS));
                                if trimmed.ends_with('{') || trimmed.ends_with('[') {
                                    indent += 1;
                                }
                            }
                            formatted
                        };
                        CachedPreview::Json { lines, is_truncated }
                    }
                    Err(e) => CachedPreview::BinaryOrError(e),
                }
            }
            "docx" => {
                match extract_docx_text(path) {
                    Some(t) if !t.is_empty() => {
                        let paragraphs: Vec<String> = t.lines()
                            .take(MAX_PREVIEW_LINES)
                            .map(|p| truncate_line(p, MAX_LINE_CHARS))
                            .filter(|p| !p.trim().is_empty())
                            .collect();
                        CachedPreview::Docx { paragraphs }
                    }
                    _ => CachedPreview::BinaryOrError("Could not extract readable text from Word document.".to_string()),
                }
            }
            "xlsx" | "xls" | "ods" => {
                use calamine::{open_workbook_auto, Reader};
                match open_workbook_auto(path) {
                    Ok(mut wb) => {
                        let sheet_names = wb.sheet_names().to_owned();
                        if sheet_names.is_empty() {
                            CachedPreview::BinaryOrError("Empty Excel workbook".to_string())
                        } else {
                            let first_sheet = sheet_names[0].clone();
                            let rows: Vec<Vec<String>> = wb.worksheet_range(&first_sheet)
                                .ok()
                                .map(|range| {
                                    range.rows()
                                        .take(60)
                                        .map(|row| row.iter().take(15).map(|c| truncate_line(&c.to_string(), 60)).collect())
                                        .collect()
                                })
                                .unwrap_or_default();
                            CachedPreview::Excel { sheet_names, first_sheet_name: first_sheet, rows }
                        }
                    }
                    Err(e) => CachedPreview::BinaryOrError(format!("Could not read Excel workbook: {}", e)),
                }
            }
            "csv" | "tsv" => {
                let is_tsv = ext == "tsv";
                match read_bounded_text(path) {
                    Ok((content, _)) => {
                        let delimiter = if is_tsv { '\t' } else { ',' };
                        let lines: Vec<&str> = content.lines().collect();
                        let total_rows = lines.len();
                        let rows: Vec<Vec<String>> = lines
                            .into_iter()
                            .take(60)
                            .map(|line| {
                                line.split(delimiter)
                                    .take(15)
                                    .map(|cell| truncate_line(cell.trim_matches('"').trim(), 60))
                                    .collect()
                            })
                            .collect();
                        CachedPreview::Csv { rows, total_rows }
                    }
                    Err(e) => CachedPreview::BinaryOrError(e),
                }
            }
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" => {
                match image::io::Reader::open(path)
                    .map_err(|e| e.to_string())
                    .and_then(|r| r.with_guessed_format().map_err(|e| e.to_string()))
                    .and_then(|r| r.decode().map_err(|e| e.to_string()))
                {
                    Ok(img) => {
                        let thumb = img.thumbnail(1000, 1000);
                        let rgba = thumb.to_rgba8();
                        let (w, h) = (rgba.width() as usize, rgba.height() as usize);
                        CachedPreview::Image {
                            width: w,
                            height: h,
                            rgba: rgba.into_raw(),
                        }
                    }
                    Err(e) => CachedPreview::BinaryOrError(format!("Could not decode image: {}", e)),
                }
            }
            "pdf" => {
                CachedPreview::BinaryOrError(String::new())
            }
            _ => {
                match read_bounded_text(path) {
                    Ok((text, is_truncated)) => {
                        let lines: Vec<String> = text.lines()
                            .take(MAX_PREVIEW_LINES)
                            .map(|l| truncate_line(l, MAX_LINE_CHARS))
                            .collect();
                        let total_digits = format!("{}", lines.len()).len().max(2);
                        CachedPreview::Text { lines, total_digits, is_truncated }
                    }
                    Err(e) => CachedPreview::BinaryOrError(e),
                }
            }
        }
    }
}

fn render_truncated_banner(ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("⚡ Large file · Preview limited to first 256 KB")
                .size(11.0)
                .color(Color32::from_rgb(180, 160, 80)),
        );
    });
    ui.add_space(4.0);
}

fn render_binary_or_error(ui: &mut Ui, path: &str, msg: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        let label_text = if msg.is_empty() {
            "⚠️ Non-UTF8 or Binary Content"
        } else {
            msg
        };
        ui.label(egui::RichText::new(label_text).color(Color32::from_rgb(160, 165, 180)));
        ui.add_space(8.0);
        if ui
            .button("↗ Open with External Application")
            .clicked()
        {
            let _ = open::that(path);
        }
    });
}

pub fn render_preview_panel(
    ui: &mut Ui,
    selected_file: Option<&FileRecord>,
    pdf_renderer: &mut crate::ui::pdf_renderer::PdfRenderer,
    icon_cache: &IconCache,
    preview_cache: &mut PreviewCache,
) {
    preview_cache.poll(ui.ctx());

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
                egui::RichText::new("Select a file or app to preview")
                    .size(13.5)
                    .strong()
                    .color(Color32::from_rgb(205, 210, 220)),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Applications, Documents, Code, PDF & Media")
                    .size(11.0)
                    .color(Color32::from_rgb(115, 120, 135)),
            );
        });
        return;
    };

    // Dedicated First-Class Application Profile UX (Section 8, 9, 24) - Software keeps its title & details
    if file.is_app() {
        render_application_profile(ui, file, icon_cache);
        return;
    }

    // Top action row for regular files (Yellow signature "Open" button on the right, shifted to the left)
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(24.0); // Moved to the left away from far right edge
            let open_btn = egui::Button::new(
                egui::RichText::new("Open ↗")
                    .size(12.0)
                    .strong()
                    .color(Color32::BLACK),
            )
            .fill(Color32::from_rgb(255, 230, 0)) // Yellow background matching Launch button
            .stroke(Stroke::NONE)
            .rounding(Rounding::same(14.0))
            .min_size(Vec2::new(76.0, 26.0));

            if ui
                .add(open_btn)
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_text("Open with default system program")
                .clicked()
            {
                let _ = open::that(&file.path);
            }
        });
    });

    ui.add_space(8.0);

    // Main File Preview Content Container
    Frame::none()
        .fill(Color32::from_rgb(20, 22, 28))
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            let ext = Path::new(&file.path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if ext == "pdf" {
                render_pdf_preview(ui, &file.path, pdf_renderer);
                return;
            }

            let cached = preview_cache.get_or_load(&file.path, file.is_dir);
            match cached {
                CachedPreview::Loading => {
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("Loading preview...")
                                .size(13.0)
                                .color(Color32::from_rgb(130, 135, 150)),
                        );
                    });
                }
                CachedPreview::Directory { entries, total_count } => {
                    render_directory_preview(ui, entries, *total_count);
                }
                CachedPreview::Markdown { content, is_truncated } => {
                    if *is_truncated {
                        render_truncated_banner(ui);
                    }
                    render_markdown_preview(ui, content);
                }
                CachedPreview::Json { lines, is_truncated } => {
                    if *is_truncated {
                        render_truncated_banner(ui);
                    }
                    render_json_preview(ui, lines);
                }
                CachedPreview::Docx { paragraphs } => {
                    render_docx_preview(ui, paragraphs);
                }
                CachedPreview::Excel { sheet_names, first_sheet_name, rows } => {
                    render_excel_preview(ui, sheet_names, first_sheet_name, rows);
                }
                CachedPreview::Csv { rows, total_rows } => {
                    render_csv_preview(ui, rows, *total_rows);
                }
                CachedPreview::Image { .. } => {
                    if let Some(texture) = preview_cache.current_texture() {
                        render_image_preview(ui, texture);
                    }
                }
                CachedPreview::Text { lines, total_digits, is_truncated } => {
                    if *is_truncated {
                        render_truncated_banner(ui);
                    }
                    render_code_or_text_preview(ui, lines, *total_digits);
                }
                CachedPreview::BinaryOrError(msg) => {
                    render_binary_or_error(ui, &file.path, msg);
                }
            }
        });
}

fn render_directory_preview(ui: &mut Ui, entries: &[(String, bool, String)], total_count: usize) {
    ui.label(
        egui::RichText::new(format!("{} item(s) in folder", total_count))
            .size(12.0)
            .color(Color32::from_rgb(140, 145, 160)),
    );
    ui.add_space(6.0);

    egui::ScrollArea::vertical()
        .id_source("folder_preview_scroll")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (name, is_dir, size_str) in entries {
                let (icon, color) = file_badge_info(name, *is_dir);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(icon).color(color));
                    ui.label(egui::RichText::new(name).color(Color32::from_rgb(220, 225, 235)));
                    if !*is_dir {
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

fn render_application_profile(ui: &mut Ui, file: &FileRecord, icon_cache: &IconCache) {
    let app_name = file.display_name();
    let publisher = file.publisher();
    let version = file.version();
    let launch_target = file.launch_target();
    let icon_key = file.icon_cache_key();

    let full_w = ui.available_width();

    ui.vertical_centered(|ui| {
        ui.set_width(full_w);
        ui.add_space(16.0);

        // 1. Prominent Application Icon (Sleek container)
        let texture_opt = icon_cache.get_or_load_texture(ui.ctx(), &icon_key, Some(&file.path));
        if let Some(texture) = texture_opt {
            ui.add(
                egui::Image::new(&texture)
                    .fit_to_exact_size(Vec2::new(64.0, 64.0))
                    .rounding(Rounding::same(12.0)),
            );
        } else {
            let (icon_rect, _) = ui.allocate_exact_size(Vec2::new(64.0, 64.0), egui::Sense::hover());
            ui.painter().rect_filled(
                icon_rect,
                Rounding::same(12.0),
                Color32::from_rgb(25, 29, 40),
            );
            ui.painter().rect_stroke(
                icon_rect,
                Rounding::same(12.0),
                Stroke::new(1.0, Color32::from_rgb(46, 56, 78)),
            );
            let words: Vec<&str> = app_name.split_whitespace().collect();
            let initials = if words.len() >= 2 {
                format!(
                    "{}{}",
                    words[0].chars().next().unwrap_or('A').to_ascii_uppercase(),
                    words[1].chars().next().unwrap_or('P').to_ascii_uppercase()
                )
            } else {
                "APP".to_string()
            };
            ui.painter().text(
                icon_rect.center(),
                egui::Align2::CENTER_CENTER,
                initials,
                egui::FontId::proportional(22.0),
                Color32::from_rgb(0, 205, 250),
            );
        }

        ui.add_space(12.0);

        // 2. Application Name (Title)
        ui.label(
            egui::RichText::new(app_name)
                .size(21.5)
                .strong()
                .color(Color32::WHITE),
        );

        ui.add_space(5.0);

        // 3. Sleek Badges and Subtitle row (Label, Publisher, Version)
        ui.horizontal(|ui| {
            // APP pill badge
            Frame::none()
                .fill(Color32::from_rgb(0, 195, 240).linear_multiply(0.18))
                .rounding(Rounding::same(4.0))
                .inner_margin(Margin::symmetric(6.0, 2.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("APP")
                            .size(10.5)
                            .strong()
                            .color(Color32::from_rgb(0, 195, 240)),
                    );
                });

            if let Some(pub_name) = publisher {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(pub_name)
                        .size(12.5)
                        .color(Color32::from_rgb(165, 170, 185)),
                );
            }

            if let Some(ver) = version {
                ui.add_space(4.0);
                Frame::none()
                    .fill(Color32::from_rgb(26, 28, 36))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::symmetric(5.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!("v{}", ver))
                                .size(10.5)
                                .color(Color32::from_rgb(135, 140, 155)),
                        );
                    });
            }
        });

        ui.add_space(18.0);

        // 4. Flat Location Card (No shadow, no stroke, completely flat, sleek dark surface)
        let loc_box_width = (full_w - 48.0).clamp(240.0, 440.0);
        Frame::none()
            .fill(Color32::from_rgb(22, 24, 31))
            .stroke(Stroke::NONE)
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(14.0, 10.0))
            .show(ui, |ui| {
                ui.set_width(loc_box_width);

                // Header with LOCATION label and Copy button
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("LOCATION")
                            .size(10.5)
                            .strong()
                            .color(Color32::from_rgb(115, 120, 135)),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let copy_btn = egui::Button::new(
                            egui::RichText::new("📋 Copy")
                                .size(11.0)
                                .color(Color32::from_rgb(140, 145, 165)),
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE);

                        if ui
                            .add(copy_btn)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text("Copy path to clipboard")
                            .clicked()
                        {
                            ui.ctx().output_mut(|o| o.copied_text = launch_target.to_string());
                        }
                    });
                });

                ui.add_space(4.0);

                // File path display
                ui.label(
                    egui::RichText::new(launch_target)
                        .size(12.5)
                        .color(Color32::from_rgb(195, 200, 215)),
                );

                if let Some(ref meta) = file.app_metadata {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Source:")
                                .size(11.0)
                                .color(Color32::from_rgb(105, 110, 125)),
                        );
                        ui.label(
                            egui::RichText::new(&meta.source)
                                .size(11.0)
                                .color(Color32::from_rgb(135, 140, 155)),
                        );
                    });
                }
            });

        ui.add_space(22.0);

        // 5. Centered Launch Action Button
        let launch_btn = egui::Button::new(
            egui::RichText::new("Launch  ↗")
                .size(14.5)
                .strong()
                .color(Color32::BLACK),
        )
        .fill(Color32::from_rgb(255, 230, 0)) // Signature SearchForge Yellow
        .stroke(Stroke::NONE)
        .rounding(Rounding::same(19.0))
        .min_size(Vec2::new(170.0, 38.0));

        if ui
            .add(launch_btn)
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text("Launch application (or press Enter)")
            .clicked()
        {
            let _ = open::that(launch_target);
        }
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

fn render_markdown_preview(ui: &mut Ui, content: &str) {
    let max_w = ui.available_width();
    egui::ScrollArea::vertical()
        .id_source("markdown_preview_scroll")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_max_width(max_w);

            let mut in_code_block = false;
            let mut code_lang = String::new();
            let mut code_accumulator = String::new();

            let lines: Vec<&str> = content.lines().take(MAX_PREVIEW_LINES).collect();
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

fn render_excel_preview(ui: &mut Ui, sheet_names: &[String], first_sheet: &str, rows: &[Vec<String>]) {
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

    render_spreadsheet_grid(ui, rows, "excel_grid");
}

fn render_csv_preview(ui: &mut Ui, rows: &[Vec<String>], total_rows: usize) {
    ui.label(
        egui::RichText::new(format!(
            "📊 {} rows previewed",
            total_rows
        ))
        .size(12.0)
        .color(Color32::from_rgb(140, 145, 160)),
    );
    ui.add_space(6.0);

    render_spreadsheet_grid(ui, rows, "csv_grid");
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

fn render_docx_preview(ui: &mut Ui, paragraphs: &[String]) {
    egui::ScrollArea::vertical()
        .id_source("docx_preview_scroll")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for paragraph in paragraphs {
                ui.label(
                    egui::RichText::new(paragraph)
                        .size(13.0)
                        .color(Color32::from_rgb(220, 225, 235)),
                );
                ui.add_space(4.0);
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

    let scroll_id = format!("pdf_scroll_{}", path);
    egui::ScrollArea::vertical()
        .id_source(scroll_id)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
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
                    } else if pdf_renderer.is_page_failed(path, page_idx) {
                        let (rect, _) = ui.allocate_exact_size(
                            egui::Vec2::new(available_w, 140.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(
                            rect,
                            Rounding::same(4.0),
                            Color32::from_rgb(26, 28, 36),
                        );
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "📄 PDF preview requires external viewer",
                            egui::FontId::proportional(12.5),
                            Color32::from_rgb(160, 165, 180),
                        );
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

fn render_json_preview(ui: &mut Ui, lines: &[String]) {
    egui::ScrollArea::vertical()
        .id_source("json_preview_scroll")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for line in lines {
                ui.label(
                    egui::RichText::new(line)
                        .monospace()
                        .size(12.0)
                        .color(Color32::from_rgb(235, 180, 110)),
                );
            }
        });
}

fn render_image_preview(ui: &mut Ui, texture: &egui::TextureHandle) {
    egui::ScrollArea::both()
        .id_source("image_preview_scroll")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let max_w = (ui.available_width() - 20.0).max(100.0);
            ui.add(
                egui::Image::new(texture)
                    .max_width(max_w)
                    .rounding(Rounding::same(8.0)),
            );
        });
}

fn render_code_or_text_preview(ui: &mut Ui, lines: &[String], total_digits: usize) {
    egui::ScrollArea::both()
        .id_source("code_preview_scroll")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .auto_shrink([false, false])
        .show(ui, |ui| {
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
                        egui::RichText::new(line)
                            .monospace()
                            .size(12.0)
                            .color(Color32::from_rgb(220, 225, 235)),
                    );
                });
            }
        });
}

