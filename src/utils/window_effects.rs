#[cfg(target_os = "windows")]
mod win_effects {
    use std::ffi::c_void;

    pub type HWND = *mut c_void;
    pub type BOOL = i32;
    pub type DWORD = u32;
    pub type HRESULT = i32;

    const DWMWA_USE_IMMERSIVE_DARK_MODE: DWORD = 20;
    const DWMWA_WINDOW_CORNER_PREFERENCE: DWORD = 33;
    const DWMWCP_ROUND: DWORD = 2; // Modern rounded window corners (matches PowerToys Run)
    const DWMWA_BORDER_COLOR: DWORD = 34;

    const GWL_EXSTYLE: i32 = -20;
    const WS_EX_TOOLWINDOW: u32 = 0x00000080;
    const WS_EX_APPWINDOW: u32 = 0x00040000;

    #[repr(C)]
    struct MARGINS {
        cx_left_width: i32,
        cx_right_width: i32,
        cy_top_height: i32,
        cy_bottom_height: i32,
    }

    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: HWND,
            dw_attribute: DWORD,
            pv_attribute: *const c_void,
            cb_attribute: DWORD,
        ) -> HRESULT;
        fn DwmExtendFrameIntoClientArea(hwnd: HWND, p_mar_inset: *const MARGINS) -> HRESULT;
    }

    #[link(name = "user32")]
    extern "system" {
        fn GetWindowLongW(hwnd: HWND, n_index: i32) -> i32;
        fn SetWindowLongW(hwnd: HWND, n_index: i32, dw_new_long: i32) -> i32;
    }

    pub fn apply_glass_effect(hwnd: HWND, _enable_glass: bool) {
        if hwnd.is_null() {
            return;
        }

        unsafe {
            // 1. Apply crisp immersive dark mode
            let dark_mode: BOOL = 1;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                &dark_mode as *const _ as *const c_void,
                std::mem::size_of::<BOOL>() as DWORD,
            );

            // 2. Apply native OS rounded window corners (Windows 11 DWM corner rounding)
            let corner_pref: DWORD = DWMWCP_ROUND;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &corner_pref as *const _ as *const c_void,
                std::mem::size_of::<DWORD>() as DWORD,
            );

            // 3. Subtle dark border (COLORREF 0x003A2E2A -> RGB(42, 46, 58))
            let border_color: DWORD = 0x003A2E2A;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_BORDER_COLOR,
                &border_color as *const _ as *const c_void,
                std::mem::size_of::<DWORD>() as DWORD,
            );

            // 4. Extend frame into client area by 1px so DWM dropshadow and round border render perfectly
            let margins = MARGINS {
                cx_left_width: 0,
                cx_right_width: 0,
                cy_top_height: 1,
                cy_bottom_height: 0,
            };
            let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);
        }
    }

    pub fn set_taskbar_presence(hwnd: HWND, show_in_taskbar: bool) {
        if hwnd.is_null() {
            return;
        }
        unsafe {
            let cur_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
            let new_style = if show_in_taskbar {
                (cur_style & !WS_EX_TOOLWINDOW) | WS_EX_APPWINDOW
            } else {
                (cur_style & !WS_EX_APPWINDOW) | WS_EX_TOOLWINDOW
            };
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_style as i32);
        }
    }
}

pub fn apply_native_glass_to_window(enable_glass: bool) {
    #[cfg(target_os = "windows")]
    {
        if let Some(hwnd) = crate::utils::tray_hotkey::find_searchforge_window() {
            win_effects::apply_glass_effect(hwnd, enable_glass);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = enable_glass;
    }
}

pub fn set_window_taskbar_presence(show_in_taskbar: bool) {
    #[cfg(target_os = "windows")]
    {
        if let Some(hwnd) = crate::utils::tray_hotkey::find_searchforge_window() {
            win_effects::set_taskbar_presence(hwnd, show_in_taskbar);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = show_in_taskbar;
    }
}

pub fn is_searchforge_focused() -> bool {
    #[cfg(target_os = "windows")]
    {
        if let Some(hwnd) = crate::utils::tray_hotkey::find_searchforge_window() {
            unsafe {
                let fg = crate::utils::tray_hotkey::win32::GetForegroundWindow();
                return fg == hwnd;
            }
        }
        false
    }
    #[cfg(not(target_os = "windows"))]
    {
        true
    }
}
