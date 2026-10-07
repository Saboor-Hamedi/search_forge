fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("src/assets/icons/image.ico");
        res.set("ProductName", "SearchForge");
        res.set("FileDescription", "SearchForge - Instant Desktop Search");
        res.set("LegalCopyright", "Copyright (C) 2026 SearchForge");
        let _ = res.compile();
    }
}
