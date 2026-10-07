use std::path::Path;

#[cfg(target_os = "windows")]
mod ffi {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    type HICON = *mut c_void;
    type HBITMAP = *mut c_void;
    type HDC = *mut c_void;
    type BOOL = i32;
    type DWORD = u32;

    #[repr(C)]
    struct SHFILEINFOW {
        h_icon: HICON,
        i_icon: i32,
        dw_attributes: DWORD,
        sz_display_name: [u16; 260],
        sz_type_name: [u16; 80],
    }

    #[repr(C)]
    struct ICONINFO {
        f_icon: BOOL,
        x_hotspot: DWORD,
        y_hotspot: DWORD,
        hbm_mask: HBITMAP,
        hbm_color: HBITMAP,
    }

    #[repr(C)]
    struct BITMAP {
        bm_type: i32,
        bm_width: i32,
        bm_height: i32,
        bm_width_bytes: i32,
        bm_planes: u16,
        bm_bits_pixel: u16,
        bm_bits: *mut c_void,
    }

    #[repr(C)]
    struct BITMAPINFOHEADER {
        bi_size: DWORD,
        bi_width: i32,
        bi_height: i32,
        bi_planes: u16,
        bi_bit_count: u16,
        bi_compression: DWORD,
        bi_size_image: DWORD,
        bi_xpels_per_meter: i32,
        bi_ypels_per_meter: i32,
        bi_clr_used: DWORD,
        bi_clr_important: DWORD,
    }

    #[repr(C)]
    struct RGBQUAD {
        rgb_blue: u8,
        rgb_green: u8,
        rgb_red: u8,
        rgb_reserved: u8,
    }

    #[repr(C)]
    struct BITMAPINFO {
        bmi_header: BITMAPINFOHEADER,
        bmi_colors: [RGBQUAD; 1],
    }

    const SHGFI_ICON: DWORD = 0x000000100;
    const SHGFI_LARGEICON: DWORD = 0x000000000;
    const DIB_RGB_COLORS: u32 = 0;
    const BI_RGB: DWORD = 0;

    #[link(name = "shell32")]
    extern "system" {
        fn SHGetFileInfoW(
            psz_path: *const u16,
            dw_file_attributes: DWORD,
            psfi: *mut SHFILEINFOW,
            cb_file_info: u32,
            u_flags: DWORD,
        ) -> usize;
    }

    #[link(name = "user32")]
    extern "system" {
        fn DestroyIcon(h_icon: HICON) -> BOOL;
        fn GetIconInfo(h_icon: HICON, p_icon_info: *mut ICONINFO) -> BOOL;
        fn GetDC(h_wnd: *mut c_void) -> HDC;
        fn ReleaseDC(h_wnd: *mut c_void, h_dc: HDC) -> i32;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn GetObjectW(h: *mut c_void, c: i32, pv: *mut c_void) -> i32;
        fn DeleteObject(ho: *mut c_void) -> BOOL;
        fn GetDIBits(
            hdc: HDC,
            hbm: HBITMAP,
            start: u32,
            c_lines: u32,
            lpv_bits: *mut c_void,
            lpbmi: *mut BITMAPINFO,
            usage: u32,
        ) -> i32;
    }

