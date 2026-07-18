use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};

pub mod strings_en;
pub mod strings_it;
pub mod strings_ru;
pub mod strings_zh;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    Ru,
    En,
    It,
    Zh,
}

impl Lang {
    fn as_index(self) -> u8 {
        match self {
            Lang::Ru => 0,
            Lang::En => 1,
            Lang::It => 2,
            Lang::Zh => 3,
        }
    }

    fn from_index(idx: u8) -> Lang {
        match idx {
            0 => Lang::Ru,
            2 => Lang::It,
            3 => Lang::Zh,
            _ => Lang::En,
        }
    }

    pub fn native_name(self) -> &'static str {
        match self {
            Lang::Ru => "Русский",
            Lang::En => "English",
            Lang::It => "Italiano",
            Lang::Zh => "中文",
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::Ru => "ru",
            Lang::En => "en",
            Lang::It => "it",
            Lang::Zh => "zh",
        }
    }
}

static CURRENT_LANG: AtomicU8 = AtomicU8::new(1);

type StringTable = HashMap<&'static str, &'static str>;

static TABLE_RU: Lazy<StringTable> = Lazy::new(strings_ru::build_table);
static TABLE_EN: Lazy<StringTable> = Lazy::new(strings_en::build_table);
static TABLE_IT: Lazy<StringTable> = Lazy::new(strings_it::build_table);
static TABLE_ZH: Lazy<StringTable> = Lazy::new(strings_zh::build_table);

fn extract_lang_code(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() || raw.eq_ignore_ascii_case("C") || raw.eq_ignore_ascii_case("POSIX") {
        return None;
    }
    let stop = raw.find(['_', '.', '@']).unwrap_or(raw.len());
    let code = &raw[..stop];
    if code.is_empty() {
        None
    } else {
        Some(code.to_lowercase())
    }
}

pub fn detect_system_lang() -> Lang {
    let candidates = [
        std::env::var("LC_ALL").ok(),
        std::env::var("LC_MESSAGES").ok(),
        std::env::var("LANG").ok(),
    ];

    for candidate in candidates.into_iter().flatten() {
        if let Some(code) = extract_lang_code(&candidate) {
            match code.as_str() {
                "ru" => return Lang::Ru,
                "en" => return Lang::En,
                "it" => return Lang::It,
                "zh" => return Lang::Zh,
                _ => continue,
            }
        }
    }
    Lang::En
}

pub fn init() {
    let lang = detect_system_lang();
    set_lang(lang);
    log::info!(
        "i18n init: detected={} ({})",
        lang.native_name(),
        lang.code()
    );
}

pub fn set_lang(lang: Lang) {
    CURRENT_LANG.store(lang.as_index(), Ordering::SeqCst);
}

pub fn current_lang() -> Lang {
    Lang::from_index(CURRENT_LANG.load(Ordering::SeqCst))
}

pub fn tr(key: &str) -> String {
    let table = match current_lang() {
        Lang::Ru => &*TABLE_RU,
        Lang::En => &*TABLE_EN,
        Lang::It => &*TABLE_IT,
        Lang::Zh => &*TABLE_ZH,
    };
    if let Some(value) = table.get(key) {
        return value.to_string();
    }
    if let Some(value) = TABLE_EN.get(key) {
        return value.to_string();
    }
    key.to_string()
}

pub fn tr_with(key: &str, value: &str) -> String {
    let base = tr(key);
    if base.contains("{}") {
        base.replacen("{}", value, 1)
    } else {
        format!("{base} {value}")
    }
}

pub fn tr_with_many(key: &str, values: &[&str]) -> String {
    let mut result = tr(key);
    for value in values {
        if let Some(pos) = result.find("{}") {
            result.replace_range(pos..pos + 2, value);
        } else {
            result.push(' ');
            result.push_str(value);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_russian_from_lang() {
        assert_eq!(extract_lang_code("ru_RU.UTF-8").as_deref(), Some("ru"));
    }

    #[test]
    fn detects_italian_plain() {
        assert_eq!(extract_lang_code("it_IT").as_deref(), Some("it"));
    }

    #[test]
    fn detects_chinese_with_modifier() {
        assert_eq!(extract_lang_code("zh_CN.UTF-8@pinyin").as_deref(), Some("zh"));
    }

    #[test]
    fn rejects_posix_and_c() {
        assert_eq!(extract_lang_code("C"), None);
        assert_eq!(extract_lang_code("POSIX"), None);
    }

    #[test]
    fn unsupported_code_is_still_parsed_but_unmatched() {
        assert_eq!(extract_lang_code("de_DE.UTF-8").as_deref(), Some("de"));
    }
}
