use crate::indexer::win_icon::extract_icon;
use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone)]
pub struct RawIconData {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone)]
pub struct IconCache {
    raw_icons: Arc<RwLock<HashMap<String, Arc<RawIconData>>>>,
    requested_keys: Arc<Mutex<HashSet<String>>>,
    textures: Arc<RwLock<HashMap<String, TextureHandle>>>,
    sender: Arc<Mutex<Sender<(String, PathBuf)>>>,
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}

impl IconCache {
    pub fn new() -> Self {
        let (tx, rx) = channel::<(String, PathBuf)>();
        let raw_icons: Arc<RwLock<HashMap<String, Arc<RawIconData>>>> = Arc::new(RwLock::new(HashMap::new()));
        let raw_icons_worker = raw_icons.clone();

        // Dedicated single background worker thread: avoids spawning 100 OS threads
        std::thread::Builder::new()
            .name("icon_worker".to_string())
            .spawn(move || {
                while let Ok((key, path)) = rx.recv() {
                    if let Some((w, h, rgba)) = extract_icon(&path) {
                        let icon_data = Arc::new(RawIconData {
                            width: w,
                            height: h,
                            rgba,
                        });
                        if let Ok(mut map) = raw_icons_worker.write() {
                            map.insert(key, icon_data);
                        }
                    }
                }
            })
            .ok();

        Self {
            raw_icons,
            requested_keys: Arc::new(Mutex::new(HashSet::new())),
            textures: Arc::new(RwLock::new(HashMap::new())),
            sender: Arc::new(Mutex::new(tx)),
        }
    }

    /// Retrieve texture if already loaded, or convert from cached raw bytes
    pub fn get_or_load_texture(&self, ctx: &Context, key: &str, path_hint: Option<&str>) -> Option<TextureHandle> {
        // 1. Check existing egui texture
        if let Ok(tex_map) = self.textures.read() {
            if let Some(handle) = tex_map.get(key) {
                return Some(handle.clone());
            }
        }

        // 2. Check if raw RGBA pixels are ready in the background cache
        if let Ok(raw_map) = self.raw_icons.read() {
            if let Some(raw) = raw_map.get(key) {
                let img = ColorImage::from_rgba_unmultiplied(
                    [raw.width as usize, raw.height as usize],
                    &raw.rgba,
                );
                let handle = ctx.load_texture(key, img, TextureOptions::LINEAR);
                if let Ok(mut tex_map) = self.textures.write() {
                    tex_map.insert(key.to_string(), handle.clone());
                }
                return Some(handle);
            }
        }

        // 3. If not cached, trigger single worker extraction
        if let Some(path) = path_hint {
            self.request_icon(key, path);
        }

        None
    }

    /// Send extraction request to dedicated background worker thread
    pub fn request_icon(&self, key: &str, path: &str) {
        let mut requested = match self.requested_keys.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        if requested.contains(key) {
            return;
        }
        requested.insert(key.to_string());
        drop(requested);

        if let Ok(sender_guard) = self.sender.lock() {
            let _ = sender_guard.send((key.to_string(), PathBuf::from(path)));
        }
    }

    /// Pre-cache an icon directly
    #[allow(dead_code)]
    pub fn insert_raw_icon(&self, key: &str, w: u32, h: u32, rgba: Vec<u8>) {
        if let Ok(mut map) = self.raw_icons.write() {
            map.insert(
                key.to_string(),
                Arc::new(RawIconData {
                    width: w,
                    height: h,
                    rgba,
                }),
            );
        }
    }
}
