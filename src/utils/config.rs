use std::path::PathBuf;

const SETTINGS_FILE_NAME: &str = "settings.json";

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Glass,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Glass
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub autostart: bool,
    pub hide_on_close: bool,
    pub spotlight_mode: bool,
    #[serde(default)]
    pub theme: ThemeMode,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            autostart: true,
            hide_on_close: true,
            spotlight_mode: true,
            theme: ThemeMode::Glass,
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

    #[allow(dead_code)]
    pub fn set_spotlight_mode(&mut self, enable: bool) {
        self.spotlight_mode = enable;
        self.autostart = enable;
        self.hide_on_close = enable;
        self.save();
        Self::sync_system_autostart(enable);
    }

    pub fn set_autostart(&mut self, enable: bool) {
        self.autostart = enable;
        self.save();
        Self::sync_system_autostart(enable);
    }

    pub fn set_theme(&mut self, theme: ThemeMode) {
        self.theme = theme;
        self.save();
    }

    #[allow(dead_code)]
    pub fn set_hide_on_close(&mut self, enable: bool) {
        self.hide_on_close = enable;
        self.save();
    }

    pub fn resolve_autostart_exe_path() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            if let Ok(exe_path) = std::env::current_exe() {
                let exe_lower = exe_path.to_string_lossy().to_ascii_lowercase();
                // If running from cargo target/debug or target/release, prefer the installed production binary if present
                if exe_lower.contains("\\target\\debug\\") || exe_lower.contains("\\target\\release\\") {
                    if let Some(local_data) = dirs::data_local_dir() {
                        let user_installed = local_data.join("Programs").join("SearchForge").join("search_forge.exe");
                        if user_installed.is_file() {
                            return Some(user_installed);
                        }
                    }
                    let pf_installed = PathBuf::from(r"C:\Program Files\SearchForge\search_forge.exe");
                    if pf_installed.is_file() {
                        return Some(pf_installed);
                    }
                }
                return Some(exe_path);
            }
            None
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::env::current_exe().ok()
        }
    }

    pub fn sync_system_autostart(enable: bool) {
        #[cfg(target_os = "windows")]
        {
            use std::ffi::c_void;
            use std::os::windows::ffi::OsStrExt;

            type HKEY = *mut c_void;
            type DWORD = u32;
            type LSTATUS = i32;

            const HKEY_CURRENT_USER: HKEY = 0x80000001_usize as HKEY;
            const KEY_SET_VALUE: DWORD = 0x0002;
            const REG_SZ: DWORD = 1;
            const ERROR_SUCCESS: LSTATUS = 0;

            #[link(name = "advapi32")]
            extern "system" {
                fn RegCreateKeyExW(
                    h_key: HKEY,
                    lp_sub_key: *const u16,
                    reserved: DWORD,
                    lp_class: *const u16,
                    dw_options: DWORD,
                    sam_desired: DWORD,
                    lp_security_attributes: *const c_void,
                    phk_result: *mut HKEY,
                    lpdw_disposition: *mut DWORD,
                ) -> LSTATUS;
                fn RegSetValueExW(
                    h_key: HKEY,
                    lp_value_name: *const u16,
                    reserved: DWORD,
                    dw_type: DWORD,
                    lp_data: *const u8,
                    cb_data: DWORD,
                ) -> LSTATUS;
                fn RegDeleteValueW(h_key: HKEY, lp_value_name: *const u16) -> LSTATUS;
                fn RegCloseKey(h_key: HKEY) -> LSTATUS;
            }

            let subkey: Vec<u16> = std::ffi::OsStr::new(r"Software\Microsoft\Windows\CurrentVersion\Run")
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let val_name: Vec<u16> = std::ffi::OsStr::new("SearchForge")
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            unsafe {
                let mut hkey: HKEY = std::ptr::null_mut();
                if RegCreateKeyExW(
                    HKEY_CURRENT_USER,
                    subkey.as_ptr(),
                    0,
                    std::ptr::null(),
                    0,
                    KEY_SET_VALUE,
                    std::ptr::null(),
                    &mut hkey,
                    std::ptr::null_mut(),
                ) == ERROR_SUCCESS
                {
                    if enable {
                        if let Some(exe_path) = Self::resolve_autostart_exe_path() {
                            let reg_val = format!("\"{}\" --autostart", exe_path.to_string_lossy());
                            let val_wide: Vec<u16> = std::ffi::OsStr::new(&reg_val)
                                .encode_wide()
                                .chain(std::iter::once(0))
                                .collect();
                            let _ = RegSetValueExW(
                                hkey,
                                val_name.as_ptr(),
                                0,
                                REG_SZ,
                                val_wide.as_ptr() as *const u8,
                                (val_wide.len() * 2) as DWORD,
                            );
                        }
                    } else {
                        let _ = RegDeleteValueW(hkey, val_name.as_ptr());
                    }
                    let _ = RegCloseKey(hkey);
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = enable;
        }
    }
}
