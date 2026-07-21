use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SidebarSection {
    Favorites,
    Devices,
    Locations,
}

impl SidebarSection {
    pub fn settings_key(self) -> &'static str {
        match self {
            SidebarSection::Favorites => "sidebar_section_favorites_visible",
            SidebarSection::Devices => "sidebar_section_devices_visible",
            SidebarSection::Locations => "sidebar_section_locations_visible",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FavoriteItem {
    Home,
    Desktop,
    Documents,
    Downloads,
    Pictures,
    Music,
    Videos,
    Applications,
}

impl FavoriteItem {
    pub fn settings_key(self) -> &'static str {
        match self {
            FavoriteItem::Home => "favorite_home_visible",
            FavoriteItem::Desktop => "favorite_desktop_visible",
            FavoriteItem::Documents => "favorite_documents_visible",
            FavoriteItem::Downloads => "favorite_downloads_visible",
            FavoriteItem::Pictures => "favorite_pictures_visible",
            FavoriteItem::Music => "favorite_music_visible",
            FavoriteItem::Videos => "favorite_videos_visible",
            FavoriteItem::Applications => "favorite_applications_visible",
        }
    }

    pub const ALL: [FavoriteItem; 8] = [
        FavoriteItem::Home,
        FavoriteItem::Desktop,
        FavoriteItem::Documents,
        FavoriteItem::Downloads,
        FavoriteItem::Pictures,
        FavoriteItem::Music,
        FavoriteItem::Videos,
        FavoriteItem::Applications,
    ];
}

fn settings_file_path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .join(".config")
    });
    config_dir.join("fasdmanager").join("settings.conf")
}

fn parse_settings_content(content: &str) -> (HashMap<String, bool>, HashMap<String, String>) {
    let mut bool_values = HashMap::new();
    let mut string_values = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some(eq_pos) = line.find('=') else {
            continue;
        };
        let key = line[..eq_pos].trim().to_string();
        let value = line[eq_pos + 1..].trim();

        if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
            bool_values.insert(key, value.eq_ignore_ascii_case("true"));
        } else {
            string_values.insert(key, value.to_string());
        }
    }

    (bool_values, string_values)
}

const ICON_THEME_SETTINGS_KEY: &str = "icon_theme";

pub struct Settings {
    values: HashMap<String, bool>,
    string_values: HashMap<String, String>,
}

impl Settings {
    pub fn load() -> Self {
        let path = settings_file_path();
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let (values, string_values) = parse_settings_content(&content);
        Settings {
            values,
            string_values,
        }
    }

    pub fn is_sidebar_section_visible(&self, section: SidebarSection) -> bool {
        self.values
            .get(section.settings_key())
            .copied()
            .unwrap_or(true)
    }

    pub fn is_favorite_visible(&self, item: FavoriteItem) -> bool {
        self.values.get(item.settings_key()).copied().unwrap_or(true)
    }

    pub fn set_sidebar_section_visible(&mut self, section: SidebarSection, visible: bool) {
        self.values
            .insert(section.settings_key().to_string(), visible);
    }

    pub fn set_favorite_visible(&mut self, item: FavoriteItem, visible: bool) {
        self.values.insert(item.settings_key().to_string(), visible);
    }

    pub fn icon_theme_value(&self) -> Option<String> {
        self.string_values.get(ICON_THEME_SETTINGS_KEY).cloned()
    }

    pub fn set_icon_theme_value(&mut self, value: &str) {
        self.string_values
            .insert(ICON_THEME_SETTINGS_KEY.to_string(), value.to_string());
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = settings_file_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut content = String::new();
        content.push_str("# FasdManager settings\n");
        content.push_str("# Auto-generated. Edit while the app is closed.\n\n");

        let mut bool_keys: Vec<&String> = self.values.keys().collect();
        bool_keys.sort();
        for key in bool_keys {
            let value = self.values.get(key).copied().unwrap_or(true);
            content.push_str(&format!("{key}={value}\n"));
        }

        let mut string_keys: Vec<&String> = self.string_values.keys().collect();
        string_keys.sort();
        for key in string_keys {
            if let Some(value) = self.string_values.get(key) {
                content.push_str(&format!("{key}={value}\n"));
            }
        }

        std::fs::write(path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_visible_when_key_missing() {
        let settings = Settings {
            values: HashMap::new(),
            string_values: HashMap::new(),
        };
        assert!(settings.is_sidebar_section_visible(SidebarSection::Devices));
        assert!(settings.is_favorite_visible(FavoriteItem::Applications));
    }

    #[test]
    fn respects_explicit_false() {
        let mut values = HashMap::new();
        values.insert(SidebarSection::Devices.settings_key().to_string(), false);
        let settings = Settings {
            values,
            string_values: HashMap::new(),
        };
        assert!(!settings.is_sidebar_section_visible(SidebarSection::Devices));
    }

    #[test]
    fn parses_key_value_content() {
        let content = "# comment\nfavorite_music_visible=false\nfavorite_home_visible=true\n";
        let (bool_values, _) = parse_settings_content(content);
        assert_eq!(bool_values.get("favorite_music_visible"), Some(&false));
        assert_eq!(bool_values.get("favorite_home_visible"), Some(&true));
    }

    #[test]
    fn parses_string_value_alongside_bool_values() {
        let content = "favorite_music_visible=false\nicon_theme=Papirus\n";
        let (bool_values, string_values) = parse_settings_content(content);
        assert_eq!(bool_values.get("favorite_music_visible"), Some(&false));
        assert_eq!(
            string_values.get("icon_theme").map(|s| s.as_str()),
            Some("Papirus")
        );
    }

    #[test]
    fn set_and_get_roundtrip() {
        let mut settings = Settings {
            values: HashMap::new(),
            string_values: HashMap::new(),
        };
        settings.set_favorite_visible(FavoriteItem::Music, false);
        assert!(!settings.is_favorite_visible(FavoriteItem::Music));
        settings.set_favorite_visible(FavoriteItem::Music, true);
        assert!(settings.is_favorite_visible(FavoriteItem::Music));
    }

    #[test]
    fn icon_theme_roundtrip() {
        let mut settings = Settings {
            values: HashMap::new(),
            string_values: HashMap::new(),
        };
        assert_eq!(settings.icon_theme_value(), None);
        settings.set_icon_theme_value("breeze");
        assert_eq!(settings.icon_theme_value().as_deref(), Some("breeze"));
    }
}
