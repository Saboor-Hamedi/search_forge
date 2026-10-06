use unicode_segmentation::UnicodeSegmentation;

#[allow(dead_code)]
pub fn grapheme_count(text: &str) -> usize {
    text.graphemes(true).count()
}


pub fn format_size(size_bytes: u64) -> String {
    if size_bytes < 1024 {
        format!("{} B", size_bytes)
    } else if size_bytes < 1024 * 1024 {
        format!("{:.1} KB", size_bytes as f64 / 1024.0)
    } else if size_bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size_bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Normalizes text for blazing fast multilingual matching (Persian, Arabic, English, etc.)
pub fn normalize_for_search(text: &str) -> String {
    let mut res = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            // Arabic to Persian / uniform letter variants
            'ي' | 'ى' | 'ئ' => res.push('ی'),
            'ك' => res.push('ک'),
            'ة' => res.push('ه'),
            'أ' | 'إ' | 'آ' => res.push('ا'),
            'ـ' => {}, // Remove Arabic Tatweel / Kashida
            // Eastern Arabic-Indic digits ('٠'..='٩') to ASCII 0-9
            '٠' => res.push('0'),
            '١' => res.push('1'),
            '٢' => res.push('2'),
            '٣' => res.push('3'),
            '٤' => res.push('4'),
            '٥' => res.push('5'),
            '٦' => res.push('6'),
            '٧' => res.push('7'),
            '٨' => res.push('8'),
            '٩' => res.push('9'),
            // Persian/Urdu digits ('۰'..='۹') to ASCII 0-9
            '۰' => res.push('0'),
            '۱' => res.push('1'),
            '۲' => res.push('2'),
            '۳' => res.push('3'),
            '۴' => res.push('4'),
            '۵' => res.push('5'),
            '۶' => res.push('6'),
            '۷' => res.push('7'),
            '۸' => res.push('8'),
            '۹' => res.push('9'),
            // Remove Arabic Harakat / diacritics
            '\u{064B}'..='\u{065F}' | '\u{0670}' => {},
            _ => {
                for lc in c.to_lowercase() {
                    res.push(lc);
                }
            }
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persian_arabic_normalization() {
        assert_eq!(normalize_for_search("سلام"), normalize_for_search("سَلام"));
        assert_eq!(normalize_for_search("كتاب"), normalize_for_search("کتاب"));
        assert_eq!(normalize_for_search("علي"), normalize_for_search("علی"));
        assert_eq!(normalize_for_search("۹۹"), "99");
        assert_eq!(normalize_for_search("٩٩"), "99");
    }
}

