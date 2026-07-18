use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum FileOpError {
    AlreadyExists(PathBuf),
    NotFound(PathBuf),
    PermissionDenied(PathBuf),
    Io(String),
}

impl std::fmt::Display for FileOpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileOpError::AlreadyExists(p) => write!(f, "already exists: {}", p.display()),
            FileOpError::NotFound(p) => write!(f, "not found: {}", p.display()),
            FileOpError::PermissionDenied(p) => write!(f, "permission denied: {}", p.display()),
            FileOpError::Io(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for FileOpError {}

fn map_io_error(e: std::io::Error, path: &Path) -> FileOpError {
    match e.kind() {
        std::io::ErrorKind::NotFound => FileOpError::NotFound(path.to_path_buf()),
        std::io::ErrorKind::PermissionDenied => FileOpError::PermissionDenied(path.to_path_buf()),
        std::io::ErrorKind::AlreadyExists => FileOpError::AlreadyExists(path.to_path_buf()),
        _ => FileOpError::Io(e.to_string()),
    }
}

fn create_directory_blocking(path: PathBuf) -> Result<(), FileOpError> {
    if path.exists() {
        return Err(FileOpError::AlreadyExists(path));
    }
    std::fs::create_dir(&path).map_err(|e| map_io_error(e, &path))
}

fn create_empty_file_blocking(path: PathBuf) -> Result<(), FileOpError> {
    if path.exists() {
        return Err(FileOpError::AlreadyExists(path));
    }
    std::fs::File::create(&path)
        .map(|_| ())
        .map_err(|e| map_io_error(e, &path))
}

fn rename_blocking(from: PathBuf, to: PathBuf) -> Result<(), FileOpError> {
    if to.exists() {
        return Err(FileOpError::AlreadyExists(to));
    }
    std::fs::rename(&from, &to).map_err(|e| map_io_error(e, &from))
}

fn copy_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    let metadata = std::fs::symlink_metadata(src)?;
    if metadata.is_dir() {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let entry_dst = dst.join(entry.file_name());
            copy_recursive(&entry.path(), &entry_dst)?;
        }
        Ok(())
    } else if metadata.file_type().is_symlink() {
        let target = std::fs::read_link(src)?;
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, dst)?;
        }
        #[cfg(not(unix))]
        {
            std::fs::copy(src, dst)?;
            let _ = target;
        }
        Ok(())
    } else {
        std::fs::copy(src, dst)?;
        Ok(())
    }
}

fn copy_blocking(src: PathBuf, dst_dir: PathBuf) -> Result<PathBuf, FileOpError> {
    let file_name = src
        .file_name()
        .ok_or_else(|| FileOpError::Io("source path has no file name".to_string()))?;
    let mut dst = dst_dir.join(file_name);

    if dst.exists() {
        let stem = dst
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let ext = dst
            .extension()
            .map(|s| s.to_string_lossy().to_string());
        let mut counter = 1u32;
        loop {
            let candidate_name = match &ext {
                Some(ext) => format!("{stem} ({counter}).{ext}"),
                None => format!("{stem} ({counter})"),
            };
            let candidate = dst_dir.join(candidate_name);
            if !candidate.exists() {
                dst = candidate;
                break;
            }
            counter += 1;
        }
    }

    copy_recursive(&src, &dst).map_err(|e| map_io_error(e, &src))?;
    Ok(dst)
}

fn move_blocking(src: PathBuf, dst_dir: PathBuf) -> Result<PathBuf, FileOpError> {
    let file_name = src
        .file_name()
        .ok_or_else(|| FileOpError::Io("source path has no file name".to_string()))?;
    let dst = dst_dir.join(file_name);

    if dst.exists() {
        return Err(FileOpError::AlreadyExists(dst));
    }

    match std::fs::rename(&src, &dst) {
        Ok(()) => Ok(dst),
        Err(e) if e.raw_os_error() == Some(18) => {
            copy_recursive(&src, &dst).map_err(|e| map_io_error(e, &src))?;
            if src.is_dir() {
                std::fs::remove_dir_all(&src).map_err(|e| map_io_error(e, &src))?;
            } else {
                std::fs::remove_file(&src).map_err(|e| map_io_error(e, &src))?;
            }
            Ok(dst)
        }
        Err(e) => Err(map_io_error(e, &src)),
    }
}

fn delete_permanently_blocking(path: PathBuf) -> Result<(), FileOpError> {
    let metadata =
        std::fs::symlink_metadata(&path).map_err(|e| map_io_error(e, &path))?;
    if metadata.is_dir() {
        std::fs::remove_dir_all(&path).map_err(|e| map_io_error(e, &path))
    } else {
        std::fs::remove_file(&path).map_err(|e| map_io_error(e, &path))
    }
}

pub async fn create_directory(path: PathBuf) -> Result<(), FileOpError> {
    tokio::task::spawn_blocking(move || create_directory_blocking(path))
        .await
        .map_err(|e| FileOpError::Io(e.to_string()))?
}

pub async fn create_empty_file(path: PathBuf) -> Result<(), FileOpError> {
    tokio::task::spawn_blocking(move || create_empty_file_blocking(path))
        .await
        .map_err(|e| FileOpError::Io(e.to_string()))?
}

pub async fn rename_entry(from: PathBuf, to: PathBuf) -> Result<(), FileOpError> {
    tokio::task::spawn_blocking(move || rename_blocking(from, to))
        .await
        .map_err(|e| FileOpError::Io(e.to_string()))?
}

pub async fn copy_entry(src: PathBuf, dst_dir: PathBuf) -> Result<PathBuf, FileOpError> {
    tokio::task::spawn_blocking(move || copy_blocking(src, dst_dir))
        .await
        .map_err(|e| FileOpError::Io(e.to_string()))?
}

pub async fn move_entry(src: PathBuf, dst_dir: PathBuf) -> Result<PathBuf, FileOpError> {
    tokio::task::spawn_blocking(move || move_blocking(src, dst_dir))
        .await
        .map_err(|e| FileOpError::Io(e.to_string()))?
}

pub async fn delete_permanently(path: PathBuf) -> Result<(), FileOpError> {
    tokio::task::spawn_blocking(move || delete_permanently_blocking(path))
        .await
        .map_err(|e| FileOpError::Io(e.to_string()))?
}

pub fn suggest_unique_name(dir: &Path, base_name: &str) -> PathBuf {
    let candidate = dir.join(base_name);
    if !candidate.exists() {
        return candidate;
    }
    let path_ref = Path::new(base_name);
    let stem = path_ref
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| base_name.to_string());
    let ext = path_ref.extension().map(|s| s.to_string_lossy().to_string());

    let mut counter = 1u32;
    loop {
        let candidate_name = match &ext {
            Some(ext) => format!("{stem} ({counter}).{ext}"),
            None => format!("{stem} ({counter})"),
        };
        let candidate = dir.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}
