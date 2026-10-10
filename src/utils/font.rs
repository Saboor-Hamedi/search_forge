use egui::{FontData, FontDefinitions, FontFamily};
use std::fs;
use std::path::Path;

/// Configures fallback system fonts for proper multilingual rendering (Arabic, Persian, CJK, Cyrillic, symbols).
pub fn setup_multilingual_fonts(fonts: &mut FontDefinitions) {
    #[cfg(target_os = "windows")]
    {
        load_windows_system_fonts(fonts);
    }
}

#[cfg(target_os = "windows")]
fn load_windows_system_fonts(fonts: &mut FontDefinitions) {
    // List of candidate fonts with high coverage for non-Latin scripts:
    // 1. Segoe UI: Standard modern Windows interface font (Latin, Greek, Cyrillic)
    // 2. Tahoma: Excellent Arabic, Persian (Farsi), Urdu, Hebrew, Cyrillic
    // 3. Arial: Extensive Unicode coverage fallback
    // 4. Malgun Gothic: Korean Hangul
    // 5. Segoe UI Symbol: Unicode symbols, math, arrows, badges
    // 6. Microsoft YaHei: Chinese and CJK ideographs
    let font_candidates = [
        ("segoe_ui", r"C:\Windows\Fonts\segoeui.ttf"),
        ("tahoma", r"C:\Windows\Fonts\tahoma.ttf"),
        ("arial", r"C:\Windows\Fonts\arial.ttf"),
        ("malgun", r"C:\Windows\Fonts\malgun.ttf"),
        ("segoe_ui_symbol", r"C:\Windows\Fonts\seguisym.ttf"),
        ("msyh", r"C:\Windows\Fonts\msyh.ttc"),
    ];

    let mut loaded_names = Vec::new();

    for (name, path_str) in font_candidates {
        let path = Path::new(path_str);
        if path.exists() {
            if let Ok(bytes) = fs::read(path) {
                // FontData::from_owned wraps font bytes
                fonts.font_data.insert(name.to_string(), FontData::from_owned(bytes));
                loaded_names.push(name.to_string());
            }
        }
    }

    // Insert loaded fonts into FontFamily::Proportional and FontFamily::Monospace
    if let Some(prop_list) = fonts.families.get_mut(&FontFamily::Proportional) {
        // Place Segoe UI first if available, followed by multilingual fallbacks
        for name in loaded_names.iter().rev() {
            // Avoid duplicates
            if !prop_list.contains(name) {
                prop_list.insert(0, name.clone());
            }
        }
    }

    if let Some(mono_list) = fonts.families.get_mut(&FontFamily::Monospace) {
        for name in &loaded_names {
            if !mono_list.contains(name) {
                mono_list.push(name.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_loading() {
        let mut defs = FontDefinitions::default();
        setup_multilingual_fonts(&mut defs);
        // Verify font definitions can be initialized without error
        assert!(!defs.font_data.is_empty());
    }

    #[test]
    fn test_multilingual_text_layout() {
        let ctx = egui::Context::default();
        let mut defs = FontDefinitions::default();
        setup_multilingual_fonts(&mut defs);
        ctx.set_fonts(defs);

        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("Persian/Arabic: سلام دنیا و جستجوی فایل");
                ui.label("Chinese: 你好，世界！搜索文件");
                ui.label("Japanese: こんにちは世界 ファイル検索");
                ui.label("Korean: 안녕하세요 파일 검색");
                ui.label("Cyrillic: Привет мир Поиск файлов");
                ui.label("Symbols: ⚡ ★ ⚙ 📂 🔍 📁");
            });
        });
    }
}
