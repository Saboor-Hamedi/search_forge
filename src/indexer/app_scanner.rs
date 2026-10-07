use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchResultType {
    Application,
    Folder,
    File,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AppMetadata {
    pub name: String,
    pub display_name: String,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub target_path: String,
    pub shortcut_path: Option<String>,
    pub icon_path: String,
    pub source: String,
}

#[derive(Clone, Debug)]
pub struct DiscoveredApp {
    pub name: String,
    pub display_name: String,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub target_path: String,
    pub shortcut_path: Option<String>,
    pub icon_path: String,
    pub source: String,
}

/// Filter out internal or helper executables
pub fn is_helper_or_internal_executable(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let stem = lower.strip_suffix(".exe").unwrap_or(&lower);

    let helper_tokens = [
        "uninstall",
        "unins000",
        "unins001",
        "uninstaller",
        "uninst",
        "helper",
        "update",
        "updater",
        "crashpad",
        "crash_handler",
        "crashreporter",
        "plugin_host",
        "runtime",
        "vcredist",
        "dxsetup",
        "elevate",
        "installer",
        "setup",
        "setup_x64",
        "setup_x86",
        "repair",
        "patch",
        "worker",
        "daemon",
        "service",
        "diagnostics",
        "feedback",
        "dump",
        "agent",
        "broker",
        "notification_helper",
    ];

    for token in &helper_tokens {
        if stem == *token || stem.contains(token) {
            return true;
        }
    }
    false
}

/// Filter out shortcuts for uninstallers, help documents, websites, etc.
pub fn is_helper_or_documentation_shortcut(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let stem = lower.strip_suffix(".lnk").unwrap_or(&lower).trim();

    let doc_tokens = [
        "uninstall",
        "uninstaller",
        "remove",
        "help",
        "documentation",
        "readme",
        "website",
        "release notes",
        "manual",
        "license",
        "changelog",
        "support",
        "faq",
        "online help",
        "configuration tool",
        "reset",
    ];

    for token in &doc_tokens {
        if stem == *token
            || stem.starts_with(&format!("{} ", token))
            || stem.ends_with(&format!(" {}", token))
            || stem.contains(&format!("uninstall {}", token))
        {
            return true;
        }
        if stem.contains("uninstall") {
            return true;
        }
    }
    false
}

/// Clean up an application name for display and comparison
pub fn clean_app_display_name(name: &str) -> String {
    let mut s = name.trim();
    if s.to_ascii_lowercase().ends_with(".lnk") {
        s = &s[..s.len().saturating_sub(4)];
    }
    if s.to_ascii_lowercase().ends_with(".exe") {
        s = &s[..s.len().saturating_sub(4)];
    }

    let s = s.trim();

    // Clean common noisy suffixes
    let mut cleaned = s.to_string();
    let suffixes_to_remove = [
        " (current user)",
        " (x64)",
        " (x86)",
        " (64-bit)",
        " (32-bit)",
    ];
    for suffix in &suffixes_to_remove {
        if let Some(stripped) = cleaned.strip_suffix(suffix) {
            cleaned = stripped.to_string();
        }
    }
    cleaned
}

/// Normalize an app name into a key for deduplication
pub fn normalize_app_key(name: &str) -> String {
    let lower = clean_app_display_name(name).to_ascii_lowercase();

    // Preserve edition distinction like "visual studio code - insiders" or "sublime merge"
    let mut normalized = String::new();
    for c in lower.chars() {
        if c.is_alphanumeric() || c == '-' {
            normalized.push(c);
        } else if !normalized.ends_with(' ') {
            normalized.push(' ');
        }
    }
    normalized.trim().to_string()
}

/// Discover applications from Start Menu, Registry, and standard Program Files
pub fn discover_installed_applications() -> Vec<DiscoveredApp> {
    let mut raw_apps = Vec::new();

    // 1. Start Menu Shortcuts
    scan_start_menu_shortcuts(&mut raw_apps);

    // 2. Desktop Shortcuts
    scan_desktop_shortcuts(&mut raw_apps);

    // 3. Modern WindowsApps (Windows Terminal wt.exe, Notepad, etc.)
    scan_windows_apps_folder(&mut raw_apps);

    // 4. Windows Registry (Uninstall keys & App Paths)
    #[cfg(target_os = "windows")]
    {
        scan_registry_applications(&mut raw_apps);
        scan_registry_app_paths(&mut raw_apps);
    }

    // 5. Well-known application folders (e.g. Sublime Text, Scoop, etc.)
    scan_well_known_program_folders(&mut raw_apps);

    // 6. Intelligent Deduplication
    deduplicate_applications(raw_apps)
}

fn scan_start_menu_shortcuts(apps: &mut Vec<DiscoveredApp>) {
    let mut dirs_to_scan = Vec::new();

    if let Some(home) = dirs::home_dir() {
        let user_start_menu = home.join(r"AppData\Roaming\Microsoft\Windows\Start Menu\Programs");
        if user_start_menu.exists() {
            dirs_to_scan.push(user_start_menu);
        }
    }

    let common_start_menu = PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs");
    if common_start_menu.exists() {
        dirs_to_scan.push(common_start_menu);
    }

    for root in dirs_to_scan {
        for entry in WalkDir::new(&root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };

            if !file_name.to_ascii_lowercase().ends_with(".lnk") {
                continue;
            }

            if is_helper_or_documentation_shortcut(file_name) {
                continue;
            }

            let display_name = clean_app_display_name(file_name);
            if display_name.is_empty() {
                continue;
            }

            let shortcut_path_str = path.to_string_lossy().to_string();

            apps.push(DiscoveredApp {
                name: display_name.clone(),
                display_name,
                publisher: None,
                version: None,
                target_path: shortcut_path_str.clone(),
                shortcut_path: Some(shortcut_path_str.clone()),
                icon_path: shortcut_path_str,
                source: "Start Menu".to_string(),
            });
        }
    }
}

