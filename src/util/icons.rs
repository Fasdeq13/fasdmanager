use crate::fs::entry::EntryKind;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};

static ICON_THEME_SEARCH_ROOTS: Lazy<Vec<PathBuf>> = Lazy::new(|| {
    let mut roots = Vec::new();
    if let Some(home) = dirs::home_dir() {
        roots.push(home.join(".local/share/icons"));
        roots.push(home.join(".icons"));
    }
    roots.push(PathBuf::from("/usr/share/icons"));
    roots.push(PathBuf::from("/usr/local/share/icons"));
    roots.push(PathBuf::from("/usr/share/pixmaps"));
    roots
});

const PREFERRED_THEME_NAMES: [&str; 3] = ["Adwaita", "hicolor", "breeze"];

fn find_svg_in_theme_root(theme_root: &Path, icon_name: &str) -> Option<PathBuf> {
    if !theme_root.is_dir() {
        return None;
    }

    let scalable_dir = theme_root.join("scalable");
    if scalable_dir.is_dir() {
        if let Some(found) = search_scalable_subdirs(&scalable_dir, icon_name) {
            return Some(found);
        }
    }

    let symbolic_dir = theme_root.join("symbolic");
    if symbolic_dir.is_dir() {
        if let Some(found) = search_scalable_subdirs(&symbolic_dir, icon_name) {
            return Some(found);
        }
    }

    None
}

fn search_scalable_subdirs(scalable_root: &Path, icon_name: &str) -> Option<PathBuf> {
    let Ok(categories) = std::fs::read_dir(scalable_root) else {
        return None;
    };
    for category in categories.flatten() {
        let candidate = category.path().join(format!("{icon_name}.svg"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub fn find_forced_svg_path(icon_name: &str) -> Option<PathBuf> {
    if icon_name.is_empty() {
        return None;
    }

    if icon_name.starts_with('/') {
        let direct = PathBuf::from(icon_name);
        if direct.extension().and_then(|e| e.to_str()) == Some("svg") && direct.is_file() {
            return Some(direct);
        }
        let with_svg_ext = direct.with_extension("svg");
        if with_svg_ext.is_file() {
            return Some(with_svg_ext);
        }
        return None;
    }

    for root in ICON_THEME_SEARCH_ROOTS.iter() {
        for theme_name in PREFERRED_THEME_NAMES {
            let theme_root = root.join(theme_name);
            if let Some(found) = find_svg_in_theme_root(&theme_root, icon_name) {
                return Some(found);
            }
        }

        let Ok(all_themes) = std::fs::read_dir(root) else {
            continue;
        };
        for theme_entry in all_themes.flatten() {
            if !theme_entry.path().is_dir() {
                continue;
            }
            if let Some(found) = find_svg_in_theme_root(&theme_entry.path(), icon_name) {
                return Some(found);
            }
        }
    }

    None
}

pub fn icon_name_for(path: &Path, kind: &EntryKind, content_type: Option<&str>) -> String {
    match kind {
        EntryKind::Directory => directory_icon_name(path),
        EntryKind::Symlink => "emblem-symbolic-link-symbolic".to_string(),
        EntryKind::AppLauncher => "application-x-executable-symbolic".to_string(),
        EntryKind::File => file_icon_name(content_type),
    }
}

fn directory_icon_name(path: &Path) -> String {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    match name.as_str() {
        "desktop" => "user-desktop".to_string(),
        "documents" => "folder-documents".to_string(),
        "downloads" => "folder-download".to_string(),
        "music" => "folder-music".to_string(),
        "pictures" => "folder-pictures".to_string(),
        "videos" => "folder-videos".to_string(),
        "templates" => "folder-templates".to_string(),
        "public" => "folder-publicshare".to_string(),
        _ => "folder".to_string(),
    }
}

fn file_icon_name(content_type: Option<&str>) -> String {
    let Some(mime) = content_type else {
        return "text-x-generic".to_string();
    };

    if mime.starts_with("image/") {
        return "image-x-generic".to_string();
    }
    if mime.starts_with("video/") {
        return "video-x-generic".to_string();
    }
    if mime.starts_with("audio/") {
        return "audio-x-generic".to_string();
    }
    if mime.starts_with("text/") {
        return "text-x-generic".to_string();
    }
    if mime == "application/pdf" {
        return "application-pdf".to_string();
    }
    if mime == "application/zip"
        || mime == "application/x-tar"
        || mime == "application/gzip"
        || mime == "application/x-7z-compressed"
        || mime == "application/x-rar-compressed"
    {
        return "package-x-generic".to_string();
    }
    if mime == "application/x-executable" || mime == "application/x-sharedlib" {
        return "application-x-executable".to_string();
    }
    if mime.contains("rust") || mime.contains("script") {
        return "text-x-script".to_string();
    }

    "text-x-generic".to_string()
}

pub fn resolve_desktop_icon_name(raw_icon: &str) -> String {
    if raw_icon.starts_with('/') {
        return raw_icon.to_string();
    }
    let trimmed = raw_icon
        .trim_end_matches(".svg")
        .trim_end_matches(".png")
        .trim_end_matches(".xpm");
    trimmed.to_string()
}
