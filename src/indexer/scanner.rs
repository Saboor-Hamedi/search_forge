use crate::indexer::app_scanner::{
    discover_installed_applications, AppMetadata, SearchResultType,
};
use crate::indexer::filters::should_ignore;
use crate::indexer::icon_cache::IconCache;
use crate::utils::unicode::normalize_for_search;
use std::path::Path;
use std::sync::{Arc, RwLock};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct FileRecord {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub item_type: SearchResultType,
    pub app_metadata: Option<AppMetadata>,
}

impl FileRecord {
    pub fn is_app(&self) -> bool {
        self.item_type == SearchResultType::Application
    }

    pub fn display_name(&self) -> &str {
        if let Some(ref meta) = self.app_metadata {
            &meta.display_name
        } else if self.name.to_ascii_lowercase().ends_with(".lnk") {
            &self.name[..self.name.len().saturating_sub(4)]
        } else {
            &self.name
        }
    }

    pub fn launch_target(&self) -> &str {
        if let Some(ref meta) = self.app_metadata {
            if let Some(ref shortcut) = meta.shortcut_path {
                shortcut
            } else {
                &meta.target_path
            }
        } else {
            &self.path
        }
    }

    pub fn publisher(&self) -> Option<&str> {
        self.app_metadata.as_ref().and_then(|m| m.publisher.as_deref())
    }

    pub fn version(&self) -> Option<&str> {
        self.app_metadata.as_ref().and_then(|m| m.version.as_deref())
    }

    pub fn icon_cache_key(&self) -> String {
        if let Some(ref meta) = self.app_metadata {
            meta.icon_path.clone()
        } else {
            self.path.clone()
        }
    }
}

/// Helper function to extract initials/acronym from text (e.g. "Visual Studio Code" -> "vsc")
fn extract_acronym(text: &str) -> String {
    let mut acr = String::new();
    for word in text.split_whitespace() {
        if let Some(first_char) = word.chars().next() {
            acr.push(first_char);
        }
    }
    acr
}