fn scan_desktop_shortcuts(apps: &mut Vec<DiscoveredApp>) {
    let mut desktop_dirs = Vec::new();

    if let Some(home) = dirs::home_dir() {
        let user_desktop = home.join("Desktop");
        if user_desktop.exists() {
            desktop_dirs.push(user_desktop);
        }
    }

    let public_desktop = PathBuf::from(r"C:\Users\Public\Desktop");
    if public_desktop.exists() {
        desktop_dirs.push(public_desktop);
    }

    for dir in desktop_dirs {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };

            if !file_name.to_ascii_lowercase().ends_with(".lnk") {
                continue;
            }

            if is_helper_or_documentation_shortcut(file_name) {
                continue;
            }

            let display_name = clean_app_display_name(file_name);
            if display_name.is_empty() {
                continue;
            }

            let shortcut_path_str = path.to_string_lossy().to_string();

            apps.push(DiscoveredApp {
                name: display_name.clone(),
                display_name,
                publisher: None,
                version: None,
                target_path: shortcut_path_str.clone(),
                shortcut_path: Some(shortcut_path_str.clone()),
                icon_path: shortcut_path_str,
                source: "Desktop".to_string(),
            });
        }
    }
}

fn scan_windows_apps_folder(apps: &mut Vec<DiscoveredApp>) {
    let mut win_apps_dir = None;
    if let Some(home) = dirs::home_dir() {
        let p = home.join(r"AppData\Local\Microsoft\WindowsApps");
        if p.exists() {
            win_apps_dir = Some(p);
        }
    }

    if let Some(dir) = win_apps_dir {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };

            let lower = file_name.to_ascii_lowercase();
            if !lower.ends_with(".exe") {
                continue;
            }

            // Exclude background host helpers
            if is_helper_or_internal_executable(&lower)
                || lower.contains("mcp")
                || lower.contains("host")
                || lower.contains("server")
                || lower.contains("centennial")
                || lower.contains("asus")
                || lower.starts_with("b9eced6f")
            {
                continue;
            }

            let (display_name, publisher) = match lower.as_str() {
                "wt.exe" => (
                    "Windows Terminal".to_string(),
                    Some("Microsoft Corporation".to_string()),
                ),
                "notepad.exe" => (
                    "Notepad".to_string(),
                    Some("Microsoft Corporation".to_string()),
                ),
                "mspaint.exe" | "pbrush.exe" => {
                    ("Paint".to_string(), Some("Microsoft Corporation".to_string()))
                }
                "snippingtool.exe" => (
                    "Snipping Tool".to_string(),
                    Some("Microsoft Corporation".to_string()),
                ),
                "ms-teams.exe" => (
                    "Microsoft Teams".to_string(),
                    Some("Microsoft Corporation".to_string()),
                ),
                "wsl.exe" => (
                    "Windows Subsystem for Linux".to_string(),
                    Some("Microsoft Corporation".to_string()),
                ),
                "ubuntu2404.exe" => ("Ubuntu 24.04".to_string(), Some("Canonical".to_string())),
                "bash.exe" => ("Bash".to_string(), None),
                "winget.exe" => (
                    "Windows Package Manager".to_string(),
                    Some("Microsoft Corporation".to_string()),
                ),
                "microsoftstore.exe" | "store.exe" => (
                    "Microsoft Store".to_string(),
                    Some("Microsoft Corporation".to_string()),
                ),
                _ => (
                    clean_app_display_name(file_name),
                    Some("Microsoft Corporation".to_string()),
                ),
            };

            let path_str = path.to_string_lossy().to_string();
            apps.push(DiscoveredApp {
                name: display_name.clone(),
                display_name,
                publisher,
                version: None,
                target_path: path_str.clone(),
                shortcut_path: None,
                icon_path: path_str,
                source: "Windows Apps".to_string(),
            });
        }
    }
}

