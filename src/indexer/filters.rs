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
        "miniconda3".to_string(),
        "anaconda3".to_string(),
        "venv".to_string(),
        ".venv".to_string(),
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
        // Build & bundler output directories
        "dist".to_string(),
        "build".to_string(),
        "out".to_string(),
        ".next".to_string(),
        ".nuxt".to_string(),
        ".turbo".to_string(),
        ".svelte-kit".to_string(),
        "vendor".to_string(),
        "site-packages".to_string(),
        "__pycache__".to_string(),
        "coverage".to_string(),
        ".nyc_output".to_string(),
        ".angular".to_string(),
        ".dart_tool".to_string(),
        ".parcel-cache".to_string(),
        "package-cache".to_string(),
        "chunks".to_string(),
    ]
}

/// Binary / compiled / cache / download extensions that clutter search results
pub const IGNORED_EXTENSIONS: &[&str] = &[
    "dll", "exe", "sys", "bin", "dat", "idx", "pack", "pak", "node", "o", "a",
    "pyc", "class", "blob", "ldb", "tmp", "lock", "pdb", "lib", "obj", "dylib",
    "so", "log", "etl", "evtx", "swp", "bak", "etag", "cache", "crdownload",
    "msi", "cab", "vmdk", "iso", "qcow2", "dmg", "jar", "war", "ear",
    "download", "dow", "part", "map", "chunk", "bundle",
];

/// Identify internal build chunks, package downloads, and hash-named assets
pub fn is_build_artifact_or_chunk(file_name: &str) -> bool {
    let lower = file_name.to_ascii_lowercase();

    // Incomplete or cached package downloads & source maps
    if lower.ends_with(".download")
        || lower.ends_with(".dow")
        || lower.ends_with(".part")
        || lower.ends_with(".crdownload")
        || lower.ends_with(".map")
    {
        return true;
    }

    // Framework internal manifests and chunks
    if lower.starts_with("_buildmanifest")
        || lower.starts_with("_ssgmanifest")
        || lower.starts_with("_app-")
        || lower.starts_with("cb=gapi.")
        || lower.starts_with("saved_resource")
    {
        return true;
    }

    // Extract stem (before first period)
    let stem = if let Some(idx) = lower.find('.') {
        &lower[..idx]
    } else {
        &lower
    };

    // Hexadecimal chunk stems: e.g. "04589bf55e9fa033", "2c1dff654a517a91", "9c6bb7a9b0161055", "abe6f3225454a36c"
    if stem.len() >= 12 && stem.chars().all(|c| c.is_ascii_hexdigit()) {
        return true;
    }

    // Chunks with hashes like "all-34eb6e0403d1b891", "main-eb8a9b767611e4db", "create-4db705dd8b0e73cf"
    if let Some(pos) = stem.rfind('-') {
        let suffix = &stem[pos + 1..];
        if suffix.len() >= 12 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
            return true;
        }
    }

    false
}

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

    // Ignore browser saved webpage resource folders (e.g. "The Conceptual Search Structure_files")
    for segment in normalized.split('/') {
        let seg_lower = segment.to_ascii_lowercase();
        if seg_lower.ends_with("_files") || seg_lower.ends_with("_data") {
            return true;
        }
    }

    // Check if filename is a hidden dot-file/folder (e.g. .git, .DS_Store)
    if let Some(file_name) = Path::new(path_str).file_name().and_then(|n| n.to_str()) {
        if file_name.starts_with('.') && file_name.len() > 1 && file_name != ".env" {
            return true;
        }

        // Check if file is a package download, build chunk, or hash asset
        if is_build_artifact_or_chunk(file_name) {
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
    fn test_build_artifacts_and_chunks_ignored() {
        let ignored = default_ignored_patterns();
        // Webpage saved files folder
        assert!(should_ignore("A:\\Master class\\Semester 4\\figures\\The Conceptual Search Structure_files\\04589bf55e9fa033.css", &ignored));
        assert!(should_ignore("A:\\Master class\\Semester 4\\figures\\The Conceptual Search Structure_files\\virgil-v2.css", &ignored));
        
        // Hexadecimal chunk stems
        assert!(should_ignore("C:\\Projects\\app\\9c6bb7a9b0161055.css", &ignored));
        assert!(should_ignore("C:\\Projects\\app\\abe6f3225454a36c.css", &ignored));
        assert!(should_ignore("C:\\Projects\\app\\2c1dff654a517a91.css", &ignored));

        // Hashed bundle and download files
        assert!(should_ignore("C:\\Projects\\all-34eb6e0403d1b891.js.download", &ignored));
        assert!(should_ignore("C:\\Projects\\main-eb8a9b767611e4db.js.download", &ignored));
        assert!(should_ignore("C:\\Projects\\_buildManifest.js.download", &ignored));
        assert!(should_ignore("C:\\Projects\\_app-efa5b14fa865b400.js.download", &ignored));

        // Build directories
        assert!(should_ignore("C:\\Projects\\.next\\static\\css\\style.css", &ignored));
        assert!(should_ignore("C:\\Projects\\dist\\bundle.js", &ignored));
        assert!(should_ignore("C:\\Projects\\build\\main.js", &ignored));
    }

    #[test]
    fn test_legitimate_files_allowed() {
        let ignored = default_ignored_patterns();
        assert!(!should_ignore("C:\\Users\\User\\Downloads\\invoice_99.pdf", &ignored));
        assert!(!should_ignore("C:\\Users\\User\\Documents\\budget.xlsx", &ignored));
        assert!(!should_ignore("C:\\Users\\User\\Desktop\\notes.md", &ignored));
        assert!(!should_ignore("C:\\Users\\User\\Projects\\main.rs", &ignored));
        assert!(!should_ignore("C:\\Users\\User\\Projects\\style.css", &ignored));
        assert!(!should_ignore("C:\\Users\\User\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Visual Studio Code.lnk", &ignored));
        assert!(!should_ignore("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\DBeaver.lnk", &ignored));
    }
}
