use crate::fs::entry::FileEntry;
use std::path::PathBuf;

#[derive(Debug)]
pub enum DirReadError {
    NotFound(PathBuf),
    PermissionDenied(PathBuf),
    Other(String),
}

impl std::fmt::Display for DirReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirReadError::NotFound(p) => write!(f, "not found: {}", p.display()),
            DirReadError::PermissionDenied(p) => write!(f, "permission denied: {}", p.display()),
            DirReadError::Other(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for DirReadError {}

fn read_dir_blocking(path: PathBuf) -> Result<Vec<FileEntry>, DirReadError> {
    let read_dir = std::fs::read_dir(&path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => DirReadError::NotFound(path.clone()),
        std::io::ErrorKind::PermissionDenied => DirReadError::PermissionDenied(path.clone()),
        _ => DirReadError::Other(e.to_string()),
    })?;

    let mut entries = Vec::new();
    for item in read_dir {
        let item = match item {
            Ok(v) => v,
            Err(_) => continue,
        };
        match FileEntry::from_path(&item.path()) {
            Ok(entry) => entries.push(entry),
            Err(_) => continue,
        }
    }
    Ok(entries)
}

pub async fn read_dir_async(path: PathBuf) -> Result<Vec<FileEntry>, DirReadError> {
    let handle = tokio::task::spawn_blocking(move || read_dir_blocking(path));
    match handle.await {
        Ok(result) => result,
        Err(join_err) => Err(DirReadError::Other(join_err.to_string())),
    }
}

fn search_blocking(
    root: PathBuf,
    query: String,
    max_results: usize,
    include_hidden: bool,
) -> Vec<FileEntry> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    let walker = walkdir::WalkDir::new(&root)
        .max_depth(12)
        .into_iter()
        .filter_entry(|e| {
            if include_hidden {
                return true;
            }
            e.file_name()
                .to_str()
                .map(|s| !s.starts_with('.') || e.path() == root)
                .unwrap_or(true)
        });

    for entry in walker.filter_map(|e| e.ok()) {
        if results.len() >= max_results {
            break;
        }
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.contains(&query_lower) {
            if let Ok(fe) = FileEntry::from_path(entry.path()) {
                results.push(fe);
            }
        }
    }

    results
}

pub async fn search_async(
    root: PathBuf,
    query: String,
    max_results: usize,
    include_hidden: bool,
) -> Vec<FileEntry> {
    let handle = tokio::task::spawn_blocking(move || {
        search_blocking(root, query, max_results, include_hidden)
    });
    handle.await.unwrap_or_default()
}