fn scan_well_known_program_folders(apps: &mut Vec<DiscoveredApp>) {
    let mut program_dirs = vec![
        PathBuf::from(r"C:\Program Files"),
        PathBuf::from(r"C:\Program Files (x86)"),
    ];

    if let Some(home) = dirs::home_dir() {
        let local_programs = home.join(r"AppData\Local\Programs");
        if local_programs.exists() {
            program_dirs.push(local_programs);
        }
        let scoop_apps = home.join(r"scoop\apps");
        if scoop_apps.exists() {
            program_dirs.push(scoop_apps);
        }
    }

    for root in program_dirs {
        if !root.exists() {
            continue;
        }

        let subdirs = match std::fs::read_dir(&root) {
            Ok(s) => s,
            Err(_) => continue,
        };

        for subdir in subdirs.filter_map(|e| e.ok()) {
            let path = subdir.path();
            if !path.is_dir() {
                continue;
            }

            let folder_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };

            // Look for primary application executables in top-level app folder
            let entries = match std::fs::read_dir(&path) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for file_entry in entries.filter_map(|e| e.ok()) {
                let file_path = file_entry.path();
                if !file_path.is_file() {
                    continue;
                }

                let exe_name = match file_path.file_name().and_then(|n| n.to_str()) {
                    Some(n) => n,
                    None => continue,
                };

                if !exe_name.to_ascii_lowercase().ends_with(".exe") {
                    continue;
                }

                if is_helper_or_internal_executable(exe_name) {
                    continue;
                }

                let display_name = if folder_name.eq_ignore_ascii_case("Microsoft VS Code") {
                    "Visual Studio Code".to_string()
                } else if folder_name.eq_ignore_ascii_case("Sublime Text") {
                    "Sublime Text".to_string()
                } else if folder_name.to_ascii_lowercase().contains("netbeans") {
                    "Apache NetBeans".to_string()
                } else if folder_name.to_ascii_lowercase().contains("dbeaver") {
                    "DBeaver".to_string()
                } else {
                    folder_name.to_string()
                };

                let path_str = file_path.to_string_lossy().to_string();

                apps.push(DiscoveredApp {
                    name: display_name.clone(),
                    display_name,
                    publisher: None,
                    version: None,
                    target_path: path_str.clone(),
                    shortcut_path: None,
                    icon_path: path_str,
                    source: "Program Files".to_string(),
                });
                break; // One primary exe per app directory is sufficient
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod reg_ffi {
    use std::ffi::c_void;

    pub type HKEY = *mut c_void;
    pub type LSTATUS = i32;
    pub type DWORD = u32;

    pub const HKEY_LOCAL_MACHINE: HKEY = 0x80000002_usize as HKEY;
    pub const HKEY_CURRENT_USER: HKEY = 0x80000001_usize as HKEY;
    pub const KEY_READ: DWORD = 0x20019;
    pub const KEY_WOW64_32KEY: DWORD = 0x0200;
    pub const ERROR_SUCCESS: LSTATUS = 0;

    #[link(name = "advapi32")]
    extern "system" {
        pub fn RegOpenKeyExW(
            h_key: HKEY,
            lp_sub_key: *const u16,
            ul_options: DWORD,
            sam_desired: DWORD,
            phk_result: *mut HKEY,
        ) -> LSTATUS;

        pub fn RegEnumKeyExW(
            h_key: HKEY,
            dw_index: DWORD,
            lp_name: *mut u16,
            lpcch_name: *mut DWORD,
            lp_reserved: *mut DWORD,
            lp_class: *mut u16,
            lpcch_class: *mut DWORD,
            lpft_last_write_time: *mut c_void,
        ) -> LSTATUS;

        pub fn RegQueryValueExW(
            h_key: HKEY,
            lp_value_name: *const u16,
            lp_reserved: *mut DWORD,
            lp_type: *mut DWORD,
            lp_data: *mut u8,
            lpcb_data: *mut DWORD,
        ) -> LSTATUS;

        pub fn RegCloseKey(h_key: HKEY) -> LSTATUS;
    }

    pub fn to_wide_null(s: &str) -> Vec<u16> {
        let mut v: Vec<u16> = s.encode_utf16().collect();
        v.push(0);
        v
    }

    pub fn query_string_value(hkey: HKEY, val_name: &str) -> Option<String> {
        let val_w = to_wide_null(val_name);
        let mut val_type: DWORD = 0;
        let mut data_len: DWORD = 0;

        unsafe {
            let res = RegQueryValueExW(
                hkey,
                val_w.as_ptr(),
                std::ptr::null_mut(),
                &mut val_type,
                std::ptr::null_mut(),
                &mut data_len,
            );
            if res != ERROR_SUCCESS || data_len == 0 {
                return None;
            }

            let mut buf = vec![0u8; data_len as usize];
            let res = RegQueryValueExW(
                hkey,
                val_w.as_ptr(),
                std::ptr::null_mut(),
                &mut val_type,
                buf.as_mut_ptr(),
                &mut data_len,
            );
            if res != ERROR_SUCCESS {
                return None;
            }

            if val_type == 1 || val_type == 2 {
                let u16_slice: &[u16] = std::slice::from_raw_parts(
                    buf.as_ptr() as *const u16,
                    (data_len as usize) / 2,
                );
                let trimmed: Vec<u16> = u16_slice.iter().copied().take_while(|&c| c != 0).collect();
                String::from_utf16(&trimmed).ok()
            } else {
                None
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn scan_registry_applications(apps: &mut Vec<DiscoveredApp>) {
    use reg_ffi::*;

    let keys_to_scan = [
        (HKEY_CURRENT_USER, r"Software\Microsoft\Windows\CurrentVersion\Uninstall", KEY_READ),
        (HKEY_LOCAL_MACHINE, r"Software\Microsoft\Windows\CurrentVersion\Uninstall", KEY_READ),
        (HKEY_LOCAL_MACHINE, r"Software\Microsoft\Windows\CurrentVersion\Uninstall", KEY_READ | KEY_WOW64_32KEY),
    ];

    for (root_hkey, subkey_path, access_mask) in keys_to_scan {
        let subkey_w = to_wide_null(subkey_path);
        let mut parent_key: HKEY = std::ptr::null_mut();

        unsafe {
            if RegOpenKeyExW(root_hkey, subkey_w.as_ptr(), 0, access_mask, &mut parent_key) != ERROR_SUCCESS {
                continue;
            }

            let mut index: DWORD = 0;
            let mut subkey_name_buf = [0u16; 256];

            loop {
                let mut name_len: DWORD = 256;
                let status = RegEnumKeyExW(
                    parent_key,
                    index,
                    subkey_name_buf.as_mut_ptr(),
                    &mut name_len,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );

                if status != ERROR_SUCCESS {
                    break;
                }
                index += 1;

                let child_subkey_name = match String::from_utf16(&subkey_name_buf[..name_len as usize]) {
                    Ok(n) => n,
                    Err(_) => continue,
                };

                let child_w = to_wide_null(&child_subkey_name);
                let mut child_key: HKEY = std::ptr::null_mut();
                if RegOpenKeyExW(parent_key, child_w.as_ptr(), 0, access_mask, &mut child_key) != ERROR_SUCCESS {
                    continue;
                }

                let display_name = query_string_value(child_key, "DisplayName");
                let publisher = query_string_value(child_key, "Publisher");
                let version = query_string_value(child_key, "DisplayVersion");
                let display_icon = query_string_value(child_key, "DisplayIcon");
                let install_loc = query_string_value(child_key, "InstallLocation");

                RegCloseKey(child_key);

                if let Some(raw_name) = display_name {
                    if raw_name.trim().is_empty() {
                        continue;
                    }

                    // Clean raw icon path (strip quotes and comma index like ,0)
                    let clean_icon_path = display_icon.as_ref().and_then(|icon| {
                        let trimmed = icon.trim().trim_matches('"');
                        let path_part = trimmed.split(',').next().unwrap_or(trimmed).trim();
                        if Path::new(path_part).exists() {
                            Some(path_part.to_string())
                        } else {
                            None
                        }
                    });

                    // Determine target executable path
                    let target_path = clean_icon_path.clone().or_else(|| {
                        install_loc.as_ref().map(|loc| loc.trim().trim_matches('"').to_string())
                    });

                    let target = match target_path {
                        Some(t) if !t.is_empty() => t,
                        _ => continue,
                    };

                    let clean_name = clean_app_display_name(&raw_name);

                    apps.push(DiscoveredApp {
                        name: clean_name.clone(),
                        display_name: clean_name,
                        publisher,
                        version,
                        target_path: target.clone(),
                        shortcut_path: None,
                        icon_path: clean_icon_path.unwrap_or(target),
                        source: "Registry".to_string(),
                    });
                }
            }

            RegCloseKey(parent_key);
        }
    }
}

#[cfg(target_os = "windows")]
fn scan_registry_app_paths(apps: &mut Vec<DiscoveredApp>) {
    use reg_ffi::*;

    let subkey_w = to_wide_null(r"Software\Microsoft\Windows\CurrentVersion\App Paths");
    let mut parent_key: HKEY = std::ptr::null_mut();

    unsafe {
        if RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey_w.as_ptr(), 0, KEY_READ, &mut parent_key) != ERROR_SUCCESS {
            return;
        }

        let mut index: DWORD = 0;
        let mut subkey_name_buf = [0u16; 256];

        loop {
            let mut name_len: DWORD = 256;
            let status = RegEnumKeyExW(
                parent_key,
                index,
                subkey_name_buf.as_mut_ptr(),
                &mut name_len,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );

            if status != ERROR_SUCCESS {
                break;
            }
            index += 1;

            let child_name = match String::from_utf16(&subkey_name_buf[..name_len as usize]) {
                Ok(n) => n,
                Err(_) => continue,
            };

            let child_w = to_wide_null(&child_name);
            let mut child_key: HKEY = std::ptr::null_mut();
            if RegOpenKeyExW(parent_key, child_w.as_ptr(), 0, KEY_READ, &mut child_key) != ERROR_SUCCESS {
                continue;
            }

            let default_val = query_string_value(child_key, "");
            RegCloseKey(child_key);

            if let Some(target_path) = default_val {
                let clean_target = target_path.trim().trim_matches('"').to_string();
                if clean_target.is_empty() || !Path::new(&clean_target).exists() {
                    continue;
                }

                if is_helper_or_internal_executable(&child_name) {
                    continue;
                }

                let display_name = if child_name.eq_ignore_ascii_case("code.exe") {
                    "Visual Studio Code".to_string()
                } else if child_name.eq_ignore_ascii_case("subl.exe") {
                    "Sublime Text".to_string()
                } else if child_name.eq_ignore_ascii_case("dbeaver.exe") {
                    "DBeaver".to_string()
                } else if child_name.eq_ignore_ascii_case("chrome.exe") {
                    "Google Chrome".to_string()
                } else {
                    clean_app_display_name(&child_name)
                };

                apps.push(DiscoveredApp {
                    name: display_name.clone(),
                    display_name,
                    publisher: None,
                    version: None,
                    target_path: clean_target.clone(),
                    shortcut_path: None,
                    icon_path: clean_target,
                    source: "App Paths".to_string(),
                });
            }
        }

        RegCloseKey(parent_key);
    }
}

/// Deduplicate applications while preserving genuine distinct editions
fn deduplicate_applications(raw_apps: Vec<DiscoveredApp>) -> Vec<DiscoveredApp> {
    let mut map: HashMap<String, DiscoveredApp> = HashMap::new();

    for app in raw_apps {
        let key = normalize_app_key(&app.name);
        if key.is_empty() {
            continue;
        }

        match map.get_mut(&key) {
            Some(existing) => {
                // Merge metadata: prefer richer info
                if existing.publisher.is_none() && app.publisher.is_some() {
                    existing.publisher = app.publisher;
                }
                if existing.version.is_none() && app.version.is_some() {
                    existing.version = app.version;
                }
                // Prefer shortcut if available, else executable
                if existing.shortcut_path.is_none() && app.shortcut_path.is_some() {
                    existing.shortcut_path = app.shortcut_path;
                }
                // Prefer existing name if shorter/cleaner
                if app.display_name.len() < existing.display_name.len()
                    && !app.display_name.to_lowercase().ends_with(".exe")
                {
                    existing.display_name = app.display_name;
                    existing.name = app.name;
                }
            }
            None => {
                map.insert(key, app);
            }
        }
    }

    let mut result: Vec<DiscoveredApp> = map.into_values().collect();
    // Sort applications by name
    result.sort_by(|a, b| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_executables_filtered() {
        assert!(is_helper_or_internal_executable("unins000.exe"));
        assert!(is_helper_or_internal_executable("uninstall.exe"));
        assert!(is_helper_or_internal_executable("helper.exe"));
        assert!(is_helper_or_internal_executable("crashpad_handler.exe"));
        assert!(is_helper_or_internal_executable("crash_handler.exe"));
        assert!(is_helper_or_internal_executable("update_installer.exe"));
        assert!(is_helper_or_internal_executable("plugin_host-3.14.exe"));
        assert!(is_helper_or_internal_executable("vcredist.exe"));

        // User-facing apps should NOT be filtered
        assert!(!is_helper_or_internal_executable("Code.exe"));
        assert!(!is_helper_or_internal_executable("sublime_text.exe"));
        assert!(!is_helper_or_internal_executable("dbeaver.exe"));
        assert!(!is_helper_or_internal_executable("netbeans64.exe"));
        assert!(!is_helper_or_internal_executable("chrome.exe"));
    }

    #[test]
    fn test_helper_shortcuts_filtered() {
        assert!(is_helper_or_documentation_shortcut("Uninstall.lnk"));
        assert!(is_helper_or_documentation_shortcut("Uninstall 4DDiG.lnk"));
        assert!(is_helper_or_documentation_shortcut("7-Zip Help.lnk"));
        assert!(is_helper_or_documentation_shortcut("Documentation.lnk"));
        assert!(is_helper_or_documentation_shortcut("Readme.lnk"));
        assert!(is_helper_or_documentation_shortcut("Website.lnk"));

        // Legitimate apps allowed
        assert!(!is_helper_or_documentation_shortcut("Visual Studio Code.lnk"));
        assert!(!is_helper_or_documentation_shortcut("Sublime Text.lnk"));
        assert!(!is_helper_or_documentation_shortcut("Apache NetBeans.lnk"));
        assert!(!is_helper_or_documentation_shortcut("DBeaver.lnk"));
    }

    #[test]
    fn test_app_name_cleaning() {
        assert_eq!(clean_app_display_name("Visual Studio Code.lnk"), "Visual Studio Code");
        assert_eq!(clean_app_display_name("DBeaver 26.1.0 (current user)"), "DBeaver 26.1.0");
        assert_eq!(clean_app_display_name("PowerToys (Preview) x64"), "PowerToys (Preview) x64");
    }

    #[test]
    fn test_deduplication_and_distinct_editions() {
        let raw = vec![
            DiscoveredApp {
                name: "Visual Studio Code.lnk".to_string(),
                display_name: "Visual Studio Code".to_string(),
                publisher: None,
                version: None,
                target_path: "C:/shortcuts/vscode.lnk".to_string(),
                shortcut_path: Some("C:/shortcuts/vscode.lnk".to_string()),
                icon_path: "C:/shortcuts/vscode.lnk".to_string(),
                source: "Start Menu".to_string(),
            },
            DiscoveredApp {
                name: "Visual Studio Code".to_string(),
                display_name: "Visual Studio Code".to_string(),
                publisher: Some("Microsoft Corporation".to_string()),
                version: Some("1.93.0".to_string()),
                target_path: "C:/Programs/VSCode/Code.exe".to_string(),
                shortcut_path: None,
                icon_path: "C:/Programs/VSCode/Code.exe".to_string(),
                source: "Registry".to_string(),
            },
            // Genuine distinct edition: Insiders!
            DiscoveredApp {
                name: "Visual Studio Code - Insiders".to_string(),
                display_name: "Visual Studio Code - Insiders".to_string(),
                publisher: Some("Microsoft Corporation".to_string()),
                version: Some("1.94.0-insider".to_string()),
                target_path: "C:/Programs/VSCode Insiders/Code - Insiders.exe".to_string(),
                shortcut_path: None,
                icon_path: "C:/Programs/VSCode Insiders/Code - Insiders.exe".to_string(),
                source: "Registry".to_string(),
            },
        ];

        let deduplicated = deduplicate_applications(raw);
        assert_eq!(deduplicated.len(), 2);

        let vscode = deduplicated.iter().find(|a| a.display_name == "Visual Studio Code").unwrap();
        assert_eq!(vscode.publisher.as_deref(), Some("Microsoft Corporation"));
        assert!(vscode.shortcut_path.is_some());

        let insiders = deduplicated.iter().find(|a| a.display_name == "Visual Studio Code - Insiders").unwrap();
        assert_eq!(insiders.publisher.as_deref(), Some("Microsoft Corporation"));
    }
}

