use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DesktopEntry {
    pub source_path: PathBuf,
    pub name: String,
    pub icon: Option<String>,
    pub exec: Option<String>,
    pub no_display: bool,
    pub hidden: bool,
    pub is_application: bool,
    pub terminal: bool,
    pub comment: Option<String>,
}

fn strip_field_codes(exec: &str) -> String {
    let mut result = String::with_capacity(exec.len());
    let mut chars = exec.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            if let Some(&next) = chars.peek() {
                match next {
                    'f' | 'F' | 'u' | 'U' | 'd' | 'D' | 'n' | 'N' | 'i' | 'c' | 'k' | 'v' | 'm' => {
                        chars.next();
                        continue;
                    }
                    '%' => {
                        result.push('%');
                        chars.next();
                        continue;
                    }
                    _ => {
                        result.push(c);
                    }
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    result.trim().to_string()
}

fn localized_key_matches(raw_key: &str, base_key: &str, lang_code: &str) -> bool {
    if raw_key == base_key {
        return true;
    }
    let expected_localized = format!("{base_key}[{lang_code}]");
    raw_key == expected_localized
}

pub fn parse_desktop_file(path: &Path, lang_code: &str) -> Option<DesktopEntry> {
    let content = std::fs::read_to_string(path).ok()?;

    let mut in_desktop_entry_section = false;
    let mut plain_values: HashMap<String, String> = HashMap::new();
    let mut localized_values: HashMap<String, String> = HashMap::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            in_desktop_entry_section = line == "[Desktop Entry]";
            continue;
        }

        if !in_desktop_entry_section {
            continue;
        }

        let Some(eq_pos) = line.find('=') else {
            continue;
        };

        let raw_key = line[..eq_pos].trim();
        let value = line[eq_pos + 1..].trim();

        for base_key in ["Name", "Comment", "GenericName"] {
            if localized_key_matches(raw_key, base_key, lang_code) {
                if raw_key == base_key {
                    plain_values.insert(base_key.to_string(), value.to_string());
                } else {
                    localized_values.insert(base_key.to_string(), value.to_string());
                }
            }
        }

        match raw_key {
            "Icon" => {
                plain_values.insert("Icon".to_string(), value.to_string());
            }
            "Exec" => {
                plain_values.insert("Exec".to_string(), value.to_string());
            }
            "Type" => {
                plain_values.insert("Type".to_string(), value.to_string());
            }
            "NoDisplay" => {
                plain_values.insert("NoDisplay".to_string(), value.to_string());
            }
            "Hidden" => {
                plain_values.insert("Hidden".to_string(), value.to_string());
            }
            "Terminal" => {
                plain_values.insert("Terminal".to_string(), value.to_string());
            }
            _ => {}
        }
    }

    let entry_type = plain_values
        .get("Type")
        .map(|s| s.as_str())
        .unwrap_or("Application");
    let is_application = entry_type == "Application";

    let name = localized_values
        .get("Name")
        .or_else(|| plain_values.get("Name"))
        .cloned()
        .or_else(|| {
            path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
        })?;

    let comment = localized_values
        .get("Comment")
        .or_else(|| plain_values.get("Comment"))
        .cloned();

    let icon = plain_values.get("Icon").cloned();
    let exec_raw = plain_values.get("Exec").cloned();
    let exec = exec_raw.map(|e| strip_field_codes(&e));

    let no_display = plain_values
        .get("NoDisplay")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let hidden = plain_values
        .get("Hidden")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let terminal = plain_values
        .get("Terminal")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    Some(DesktopEntry {
        source_path: path.to_path_buf(),
        name,
        icon,
        exec,
        no_display,
        hidden,
        is_application,
        terminal,
        comment,
    })
}

fn scan_one_dir(dir: &Path, lang_code: &str, out: &mut Vec<DesktopEntry>) {
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return;
    };
    for item in read_dir.flatten() {
        let path = item.path();
        if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
            continue;
        }
        if let Some(entry) = parse_desktop_file(&path, lang_code) {
            if entry.is_application && !entry.no_display && !entry.hidden {
                out.push(entry);
            }
        }
    }
}

pub fn scan_desktop_applications(lang_code: &str) -> Vec<DesktopEntry> {
    let search_dirs: Vec<PathBuf> = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
        dirs::data_local_dir()
            .map(|p| p.join("applications"))
            .unwrap_or_else(|| PathBuf::from("/nonexistent")),
    ];

    let mut all_entries = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    for dir in search_dirs {
        let mut batch = Vec::new();
        scan_one_dir(&dir, lang_code, &mut batch);
        for entry in batch {
            if seen_names.insert(entry.name.clone()) {
                all_entries.push(entry);
            }
        }
    }

    all_entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    all_entries
}

pub async fn scan_desktop_applications_async(lang_code: String) -> Vec<DesktopEntry> {
    let handle =
        tokio::task::spawn_blocking(move || scan_desktop_applications(&lang_code));
    handle.await.unwrap_or_default()
}