    pub fn extract_icon_rgba(path: &Path) -> Option<(u32, u32, Vec<u8>)> {
        let mut path_wide: Vec<u16> = path.as_os_str().encode_wide().collect();
        path_wide.push(0);

        unsafe {
            let mut shfi: SHFILEINFOW = std::mem::zeroed();
            let res = SHGetFileInfoW(
                path_wide.as_ptr(),
                0,
                &mut shfi,
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_LARGEICON,
            );

            if res == 0 || shfi.h_icon.is_null() {
                return None;
            }

            let mut icon_info: ICONINFO = std::mem::zeroed();
            if GetIconInfo(shfi.h_icon, &mut icon_info) == 0 {
                DestroyIcon(shfi.h_icon);
                return None;
            }

            let mut bmp: BITMAP = std::mem::zeroed();
            let bmp_target = if !icon_info.hbm_color.is_null() {
                icon_info.hbm_color
            } else {
                icon_info.hbm_mask
            };

            let bmp_res = GetObjectW(
                bmp_target,
                std::mem::size_of::<BITMAP>() as i32,
                &mut bmp as *mut _ as *mut c_void,
            );

            if bmp_res == 0 || bmp.bm_width <= 0 || bmp.bm_height <= 0 {
                if !icon_info.hbm_color.is_null() {
                    DeleteObject(icon_info.hbm_color);
                }
                if !icon_info.hbm_mask.is_null() {
                    DeleteObject(icon_info.hbm_mask);
                }
                DestroyIcon(shfi.h_icon);
                return None;
            }

            let width = bmp.bm_width as u32;
            let height = if icon_info.hbm_color.is_null() {
                (bmp.bm_height / 2) as u32
            } else {
                bmp.bm_height as u32
            };

            let hdc = GetDC(std::ptr::null_mut());
            if hdc.is_null() {
                if !icon_info.hbm_color.is_null() {
                    DeleteObject(icon_info.hbm_color);
                }
                if !icon_info.hbm_mask.is_null() {
                    DeleteObject(icon_info.hbm_mask);
                }
                DestroyIcon(shfi.h_icon);
                return None;
            }

            let mut bmi: BITMAPINFO = std::mem::zeroed();
            bmi.bmi_header.bi_size = std::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
            bmi.bmi_header.bi_width = width as i32;
            bmi.bmi_header.bi_height = -(height as i32); // Top-down
            bmi.bmi_header.bi_planes = 1;
            bmi.bmi_header.bi_bit_count = 32;
            bmi.bmi_header.bi_compression = BI_RGB;

            let pixel_count = (width * height) as usize;
            let mut bgra_buf: Vec<u8> = vec![0u8; pixel_count * 4];

            let target_hbm = if !icon_info.hbm_color.is_null() {
                icon_info.hbm_color
            } else {
                icon_info.hbm_mask
            };

            let lines = GetDIBits(
                hdc,
                target_hbm,
                0,
                height,
                bgra_buf.as_mut_ptr() as *mut c_void,
                &mut bmi,
                DIB_RGB_COLORS,
            );

            ReleaseDC(std::ptr::null_mut(), hdc);
            if !icon_info.hbm_color.is_null() {
                DeleteObject(icon_info.hbm_color);
            }
            if !icon_info.hbm_mask.is_null() {
                DeleteObject(icon_info.hbm_mask);
            }
            DestroyIcon(shfi.h_icon);

            if lines == 0 {
                return None;
            }

            let has_alpha = bgra_buf.chunks_exact(4).any(|chunk| chunk[3] > 0);

            let mut rgba = Vec::with_capacity(pixel_count * 4);
            for chunk in bgra_buf.chunks_exact(4) {
                let b = chunk[0];
                let g = chunk[1];
                let r = chunk[2];
                let a = if has_alpha { chunk[3] } else { 255 };
                rgba.push(r);
                rgba.push(g);
                rgba.push(b);
                rgba.push(a);
            }

            Some((width, height, rgba))
        }
    }
}

pub fn extract_icon(path: &Path) -> Option<(u32, u32, Vec<u8>)> {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        let ext_lower = ext.to_ascii_lowercase();
        if ext_lower == "png" || ext_lower == "ico" || ext_lower == "jpg" || ext_lower == "jpeg" {
            if let Ok(img) = image::open(path) {
                let rgba_img = img.to_rgba8();
                let (w, h) = rgba_img.dimensions();
                return Some((w, h, rgba_img.into_raw()));
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(icon) = ffi::extract_icon_rgba(path) {
            return Some(icon);
        }

        // Fallback for execution alias reparse points (e.g. wt.exe, notepad.exe)
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let lower = file_name.to_ascii_lowercase();
        let package_prefix = match lower.as_str() {
            "wt.exe" => Some("Microsoft.WindowsTerminal"),
            "notepad.exe" => Some("Microsoft.WindowsNotepad"),
            "snippingtool.exe" => Some("Microsoft.ScreenSketch"),
            "mspaint.exe" | "pbrush.exe" => Some("Microsoft.Paint"),
            "store.exe" | "microsoftstore.exe" => Some("Microsoft.WindowsStore"),
            "ms-teams.exe" => Some("MicrosoftTeams"),
            _ => None,
        };

        if let Some(prefix) = package_prefix {
            if let Some(logo_path) = crate::indexer::app_scanner::resolve_app_alias_logo(prefix) {
                if let Ok(img) = image::open(&logo_path) {
                    let rgba_img = img.to_rgba8();
                    let (w, h) = rgba_img.dimensions();
                    return Some((w, h, rgba_img.into_raw()));
                }
            }
        }

        None
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = path;
        None
    }
}