/// Calculate relevance score for ranking
pub fn score_record(norm_query: &str, record: &FileRecord) -> i64 {
    if norm_query.is_empty() {
        return 0;
    }

    if record.item_type == SearchResultType::Application {
        let display_name = record.display_name();
        let norm_app_name = normalize_for_search(display_name);

        // 1. Exact match or major priority match (Terminal, cmd, etc.)
        if (norm_query == "terminal" || norm_query == "term" || norm_query == "wt")
            && (norm_app_name.contains("terminal") || norm_app_name == "windows terminal" || norm_app_name == "wt")
        {
            // Windows Terminal gets supreme ranking
            if norm_app_name.contains("windows terminal") || norm_app_name == "wt" {
                return 32_000;
            }
            return 30_000;
        }

        if norm_app_name == norm_query {
            return 26_000;
        }

        // 2. Acronym or canonical alias match
        let acronym = extract_acronym(&norm_app_name);
        if acronym == norm_query {
            return 22_000;
        }
        if norm_query == "cmd" && (norm_app_name.contains("command prompt") || norm_app_name.contains("terminal")) {
            return 25_000;
        }
        if norm_query == "vscode" && norm_app_name.contains("visual studio code") {
            return 22_000;
        }
        if norm_query == "sublime" && norm_app_name.contains("sublime") {
            return 22_000;
        }
        if norm_query == "subl" && norm_app_name.contains("sublime") {
            return 21_000;
        }
        if norm_query == "netbeans" && norm_app_name.contains("netbeans") {
            return 22_000;
        }
        if norm_query == "dbeaver" && norm_app_name.contains("dbeaver") {
            return 22_000;
        }
        if norm_query == "vs" && norm_app_name.contains("visual studio") {
            return 20_000;
        }

        // 3. Application name starts with query
        if norm_app_name.starts_with(norm_query) {
            return 19_000;
        }

        // 4. Word boundary match
        let words: Vec<&str> = norm_app_name.split_whitespace().collect();
        if words.iter().any(|w| w.starts_with(norm_query)) {
            return 16_000;
        }

        // 5. Application name contains query substring
        if norm_app_name.contains(norm_query) {
            if norm_query.len() >= 3 && norm_app_name.contains("terminal") {
                return 28_000;
            }
            return 13_000;
        }

        // 6. Space-insensitive match (e.g. "visualstudiocode" matches "visual studio code")
        let compact_name: String = norm_app_name.chars().filter(|c| !c.is_whitespace()).collect();
        let compact_query: String = norm_query.chars().filter(|c| !c.is_whitespace()).collect();
        if compact_name.contains(&compact_query) {
            return 12_000;
        }

        // 7. Executable target match
        let target_stem = Path::new(&record.path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let norm_stem = normalize_for_search(target_stem);
        if norm_stem == norm_query {
            return 11_000;
        }
        if norm_stem.starts_with(norm_query) {
            return 9_000;
        }
        if norm_stem.contains(norm_query) {
            return 7_000;
        }

        // 8. Publisher match
        if let Some(pub_name) = record.publisher() {
            let norm_pub = normalize_for_search(pub_name);
            if norm_pub.contains(norm_query) {
                return 5_000;
            }
        }

        // 9. Path match fallback
        let norm_path = normalize_for_search(&record.path);
        if norm_path.contains(norm_query) {
            return 3_500;
        }

        0
    } else {
        // Ordinary file or folder
        let norm_name = normalize_for_search(&record.name);

        if norm_name == norm_query {
            return 2_000;
        }
        if norm_name.starts_with(norm_query) {
            return 1_600;
        }
        let words: Vec<&str> = norm_name.split_whitespace().collect();
        if words.iter().any(|w| w.starts_with(norm_query)) {
            return 1_300;
        }
        if norm_name.contains(norm_query) {
            return 900;
        }

        let norm_path = normalize_for_search(&record.path);
        if norm_path.contains(norm_query) {
            return 200;
        }

        0
    }
}

/// Walk roots in background thread, discover installed applications, and populate `store`.
pub fn start_background_scan(
    ignored: Vec<String>,
    store: Arc<RwLock<Vec<FileRecord>>>,
    icon_cache: IconCache,
) {
    std::thread::spawn(move || {
        // --- PHASE 1: Software Discovery & Premium Application Indexing ---
        // Runs immediately so applications are ready within milliseconds of launch!
        let discovered_apps = discover_installed_applications();
        let mut app_records = Vec::with_capacity(discovered_apps.len());

        for app in &discovered_apps {
            let record = FileRecord {
                name: app.name.clone(),
                path: app.target_path.clone(),
                size: 0,
                is_dir: false,
                item_type: SearchResultType::Application,
                app_metadata: Some(AppMetadata {
                    name: app.name.clone(),
                    display_name: app.display_name.clone(),
                    publisher: app.publisher.clone(),
                    version: app.version.clone(),
                    target_path: app.target_path.clone(),
                    shortcut_path: app.shortcut_path.clone(),
                    icon_path: app.icon_path.clone(),
                    source: app.source.clone(),
                }),
            };
            app_records.push(record);

            // Pre-extract icon in background cache
            icon_cache.request_icon(&app.icon_path, &app.icon_path);
        }

        if let Ok(mut lock) = store.write() {
            lock.append(&mut app_records);
        }

        // --- PHASE 2: User Environment & Workspace Filesystem Discovery ---
        let mut roots = Vec::new();

        // Target user's primary document, download, desktop, and project folders
        if let Some(home) = dirs::home_dir() {
            let user_subfolders = [
                "Desktop", "Documents", "Downloads", "Pictures", "Videos", "Music", "Projects", "source", "repos"
            ];
            for sub in &user_subfolders {
                let p = home.join(sub);
                if p.exists() {
                    roots.push(p);
                }
            }
        }

        // Current working directory (e.g. active workspace or project)
        if let Ok(cwd) = std::env::current_dir() {
            if !roots.iter().any(|r| cwd.starts_with(r)) {
                roots.push(cwd);
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
                    if let Some(name) = entry.file_name().to_str() {
                        // Skip hidden dot-directory subtrees (e.g. .git, .vscode, .idea)
                        if name.starts_with('.') && name != ".env" {
                            return false;
                        }
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

                if metadata.is_dir() {
                    continue;
                }

                let path_str = entry.path().to_string_lossy().to_string();
                let record = FileRecord {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path: path_str,
                    size: metadata.len(),
                    is_dir: false,
                    item_type: SearchResultType::File,
                    app_metadata: None,
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

/// High performance, premium ranked search with instant early termination
pub fn search_records(store: &[FileRecord], query: &str, limit: usize) -> Vec<FileRecord> {
    let trimmed = query.trim();

    // Section 12: Empty query suggestions - return top installed applications
    if trimmed.is_empty() {
        return store
            .iter()
            .filter(|r| r.item_type == SearchResultType::Application)
            .take(limit)
            .cloned()
            .collect();
    }

    let norm_query = normalize_for_search(trimmed);

    // Tier 1: Search applications first (only ~100 items, finishes in < 0.02ms)
    let mut app_matches: Vec<(i64, &FileRecord)> = Vec::new();
    for record in store {
        if record.item_type == SearchResultType::Application {
            let score = score_record(&norm_query, record);
            if score > 0 {
                app_matches.push((score, record));
            }
        }
    }
    app_matches.sort_by(|a, b| b.0.cmp(&a.0));

    // If applications alone fulfill the limit, return immediately without touching files
    if app_matches.len() >= limit {
        return app_matches
            .into_iter()
            .take(limit)
            .map(|(_, r)| r.clone())
            .collect();
    }

    let remaining_needed = limit - app_matches.len();
    let mut file_matches: Vec<(i64, &FileRecord)> = Vec::new();

    // Tier 2: Search ordinary files, score them, and sort by relevance
    for record in store {
        if record.item_type != SearchResultType::Application {
            let score = score_record(&norm_query, record);
            if score > 0 {
                file_matches.push((score, record));
            }
        }
    }
    file_matches.sort_by(|a, b| b.0.cmp(&a.0));

    let mut result = Vec::with_capacity(limit);
    for (_, app) in app_matches {
        result.push(app.clone());
    }
    for (_, file) in file_matches.into_iter().take(remaining_needed) {
        result.push(file.clone());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_app(name: &str, display_name: &str, publisher: Option<&str>, path: &str) -> FileRecord {
        FileRecord {
            name: name.to_string(),
            path: path.to_string(),
            size: 0,
            is_dir: false,
            item_type: SearchResultType::Application,
            app_metadata: Some(AppMetadata {
                name: name.to_string(),
                display_name: display_name.to_string(),
                publisher: publisher.map(|s| s.to_string()),
                version: Some("1.0.0".to_string()),
                target_path: path.to_string(),
                shortcut_path: None,
                icon_path: path.to_string(),
                source: "Test".to_string(),
            }),
        }
    }

    fn create_test_file(name: &str, path: &str) -> FileRecord {
        FileRecord {
            name: name.to_string(),
            path: path.to_string(),
            size: 1024,
            is_dir: false,
            item_type: SearchResultType::File,
            app_metadata: None,
        }
    }

    #[test]
    fn test_search_finds_visual_studio_code_above_files() {
        let store = vec![
            create_test_file("vscode_settings.json", "C:/Users/User/.config/vscode_settings.json"),
            create_test_file("vscode.txt", "C:/Users/User/Desktop/vscode.txt"),
            create_test_app("Visual Studio Code", "Visual Studio Code", Some("Microsoft"), "C:/Programs/VSCode/Code.exe"),
            create_test_file("notes_about_vscode.md", "C:/Users/User/Documents/notes_about_vscode.md"),
        ];

        let results = search_records(&store, "vscode", 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].display_name(), "Visual Studio Code");
        assert_eq!(results[0].item_type, SearchResultType::Application);
    }

    #[test]
    fn test_search_finds_sublime_text() {
        let store = vec![
            create_test_file("sublime_plugin.py", "C:/projects/sublime_plugin.py"),
            create_test_app("Sublime Text", "Sublime Text", Some("Sublime HQ"), "C:/Program Files/Sublime Text/sublime_text.exe"),
        ];

        let results = search_records(&store, "sublime", 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].display_name(), "Sublime Text");
        assert_eq!(results[0].item_type, SearchResultType::Application);
    }

    #[test]
    fn test_search_finds_apache_netbeans() {
        let store = vec![
            create_test_file("netbeans_project.xml", "C:/projects/netbeans_project.xml"),
            create_test_app("Apache NetBeans", "Apache NetBeans", Some("Apache"), "C:/Program Files/NetBeans/bin/netbeans64.exe"),
        ];

        let results = search_records(&store, "netbeans", 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].display_name(), "Apache NetBeans");
        assert_eq!(results[0].item_type, SearchResultType::Application);
    }

    #[test]
    fn test_search_finds_dbeaver() {
        let store = vec![
            create_test_file("dbeaver_export.sql", "C:/exports/dbeaver_export.sql"),
            create_test_app("DBeaver", "DBeaver", Some("DBeaver Corp"), "C:/Users/User/AppData/Local/DBeaver/dbeaver.exe"),
        ];

        let results = search_records(&store, "dbeaver", 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].display_name(), "DBeaver");
        assert_eq!(results[0].item_type, SearchResultType::Application);
    }

    #[test]
    fn test_search_finds_windows_terminal() {
        let store = vec![
            create_test_file("terminal_output.log", "C:/logs/terminal_output.log"),
            create_test_file("terminal.sh", "C:/scripts/terminal.sh"),
            create_test_app("Windows Terminal", "Windows Terminal", Some("Microsoft Corporation"), "C:/Users/User/AppData/Local/Microsoft/WindowsApps/wt.exe"),
        ];

        let results = search_records(&store, "terminal", 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].display_name(), "Windows Terminal");
        assert_eq!(results[0].item_type, SearchResultType::Application);

        let results_wt = search_records(&store, "wt", 5);
        assert!(!results_wt.is_empty());
        assert_eq!(results_wt[0].display_name(), "Windows Terminal");
    }

    #[test]
    fn test_empty_query_returns_top_applications() {
        let store = vec![
            create_test_file("file1.txt", "C:/file1.txt"),
            create_test_app("Visual Studio Code", "Visual Studio Code", Some("Microsoft"), "C:/Code.exe"),
            create_test_app("Sublime Text", "Sublime Text", Some("Sublime HQ"), "C:/sublime_text.exe"),
            create_test_file("file2.txt", "C:/file2.txt"),
        ];

        let results = search_records(&store, "", 5);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.item_type == SearchResultType::Application));
    }
}

