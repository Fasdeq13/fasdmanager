use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum XdgUserDir {
    Desktop,
    Download,
    Templates,
    PublicShare,
    Documents,
    Music,
    Pictures,
    Videos,
}

impl XdgUserDir {
    fn env_key(self) -> &'static str {
        match self {
            XdgUserDir::Desktop => "XDG_DESKTOP_DIR",
            XdgUserDir::Download => "XDG_DOWNLOAD_DIR",
            XdgUserDir::Templates => "XDG_TEMPLATES_DIR",
            XdgUserDir::PublicShare => "XDG_PUBLICSHARE_DIR",
            XdgUserDir::Documents => "XDG_DOCUMENTS_DIR",
            XdgUserDir::Music => "XDG_MUSIC_DIR",
            XdgUserDir::Pictures => "XDG_PICTURES_DIR",
            XdgUserDir::Videos => "XDG_VIDEOS_DIR",
        }
    }

    fn fallback_english_name(self) -> &'static str {
        match self {
            XdgUserDir::Desktop => "Desktop",
            XdgUserDir::Download => "Downloads",
            XdgUserDir::Templates => "Templates",
            XdgUserDir::PublicShare => "Public",
            XdgUserDir::Documents => "Documents",
            XdgUserDir::Music => "Music",
            XdgUserDir::Pictures => "Pictures",
            XdgUserDir::Videos => "Videos",
        }
    }
}

fn unescape_shell_value(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                result.push(next);
                chars.next();
                continue;
            }
        }
        result.push(c);
    }
    result
}

fn parse_user_dirs_file(content: &str, home: &std::path::Path) -> HashMap<String, PathBuf> {
    let mut result = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some(eq_pos) = line.find('=') else {
            continue;
        };

        let key = line[..eq_pos].trim();
        if !key.starts_with("XDG_") || !key.ends_with("_DIR") {
            continue;
        }

        let mut value = line[eq_pos + 1..].trim();
        if (value.starts_with('"') && value.ends_with('"') && value.len() >= 2)
            || (value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2)
        {
            value = &value[1..value.len() - 1];
        }

        let unescaped = unescape_shell_value(value);

        let resolved_path = if let Some(rest) = unescaped.strip_prefix("$HOME/") {
            home.join(rest)
        } else if let Some(rest) = unescaped.strip_prefix("$HOME") {
            if rest.is_empty() {
                home.to_path_buf()
            } else {
                home.join(rest.trim_start_matches('/'))
            }
        } else if unescaped.starts_with('/') {
            PathBuf::from(unescaped)
        } else {
            home.join(unescaped)
        };

        result.insert(key.to_string(), resolved_path);
    }

    result
}

pub fn resolve_xdg_user_dir(dir: XdgUserDir) -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

    let config_dir = dirs::config_dir().unwrap_or_else(|| home.join(".config"));
    let user_dirs_file = config_dir.join("user-dirs.dirs");

    if let Ok(content) = std::fs::read_to_string(&user_dirs_file) {
        let parsed = parse_user_dirs_file(&content, &home);
        if let Some(path) = parsed.get(dir.env_key()) {
            if path.is_dir() {
                return path.clone();
            }
        }
    }

    home.join(dir.fallback_english_name())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_english_dirs() {
        let content = "XDG_MUSIC_DIR=\"$HOME/Music\"\nXDG_DOWNLOAD_DIR=\"$HOME/Downloads\"\n";
        let home = PathBuf::from("/home/test");
        let parsed = parse_user_dirs_file(content, &home);
        assert_eq!(
            parsed.get("XDG_MUSIC_DIR"),
            Some(&PathBuf::from("/home/test/Music"))
        );
        assert_eq!(
            parsed.get("XDG_DOWNLOAD_DIR"),
            Some(&PathBuf::from("/home/test/Downloads"))
        );
    }

    #[test]
    fn parses_russian_localized_dirs() {
        let content = "XDG_MUSIC_DIR=\"$HOME/Музыка\"\nXDG_DOCUMENTS_DIR=\"$HOME/Документы\"\n";
        let home = PathBuf::from("/home/fasdeq");
        let parsed = parse_user_dirs_file(content, &home);
        assert_eq!(
            parsed.get("XDG_MUSIC_DIR"),
            Some(&PathBuf::from("/home/fasdeq/Музыка"))
        );
        assert_eq!(
            parsed.get("XDG_DOCUMENTS_DIR"),
            Some(&PathBuf::from("/home/fasdeq/Документы"))
        );
    }

    #[test]
    fn parses_greek_dirs_with_spaces() {
        let content = "XDG_DESKTOP_DIR=\"$HOME/Επιφάνεια εργασίας\"\n";
        let home = PathBuf::from("/home/test");
        let parsed = parse_user_dirs_file(content, &home);
        assert_eq!(
            parsed.get("XDG_DESKTOP_DIR"),
            Some(&PathBuf::from("/home/test/Επιφάνεια εργασίας"))
        );
    }

    #[test]
    fn ignores_comments_and_blank_lines() {
        let content = "# comment\n\nXDG_MUSIC_DIR=\"$HOME/Music\"\n";
        let home = PathBuf::from("/home/test");
        let parsed = parse_user_dirs_file(content, &home);
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn handles_custom_absolute_path() {
        let content = "XDG_DOWNLOAD_DIR=\"/mnt/storage/downloads\"\n";
        let home = PathBuf::from("/home/test");
        let parsed = parse_user_dirs_file(content, &home);
        assert_eq!(
            parsed.get("XDG_DOWNLOAD_DIR"),
            Some(&PathBuf::from("/mnt/storage/downloads"))
        );
    }
}
