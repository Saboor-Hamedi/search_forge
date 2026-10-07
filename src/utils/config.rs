use std::path::PathBuf;

const SETTINGS_FILE_NAME: &str = "settings.json";

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub autostart: bool,
    pub hide_on_close: bool,
    pub spotlight_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            autostart: true,
            hide_on_close: true,
            spotlight_mode: true, // Spotlight mode on by default
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("SearchForge");
            let _ = std::fs::create_dir_all(&app_dir);
            app_dir.join(SETTINGS_FILE_NAME)
        } else {
            PathBuf::from(SETTINGS_FILE_NAME)
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<Self>(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, content);
        }
    }

    pub fn set_spotlight_mode(&mut self, enable: bool) {
        self.spotlight_mode = enable;
        self.autostart = enable;
        self.hide_on_close = enable;
        self.save();
        Self::sync_system_autostart(enable);
    }

    #[allow(dead_code)]
    pub fn set_autostart(&mut self, enable: bool) {
        self.autostart = enable;
        self.save();
        Self::sync_system_autostart(enable);
    }

    #[allow(dead_code)]
    pub fn set_hide_on_close(&mut self, enable: bool) {
        self.hide_on_close = enable;
        self.save();
    }

    pub fn sync_system_autostart(enable: bool) {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            if let Ok(exe_path) = std::env::current_exe() {
                let exe_str = exe_path.to_string_lossy().to_string();
                if enable {
                    let cmd = format!(
                        "Set-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' -Name 'SearchForge' -Value '\"{}\"'",
                        exe_str
                    );
                    let _ = std::process::Command::new("powershell")
                        .args(["-NoProfile", "-Command", &cmd])
                        .creation_flags(0x08000000) // CREATE_NO_WINDOW
                        .output();
                } else {
                    let cmd = "Remove-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' -Name 'SearchForge' -ErrorAction SilentlyContinue";
                    let _ = std::process::Command::new("powershell")
                        .args(["-NoProfile", "-Command", cmd])
                        .creation_flags(0x08000000) // CREATE_NO_WINDOW
                        .output();
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = enable;
        }
    }
}
