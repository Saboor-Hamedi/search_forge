use crate::indexer::filters::should_ignore;
use std::path::PathBuf;
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
pub fn start_background_scan(roots: Vec<PathBuf>, ignored: Vec<String>, store: Arc<RwLock<Vec<FileRecord>>>) {
    std::thread::spawn(move || {
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

                if batch.len() >= 200 {
                    if let Ok(mut lock) = store.write() {
                        lock.append(&mut batch);
                    }
                    batch.clear();
                }
            }

            if !batch.is_empty() {
                if let Ok(mut lock) = store.write() {
                    lock.append(&mut batch);
                }
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



