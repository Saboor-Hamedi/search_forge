use egui::Context;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    Show,
    Hide,
    Quit,
}

pub struct TrayHotkeyManager {
    receiver: Receiver<TrayEvent>,
}

impl TrayHotkeyManager {
    pub fn new(ctx: Context) -> Self {
        let (tx, rx) = channel();
        start_tray_and_hotkey_listener(tx, ctx);
        Self { receiver: rx }
    }

    pub fn try_recv(&self) -> Option<TrayEvent> {
        self.receiver.try_recv().ok()
    }
}

#[cfg(target_os = "windows")]
pub fn find_searchforge_window() -> Option<win32::HWND> {
    use std::os::windows::ffi::OsStrExt;
    let wide_title: Vec<u16> = std::ffi::OsStr::new("SearchForge")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let hwnd = unsafe { win32::FindWindowW(std::ptr::null(), wide_title.as_ptr()) };
    if !hwnd.is_null() {
        Some(hwnd)
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
pub fn bring_window_to_foreground(hwnd: win32::HWND) {
    unsafe {
        if let Some(ref mut state) = GLOBAL_TRAY_STATE {
            state.is_visible = true;
        }
        win32::ShowWindow(hwnd, win32::SW_RESTORE);
        win32::ShowWindow(hwnd, win32::SW_SHOW);
        win32::SetForegroundWindow(hwnd);
        win32::BringWindowToTop(hwnd);
        win32::SetFocus(hwnd);
    }
}

#[cfg(target_os = "windows")]
pub fn hide_searchforge_window(hwnd: win32::HWND) {
    unsafe {
        if let Some(ref mut state) = GLOBAL_TRAY_STATE {
            state.is_visible = false;
        }
        win32::ShowWindow(hwnd, win32::SW_HIDE);
    }
}

#[cfg(target_os = "windows")]
fn start_tray_and_hotkey_listener(sender: Sender<TrayEvent>, ctx: Context) {
    std::thread::Builder::new()
        .name("tray_hotkey_worker".to_string())
        .spawn(move || {
            unsafe {
                run_win32_tray_and_hotkey(sender, ctx);
            }
        })
        .ok();
}

#[cfg(target_os = "windows")]
#[cfg(target_os = "windows")]
fn load_tray_hicon() -> win32::HICON {
    let ico_bytes = include_bytes!("../assets/icons/image.ico");
    if ico_bytes.len() >= 6 {
        let count = u16::from_le_bytes([ico_bytes[4], ico_bytes[5]]) as usize;
        let mut best_offset = 0;
        let mut best_size = 0;
        let mut best_diff = i32::MAX;
        let target_size = 16;
        for i in 0..count {
            let entry_offset = 6 + i * 16;
            if entry_offset + 16 <= ico_bytes.len() {
                let w = match ico_bytes[entry_offset] {
                    0 => 256,
                    other => other as i32,
                };
                let bytes_in_res = u32::from_le_bytes([
                    ico_bytes[entry_offset + 8],
                    ico_bytes[entry_offset + 9],
                    ico_bytes[entry_offset + 10],
                    ico_bytes[entry_offset + 11],
                ]) as usize;
                let img_offset = u32::from_le_bytes([
                    ico_bytes[entry_offset + 12],
                    ico_bytes[entry_offset + 13],
                    ico_bytes[entry_offset + 14],
                    ico_bytes[entry_offset + 15],
                ]) as usize;
                let diff = (w - target_size).abs();
                if diff < best_diff && img_offset + bytes_in_res <= ico_bytes.len() {
                    best_diff = diff;
                    best_offset = img_offset;
                    best_size = bytes_in_res;
                }
            }
        }
        if best_size > 0 {
            let res_bytes = &ico_bytes[best_offset..best_offset + best_size];
            let hicon = unsafe {
                win32::CreateIconFromResourceEx(
                    res_bytes.as_ptr(),
                    best_size as u32,
                    1,
                    0x00030000,
                    16,
                    16,
                    0,
                )
            };
            if !hicon.is_null() {
                return hicon;
            }
        }
    }

    // Fallback: try raw PNG bytes (supported natively on Vista+)
    let png_bytes = include_bytes!("../assets/icons/image_64.png");
    let hicon = unsafe {
        win32::CreateIconFromResourceEx(
            png_bytes.as_ptr(),
            png_bytes.len() as u32,
            1,
            0x00030000,
            16,
            16,
            0,
        )
    };
    if !hicon.is_null() {
        return hicon;
    }

    // Fallback: LoadIconW
    unsafe {
        let h_instance = win32::GetModuleHandleW(std::ptr::null());
        let h = win32::LoadIconW(h_instance, 1 as *const u16);
        if !h.is_null() {
            h
        } else {
            win32::LoadIconW(std::ptr::null_mut(), win32::IDI_APPLICATION)
        }
    }
}

#[cfg(target_os = "windows")]
unsafe fn run_win32_tray_and_hotkey(sender: Sender<TrayEvent>, ctx: Context) {
    use std::os::windows::ffi::OsStrExt;

    let class_name: Vec<u16> = std::ffi::OsStr::new("SearchForgeTrayMsgClass")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let h_instance = win32::GetModuleHandleW(std::ptr::null());

    let wnd_class = win32::WNDCLASSEXW {
        cb_size: std::mem::size_of::<win32::WNDCLASSEXW>() as u32,
        style: 0,
        lpfn_wnd_proc: tray_window_proc,
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance,
        h_icon: std::ptr::null_mut(),
        h_cursor: std::ptr::null_mut(),
        hbr_background: std::ptr::null_mut(),
        lpsz_menu_name: std::ptr::null(),
        lpsz_class_name: class_name.as_ptr(),
        h_icon_sm: std::ptr::null_mut(),
    };

    win32::RegisterClassExW(&wnd_class);

    // Create a hidden top-level window (HWND_MESSAGE does NOT receive global hotkeys or tray events reliably)
    let hwnd = win32::CreateWindowExW(
        0,
        class_name.as_ptr(),
        std::ptr::null(),
        0,
        0,
        0,
        0,
        0,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        h_instance,
        std::ptr::null_mut(),
    );

    // Store sender and ctx in global state for window proc
    GLOBAL_TRAY_STATE = Some(GlobalTrayState {
        sender: sender.clone(),
        ctx: ctx.clone(),
        tray_hwnd: hwnd,
        is_visible: true,
    });

    // Register Alt + K as system-wide global hotkey
    let hotkey_registered = win32::RegisterHotKey(
        hwnd,
        win32::HOTKEY_ID_ALT_K,
        win32::MOD_ALT | win32::MOD_NOREPEAT,
        win32::VK_K,
    ) != 0 || win32::RegisterHotKey(
        hwnd,
        win32::HOTKEY_ID_ALT_K,
        win32::MOD_ALT,
        win32::VK_K,
    ) != 0;

    if !hotkey_registered {
        eprintln!("Warning: Could not register global Alt+K hotkey (may already be in use by another app)");
    }

    // Load application icon from embedded icon resource
    let h_icon = load_tray_hicon();

    // Prepare system tray notification icon
    let mut tip: [u16; 128] = [0; 128];
    let tip_text: Vec<u16> = std::ffi::OsStr::new("SearchForge (Alt+K)")
        .encode_wide()
        .collect();
    for (i, &c) in tip_text.iter().take(127).enumerate() {
        tip[i] = c;
    }

    let mut nid: win32::NOTIFYICONDATAW = std::mem::zeroed();
    nid.cb_size = std::mem::size_of::<win32::NOTIFYICONDATAW>() as u32;
    nid.h_wnd = hwnd;
    nid.u_id = 1;
    nid.u_flags = win32::NIF_MESSAGE | win32::NIF_ICON | win32::NIF_TIP;
    nid.u_callback_message = win32::WM_TRAYICON;
    nid.h_icon = h_icon;
    nid.sz_tip = tip;

    win32::Shell_NotifyIconW(win32::NIM_ADD, &nid);

    // Standard Win32 Message Loop
    let mut msg: win32::MSG = std::mem::zeroed();
    while win32::GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
        win32::TranslateMessage(&msg);
        win32::DispatchMessageW(&msg);
    }

    // Cleanup tray icon on termination
    win32::UnregisterHotKey(hwnd, win32::HOTKEY_ID_ALT_K);
    win32::Shell_NotifyIconW(win32::NIM_DELETE, &nid);
}

#[cfg(target_os = "windows")]
struct GlobalTrayState {
    sender: Sender<TrayEvent>,
    ctx: Context,
    #[allow(dead_code)]
    tray_hwnd: win32::HWND,
    is_visible: bool,
}

#[cfg(target_os = "windows")]
static mut GLOBAL_TRAY_STATE: Option<GlobalTrayState> = None;

#[cfg(target_os = "windows")]
unsafe extern "system" fn tray_window_proc(
    hwnd: win32::HWND,
    msg: win32::UINT,
    w_param: win32::WPARAM,
    l_param: win32::LPARAM,
) -> win32::LRESULT {
    match msg {
        win32::WM_HOTKEY => {
            if w_param as i32 == win32::HOTKEY_ID_ALT_K {
                toggle_searchforge();
            }
            0
        }
        win32::WM_TRAYICON => {
            let event = l_param as win32::UINT;
            match event {
                win32::WM_LBUTTONUP | win32::WM_LBUTTONDBLCLK => {
                    show_searchforge();
                }
                win32::WM_RBUTTONUP => {
                    show_tray_context_menu(hwnd);
                }
                _ => {}
            }
            0
        }
        win32::WM_COMMAND => {
            let cmd_id = (w_param & 0xFFFF) as usize;
            if cmd_id == win32::MENU_ID_OPEN {
                show_searchforge();
            } else if cmd_id == win32::MENU_ID_QUIT {
                if let Some(ref state) = GLOBAL_TRAY_STATE {
                    let _ = state.sender.send(TrayEvent::Quit);
                    state.ctx.request_repaint();
                }
                std::process::exit(0);
            }
            0
        }
        win32::WM_DESTROY => {
            win32::PostQuitMessage(0);
            0
        }
        _ => win32::DefWindowProcW(hwnd, msg, w_param, l_param),
    }
}

#[cfg(target_os = "windows")]
unsafe fn toggle_searchforge() {
    let main_hwnd_opt = find_searchforge_window();
    if let Some(hwnd) = main_hwnd_opt {
        let is_visible = win32::IsWindowVisible(hwnd) != 0;
        let fg = win32::GetForegroundWindow();
        if is_visible && fg == hwnd {
            hide_searchforge();
            return;
        }
    }

    // Otherwise, pop up and focus SearchForge
    show_searchforge();
}

#[cfg(target_os = "windows")]
pub unsafe fn show_searchforge() {
    if let Some(ref mut state) = GLOBAL_TRAY_STATE {
        state.is_visible = true;
        let _ = state.sender.send(TrayEvent::Show);
        state.ctx.request_repaint();
    }
    // Directly restore and activate the HWND so winit processes the wakeup immediately
    if let Some(hwnd) = find_searchforge_window() {
        win32::ShowWindow(hwnd, win32::SW_RESTORE);
        win32::ShowWindow(hwnd, win32::SW_SHOW);
        win32::SetForegroundWindow(hwnd);
        win32::BringWindowToTop(hwnd);
        win32::SetFocus(hwnd);
    }
}

#[cfg(target_os = "windows")]
pub unsafe fn hide_searchforge() {
    if let Some(ref mut state) = GLOBAL_TRAY_STATE {
        state.is_visible = false;
        let _ = state.sender.send(TrayEvent::Hide);
        state.ctx.request_repaint();
    }
    if let Some(hwnd) = find_searchforge_window() {
        win32::ShowWindow(hwnd, win32::SW_HIDE);
    }
}

#[cfg(target_os = "windows")]
unsafe fn show_tray_context_menu(hwnd: win32::HWND) {
    use std::os::windows::ffi::OsStrExt;

    let h_menu = win32::CreatePopupMenu();
    if h_menu.is_null() {
        return;
    }

    let open_text: Vec<u16> = std::ffi::OsStr::new("Open SearchForge (Alt+K)")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let quit_text: Vec<u16> = std::ffi::OsStr::new("Quit SearchForge")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    win32::AppendMenuW(h_menu, 0, win32::MENU_ID_OPEN, open_text.as_ptr());
    win32::AppendMenuW(h_menu, win32::MF_SEPARATOR, 0, std::ptr::null());
    win32::AppendMenuW(h_menu, 0, win32::MENU_ID_QUIT, quit_text.as_ptr());

    let mut pt: win32::POINT = std::mem::zeroed();
    win32::GetCursorPos(&mut pt);

    win32::SetForegroundWindow(hwnd);
    win32::TrackPopupMenu(
        h_menu,
        win32::TPM_RIGHTBUTTON,
        pt.x,
        pt.y,
        0,
        hwnd,
        std::ptr::null(),
    );
    win32::PostMessageW(hwnd, 0, 0, 0);
    win32::DestroyMenu(h_menu);
}

#[cfg(not(target_os = "windows"))]
fn start_tray_and_hotkey_listener(_sender: Sender<TrayEvent>, _ctx: Context) {}

#[cfg(target_os = "windows")]
pub mod win32 {
    use std::ffi::c_void;

    pub type HWND = *mut c_void;
    pub type HICON = *mut c_void;
    pub type HINSTANCE = *mut c_void;
    pub type HMENU = *mut c_void;
    pub type LRESULT = isize;
    pub type WPARAM = usize;
    pub type LPARAM = isize;
    pub type UINT = u32;
    pub type BOOL = i32;
    pub type DWORD = u32;

    pub const WM_HOTKEY: UINT = 0x0312;
    pub const WM_APP: UINT = 0x8000;
    pub const WM_TRAYICON: UINT = WM_APP + 100;
    pub const WM_LBUTTONUP: UINT = 0x0202;
    pub const WM_RBUTTONUP: UINT = 0x0205;
    pub const WM_LBUTTONDBLCLK: UINT = 0x0203;
    pub const WM_COMMAND: UINT = 0x0111;
    pub const WM_DESTROY: UINT = 0x0002;

    pub const HOTKEY_ID_ALT_K: i32 = 1001;
    pub const MENU_ID_OPEN: usize = 2001;
    pub const MENU_ID_QUIT: usize = 2002;

    pub const MOD_ALT: UINT = 0x0001;
    pub const MOD_NOREPEAT: UINT = 0x4000;
    pub const VK_K: UINT = 0x4B;

    pub const NIM_ADD: DWORD = 0x00000000;
    #[allow(dead_code)]
    pub const NIM_MODIFY: DWORD = 0x00000001;
    pub const NIM_DELETE: DWORD = 0x00000002;

    pub const NIF_MESSAGE: UINT = 0x00000001;
    pub const NIF_ICON: UINT = 0x00000002;
    pub const NIF_TIP: UINT = 0x00000004;

    pub const SW_HIDE: i32 = 0;
    pub const SW_SHOW: i32 = 5;
    pub const SW_RESTORE: i32 = 9;

    pub const TPM_RIGHTBUTTON: UINT = 0x0002;
    pub const MF_SEPARATOR: UINT = 0x00000800;
    pub const IDI_APPLICATION: *const u16 = 32512 as *const u16;

    #[repr(C)]
    pub struct NOTIFYICONDATAW {
        pub cb_size: DWORD,
        pub h_wnd: HWND,
        pub u_id: UINT,
        pub u_flags: UINT,
        pub u_callback_message: UINT,
        pub h_icon: HICON,
        pub sz_tip: [u16; 128],
        pub dw_state: DWORD,
        pub dw_state_mask: DWORD,
        pub sz_info: [u16; 256],
        pub u_timeout_or_version: UINT,
        pub sz_info_title: [u16; 64],
        pub dw_info_flags: DWORD,
        pub guid_item: [u8; 16],
        pub h_balloon_icon: HICON,
    }

    #[repr(C)]
    pub struct POINT {
        pub x: i32,
        pub y: i32,
    }

    #[repr(C)]
    pub struct MSG {
        pub hwnd: HWND,
        pub message: UINT,
        pub w_param: WPARAM,
        pub l_param: LPARAM,
        pub time: DWORD,
        pub pt: POINT,
    }

    #[repr(C)]
    pub struct WNDCLASSEXW {
        pub cb_size: UINT,
        pub style: UINT,
        pub lpfn_wnd_proc: unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
        pub cb_cls_extra: i32,
        pub cb_wnd_extra: i32,
        pub h_instance: HINSTANCE,
        pub h_icon: HICON,
        pub h_cursor: *mut c_void,
        pub hbr_background: *mut c_void,
        pub lpsz_menu_name: *const u16,
        pub lpsz_class_name: *const u16,
        pub h_icon_sm: HICON,
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn RegisterHotKey(h_wnd: HWND, id: i32, fs_modifiers: UINT, vk: UINT) -> BOOL;
        pub fn UnregisterHotKey(h_wnd: HWND, id: i32) -> BOOL;
        pub fn GetMessageW(lp_msg: *mut MSG, h_wnd: HWND, w_msg_filter_min: UINT, w_msg_filter_max: UINT) -> BOOL;
        pub fn TranslateMessage(lp_msg: *const MSG) -> BOOL;
        pub fn DispatchMessageW(lp_msg: *const MSG) -> LRESULT;
        pub fn DefWindowProcW(h_wnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT;
        pub fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
        pub fn CreateWindowExW(
            dw_ex_style: DWORD,
            lp_class_name: *const u16,
            lp_window_name: *const u16,
            dw_style: DWORD,
            x: i32,
            y: i32,
            n_width: i32,
            n_height: i32,
            h_wnd_parent: HWND,
            h_menu: HMENU,
            h_instance: HINSTANCE,
            lp_param: *mut c_void,
        ) -> HWND;
        pub fn PostQuitMessage(n_exit_code: i32);
        pub fn PostMessageW(h_wnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> BOOL;
        pub fn FindWindowW(lp_class_name: *const u16, lp_window_name: *const u16) -> HWND;
        pub fn ShowWindow(h_wnd: HWND, n_cmd_show: i32) -> BOOL;
        pub fn SetForegroundWindow(h_wnd: HWND) -> BOOL;
        pub fn GetForegroundWindow() -> HWND;
        pub fn BringWindowToTop(h_wnd: HWND) -> BOOL;
        pub fn SetFocus(h_wnd: HWND) -> HWND;
        pub fn GetCursorPos(lp_point: *mut POINT) -> BOOL;
        pub fn CreatePopupMenu() -> HMENU;
        pub fn AppendMenuW(h_menu: HMENU, u_flags: UINT, u_id_new_item: usize, lp_new_item: *const u16) -> BOOL;
        pub fn TrackPopupMenu(h_menu: HMENU, u_flags: UINT, x: i32, y: i32, n_reserved: i32, h_wnd: HWND, prc_rect: *const c_void) -> BOOL;
        pub fn DestroyMenu(h_menu: HMENU) -> BOOL;
        pub fn LoadIconW(h_instance: HINSTANCE, lp_icon_name: *const u16) -> HICON;
        pub fn CreateIconFromResourceEx(
            pb_icon_bits: *const u8,
            cb_icon_bits: DWORD,
            f_icon: BOOL,
            dw_version: DWORD,
            cx_desired: i32,
            cy_desired: i32,
            u_flags: UINT,
        ) -> HICON;
        pub fn IsWindowVisible(h_wnd: HWND) -> BOOL;
        #[allow(dead_code)]
        pub fn AttachThreadInput(id_attach: DWORD, id_attach_to: DWORD, f_attach: BOOL) -> BOOL;
        #[allow(dead_code)]
        pub fn GetWindowThreadProcessId(h_wnd: HWND, lpdw_process_id: *mut DWORD) -> DWORD;
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn GetModuleHandleW(lp_module_name: *const u16) -> HINSTANCE;
        #[allow(dead_code)]
        pub fn GetCurrentThreadId() -> DWORD;
    }

    #[link(name = "shell32")]
    extern "system" {
        pub fn Shell_NotifyIconW(dw_message: DWORD, lp_data: *const NOTIFYICONDATAW) -> BOOL;
    }
}
