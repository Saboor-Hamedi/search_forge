use crate::indexer::filters::should_ignore;
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct FileRecord {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

use crate::utils::unicode::normalize_for_search;
use std::sync::{Arc, RwLock};

/// Walk roots in a background thread and populate `store` asynchronously.
pub fn start_background_scan(ignored: Vec<String>, store: Arc<RwLock<Vec<FileRecord>>>) {
    std::thread::spawn(move || {
        let mut roots = Vec::new();

        // 1. Index Start Menu shortcuts first so software is instantly searchable
        if let Some(home) = dirs::home_dir() {
            let user_start_menu = home.join("AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs");
            if user_start_menu.exists() {
                roots.push(user_start_menu);
            }
        }
        let common_start_menu = std::path::PathBuf::from("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs");
        if common_start_menu.exists() {
            roots.push(common_start_menu);
        }

        // 2. Dynamically detect all active drives on the system (A:\ to Z:\)
        #[cfg(target_os = "windows")]
        {
            for drive_letter in b'A'..=b'Z' {
                let drive_str = format!("{}:\\", drive_letter as char);
                let drive_path = std::path::PathBuf::from(&drive_str);
                if drive_path.exists() && std::fs::read_dir(&drive_path).is_ok() {
                    roots.push(drive_path);
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            if let Some(home) = dirs::home_dir() {
                roots.push(home);
            }
        }

        for root in roots {
            if !root.exists() {
                continue;
            }
            let walker = WalkDir::new(&root)
                .follow_links(false)
                .into_iter()
                .filter_entry(|entry| {
                    if entry.depth() == 0 {
                        return true;
                    }
                    let path_str = entry.path().to_string_lossy();
                    !should_ignore(&path_str, &ignored)
                });

            let mut batch = Vec::new();
            for entry in walker.filter_map(|e| e.ok()) {
                if entry.depth() == 0 {
                    continue;
                }

                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                // Extract all files from folders; do not index folder entries themselves
                if metadata.is_dir() {
                    continue;
                }

                let path_str = entry.path().to_string_lossy().to_string();
                let record = FileRecord {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path: path_str,
                    size: metadata.len(),
                    is_dir: false,
                };

                batch.push(record);

                if batch.len() >= 2000 {
                    if let Ok(mut lock) = store.write() {
                        lock.append(&mut batch);
                    }
                    batch.clear();
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            }

            if !batch.is_empty() {
                if let Ok(mut lock) = store.write() {
                    lock.append(&mut batch);
                }
                batch.clear();
            }
        }
    });
}

/// Blazing fast in-memory search supporting Persian, Arabic, English, and all scripts
pub fn search_records(store: &[FileRecord], query: &str, limit: usize) -> Vec<FileRecord> {
    if query.trim().is_empty() {
        return Vec::new();
    }
    let norm_query = normalize_for_search(query.trim());
    let mut matches = Vec::new();

    // Match on filename first
    for record in store {
        let norm_name = normalize_for_search(&record.name);
        if norm_name.contains(&norm_query) {
            matches.push(record.clone());
            if matches.len() >= limit {
                return matches;
            }
        }
    }

    // If query has slashes and we need more results, match on full path
    if matches.len() < limit && (norm_query.contains('/') || norm_query.contains('\\')) {
        for record in store {
            let norm_path = normalize_for_search(&record.path);
            if norm_path.contains(&norm_query) && !matches.iter().any(|m| m.path == record.path) {
                matches.push(record.clone());
                if matches.len() >= limit {
                    return matches;
                }
            }
        }
    }

    matches
}



