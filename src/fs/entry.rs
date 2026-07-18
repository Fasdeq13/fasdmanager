use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntryKind {
    Directory,
    File,
    Symlink,
    AppLauncher,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub display_name: String,
    pub kind: EntryKind,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub is_hidden: bool,
    pub icon_name: String,
    pub content_type: Option<String>,
    pub exec_command: Option<String>,
    pub app_id: Option<String>,
}

impl FileEntry {
    pub fn from_path(path: &Path) -> std::io::Result<Self> {
        let metadata = std::fs::symlink_metadata(path)?;
        let file_type = metadata.file_type();

        let kind = if file_type.is_symlink() {
            EntryKind::Symlink
        } else if file_type.is_dir() {
            EntryKind::Directory
        } else {
            EntryKind::File
        };

        let display_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let is_hidden = display_name.starts_with('.');

        let resolved_metadata = if file_type.is_symlink() {
            std::fs::metadata(path).ok()
        } else {
            Some(metadata.clone())
        };

        let size = resolved_metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = metadata.modified().ok();

        let content_type = if kind == EntryKind::File {
            mime_guess::from_path(path)
                .first()
                .map(|m| m.essence_str().to_string())
        } else {
            None
        };

        let icon_name = crate::util::icons::icon_name_for(path, &kind, content_type.as_deref());

        Ok(FileEntry {
            path: path.to_path_buf(),
            display_name,
            kind,
            size,
            modified,
            is_hidden,
            icon_name,
            content_type,
            exec_command: None,
            app_id: None,
        })
    }

    pub fn from_app_entry(
        display_name: String,
        icon_name: String,
        exec_command: String,
        app_id: String,
        source_desktop_file: PathBuf,
    ) -> Self {
        FileEntry {
            path: source_desktop_file,
            display_name,
            kind: EntryKind::AppLauncher,
            size: 0,
            modified: None,
            is_hidden: false,
            icon_name,
            content_type: Some("application/x-desktop".to_string()),
            exec_command: Some(exec_command),
            app_id: Some(app_id),
        }
    }

    pub fn is_directory_like(&self) -> bool {
        matches!(self.kind, EntryKind::Directory)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Name,
    Size,
    Modified,
    Kind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

pub fn sort_entries(entries: &mut [FileEntry], key: SortKey, direction: SortDirection) {
    entries.sort_by(|a, b| {
        let dirs_first = match (a.is_directory_like(), b.is_directory_like()) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        };

        if dirs_first != std::cmp::Ordering::Equal {
            return dirs_first;
        }

        let ordering = match key {
            SortKey::Name => a
                .display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase()),
            SortKey::Size => a.size.cmp(&b.size),
            SortKey::Modified => a.modified.cmp(&b.modified),
            SortKey::Kind => a.kind.cmp(&b.kind),
        };

        match direction {
            SortDirection::Ascending => ordering,
            SortDirection::Descending => ordering.reverse(),
        }
    });
}
