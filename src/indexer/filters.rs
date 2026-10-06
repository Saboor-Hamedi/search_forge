use std::path::Path;

/// Folder patterns to skip during indexing
pub fn default_ignored_patterns() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        ".git".to_string(),
        "target".to_string(),
        "AppData".to_string(),
        ".cache".to_string(),
        "$RECYCLE.BIN".to_string(),
        ".cargo".to_string(),
        ".rustup".to_string(),
        ".vscode".to_string(),
        ".idea".to_string(),
        ".gradle".to_string(),
        "Temp".to_string(),
        "System Volume Information".to_string(),
        "Windows".to_string(),
        "Program Files".to_string(),
        "Program Files (x86)".to_string(),
        "ProgramData".to_string(),
    ]
}

/// Binary / compiled / cache extensions that clutter search results with random numbers/hashes
pub const IGNORED_EXTENSIONS: &[&str] = &[
    "dll", "exe", "sys", "bin", "dat", "idx", "pack", "pak", "node", "o", "a",
    "pyc", "class", "blob", "ldb", "tmp", "lock", "pdb", "lib", "obj", "dylib",
    "so", "log", "etl", "evtx", "swp", "bak", "etag", "cache", "crdownload",
    "msi", "cab", "vmdk", "iso", "qcow2", "dmg", "jar", "war", "ear"
];

pub fn should_ignore(path_str: &str, ignored: &[String]) -> bool {
    let normalized = path_str.replace('\\', "/");
    
    let is_start_menu = normalized.to_ascii_lowercase().contains("start menu");

    // Check ignored folder patterns
    for pattern in ignored {
        let trimmed = pattern.trim_matches('/');
        if trimmed.is_empty() {
            continue;
        }

        // Allow Windows Start Menu applications
        if is_start_menu && (
            trimmed.eq_ignore_ascii_case("AppData")
            || trimmed.eq_ignore_ascii_case("ProgramData")
            || trimmed.eq_ignore_ascii_case("Windows")
            || trimmed.eq_ignore_ascii_case("Program Files")
            || trimmed.eq_ignore_ascii_case("Program Files (x86)")
        ) {
            continue;
        }

        for segment in normalized.split('/') {
            if segment.eq_ignore_ascii_case(trimmed) {
                return true;
            }
        }
        if normalized.contains(pattern.as_str()) {
            return true;
        }
    }

    // Check if filename is a hidden dot-file/folder (e.g. .git, .DS_Store)
    if let Some(file_name) = Path::new(path_str).file_name().and_then(|n| n.to_str()) {
        if file_name.starts_with('.') && file_name.len() > 1 && file_name != ".env" {
            return true;
        }

        // Check if extension is binary/cache garbage (allow .lnk application shortcuts)
        if let Some(ext) = Path::new(path_str).extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            if ext_lower != "lnk" && IGNORED_EXTENSIONS.contains(&ext_lower.as_str()) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_garbage_ignored() {
        let ignored = default_ignored_patterns();
        assert!(should_ignore("C:\\Users\\User\\Downloads\\chunk_99.bin", &ignored));
        assert!(should_ignore("C:\\Users\\User\\Downloads\\driver.sys", &ignored));
        assert!(should_ignore("C:\\Users\\User\\Downloads\\lib.dll", &ignored));
        assert!(should_ignore("C:\\Users\\User\\AppData\\Local\\test.txt", &ignored));
        assert!(should_ignore("C:\\Users\\User\\.git\\HEAD", &ignored));
    }

    #[test]
    fn test_legitimate_files_allowed() {
        let ignored = default_ignored_patterns();
        assert!(!should_ignore("C:\\Users\\User\\Downloads\\invoice_99.pdf", &ignored));
        assert!(!should_ignore("C:\\Users\\User\\Documents\\budget.xlsx", &ignored));
        assert!(!should_ignore("C:\\Users\\User\\Desktop\\notes.md", &ignored));
        assert!(!should_ignore("C:\\Users\\User\\Projects\\main.rs", &ignored));
        assert!(!should_ignore("C:\\Users\\User\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Visual Studio Code.lnk", &ignored));
        assert!(!should_ignore("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\DBeaver.lnk", &ignored));
    }
}
