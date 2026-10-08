use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use pdfium_render::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub struct PdfPageRenderResult {
    pub path: String,
    pub page_index: u16,
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
    pub success: bool,
}

pub struct PdfRenderer {
    tx: Sender<(String, u16, u16)>, // (path, page_index, target_width)
    rx: Receiver<PdfPageRenderResult>,
    cache: HashMap<(String, u16), TextureHandle>,
    page_counts: HashMap<String, u16>,
    requested: HashMap<(String, u16), bool>,
    failed_pages: HashMap<(String, u16), bool>,
}

fn create_pdfium() -> Option<Pdfium> {
    // 1. Try directory of the current executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let candidate = parent.join("pdfium.dll");
            if candidate.exists() {
                if let Ok(bindings) = Pdfium::bind_to_library(&candidate) {
                    return Some(Pdfium::new(bindings));
                }
            }
        }
    }

    // 2. Try current working directory
    let local_dll = Path::new("pdfium.dll");
    if local_dll.exists() {
        if let Ok(bindings) = Pdfium::bind_to_library(local_dll) {
            return Some(Pdfium::new(bindings));
        }
    }

    // 3. Fallback to platform library name lookup / system library
    if let Ok(bindings) = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")) {
        return Some(Pdfium::new(bindings));
    }

    if let Ok(bindings) = Pdfium::bind_to_system_library() {
        return Some(Pdfium::new(bindings));
    }

    None
}

impl PdfRenderer {
    pub fn new() -> Self {
        let (req_tx, req_rx) = channel::<(String, u16, u16)>();
        let (res_tx, res_rx) = channel::<PdfPageRenderResult>();

        // Background rendering thread: never blocks the egui UI frame rate
        thread::spawn(move || {
            let pdfium_opt = create_pdfium();
            let Some(pdfium) = pdfium_opt else {
                eprintln!("[PdfRenderer] Could not initialize Pdfium library");
                while let Ok((path, page_index, _)) = req_rx.recv() {
                    let _ = res_tx.send(PdfPageRenderResult {
                        path,
                        page_index,
                        width: 0,
                        height: 0,
                        rgba: Vec::new(),
                        success: false,
                    });
                }
                return;
            };

            while let Ok((path, page_index, target_width)) = req_rx.recv() {
                let doc = match pdfium.load_pdf_from_file(&path, None) {
                    Ok(d) => d,
                    Err(_) => {
                        let _ = res_tx.send(PdfPageRenderResult {
                            path,
                            page_index,
                            width: 0,
                            height: 0,
                            rgba: Vec::new(),
                            success: false,
                        });
                        continue;
                    }
                };

                let page = match doc.pages().get(page_index.into()) {
                    Ok(p) => p,
                    Err(_) => {
                        let _ = res_tx.send(PdfPageRenderResult {
                            path,
                            page_index,
                            width: 0,
                            height: 0,
                            rgba: Vec::new(),
                            success: false,
                        });
                        continue;
                    }
                };

                let render_config = PdfRenderConfig::new().set_target_width(target_width.max(300).into());
                let bitmap = match page.render_with_config(&render_config) {
                    Ok(b) => b,
                    Err(_) => {
                        let _ = res_tx.send(PdfPageRenderResult {
                            path,
                            page_index,
                            width: 0,
                            height: 0,
                            rgba: Vec::new(),
                            success: false,
                        });
                        continue;
                    }
                };

                let width = bitmap.width() as usize;
                let height = bitmap.height() as usize;
                let rgba = bitmap.as_rgba_bytes().to_vec();

                let _ = res_tx.send(PdfPageRenderResult {
                    path,
                    page_index,
                    width,
                    height,
                    rgba,
                    success: true,
                });
            }
        });

        Self {
            tx: req_tx,
            rx: res_rx,
            cache: HashMap::new(),
            page_counts: HashMap::new(),
            requested: HashMap::new(),
            failed_pages: HashMap::new(),
        }
    }

    pub fn get_or_query_page_count(&mut self, path: &str) -> u16 {
        if let Some(&count) = self.page_counts.get(path) {
            return count;
        }

        // Return 1 initially and let background thread determine or render without freezing the UI thread
        self.page_counts.insert(path.to_string(), 1);
        1
    }

    pub fn receive_rendered_textures(&mut self, ctx: &Context) {
        while let Ok(res) = self.rx.try_recv() {
            let key = (res.path.clone(), res.page_index);
            if res.success && res.width > 0 && res.height > 0 {
                let color_image = ColorImage::from_rgba_unmultiplied(
                    [res.width, res.height],
                    &res.rgba,
                );
                let handle = ctx.load_texture(
                    format!("pdf_{}_{}", res.path, res.page_index),
                    color_image,
                    TextureOptions::LINEAR,
                );
                self.cache.insert(key, handle);
            } else {
                self.failed_pages.insert(key, true);
            }
            ctx.request_repaint();
        }
    }

    pub fn is_page_failed(&self, path: &str, page_index: u16) -> bool {
        self.failed_pages.get(&(path.to_string(), page_index)).copied().unwrap_or(false)
    }

    pub fn get_rendered_page(
        &mut self,
        path: &str,
        page_index: u16,
        target_width: u16,
    ) -> Option<&TextureHandle> {
        let key = (path.to_string(), page_index);
        if self.cache.contains_key(&key) {
            return self.cache.get(&key);
        }

        // Request background rasterization if not already queued
        if !self.requested.contains_key(&key) {
            self.requested.insert(key, true);
            let _ = self.tx.send((path.to_string(), page_index, target_width));
        }

        None
    }

    pub fn clear_cache_except(&mut self, current_path: &str) {
        if self.cache.len() > 20 {
            self.cache.retain(|(p, _), _| p == current_path);
            self.requested.retain(|(p, _), _| p == current_path);
            self.failed_pages.retain(|(p, _), _| p == current_path);
        }
    }
}
