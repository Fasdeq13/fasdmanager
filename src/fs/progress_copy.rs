use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

const COPY_CHUNK_SIZE: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone)]
pub enum CopyEvent {
    Planning,
    Started { total_bytes: u64, total_files: u64 },
    FileStarted { name: String },
    Progress {
        bytes_done: u64,
        total_bytes: u64,
        files_done: u64,
        total_files: u64,
        bytes_per_second: f64,
    },
    FileFinished,
    Finished { destination: PathBuf },
    Cancelled,
    Failed { message: String },
}

#[derive(Clone)]
pub struct CancelToken {
    flag: Arc<AtomicBool>,
}

impl CancelToken {
    pub fn new() -> Self {
        CancelToken {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}

struct PlannedTree {
    total_bytes: u64,
    total_files: u64,
    files: Vec<PathBuf>,
    dirs: Vec<PathBuf>,
}

fn plan_tree(root: &Path) -> std::io::Result<PlannedTree> {
    let mut total_bytes = 0u64;
    let mut total_files = 0u64;
    let mut files = Vec::new();
    let mut dirs = Vec::new();

    let root_metadata = std::fs::symlink_metadata(root)?;
    if root_metadata.is_dir() {
        dirs.push(root.to_path_buf());
        for entry in walkdir::WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path == root {
                continue;
            }
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if metadata.is_dir() {
                dirs.push(path.to_path_buf());
            } else {
                total_bytes = total_bytes.saturating_add(metadata.len());
                total_files += 1;
                files.push(path.to_path_buf());
            }
        }
    } else {
        total_bytes = root_metadata.len();
        total_files = 1;
        files.push(root.to_path_buf());
    }

    Ok(PlannedTree {
        total_bytes,
        total_files,
        files,
        dirs,
    })
}

fn copy_file_chunked(
    src: &Path,
    dst: &Path,
    bytes_done_total: &AtomicU64,
    cancel: &CancelToken,
) -> std::io::Result<()> {
    let src_file = std::fs::File::open(src)?;
    let dst_file = std::fs::File::create(dst)?;

    let mut reader = BufReader::with_capacity(COPY_CHUNK_SIZE, src_file);
    let mut writer = BufWriter::with_capacity(COPY_CHUNK_SIZE, dst_file);

    let mut buffer = vec![0u8; COPY_CHUNK_SIZE];

    loop {
        if cancel.is_cancelled() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "copy cancelled by user",
            ));
        }

        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        writer.write_all(&buffer[..bytes_read])?;
        bytes_done_total.fetch_add(bytes_read as u64, Ordering::Relaxed);
    }

    writer.flush()?;

    if let Ok(perms) = std::fs::metadata(src).map(|m| m.permissions()) {
        let _ = std::fs::set_permissions(dst, perms);
    }

    Ok(())
}

fn relative_path_for(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

fn cleanup_partial_destination(dst: &Path, src_is_dir: bool) {
    if src_is_dir {
        let _ = std::fs::remove_dir_all(dst);
    } else {
        let _ = std::fs::remove_file(dst);
    }
}

fn run_copy_blocking(
    src: PathBuf,
    dst: PathBuf,
    cancel: CancelToken,
    progress_tx: tokio::sync::mpsc::UnboundedSender<CopyEvent>,
) {
    let _ = progress_tx.send(CopyEvent::Planning);

    let plan = match plan_tree(&src) {
        Ok(p) => p,
        Err(e) => {
            let _ = progress_tx.send(CopyEvent::Failed {
                message: e.to_string(),
            });
            return;
        }
    };

    let _ = progress_tx.send(CopyEvent::Started {
        total_bytes: plan.total_bytes,
        total_files: plan.total_files,
    });

    let src_is_dir = std::fs::symlink_metadata(&src)
        .map(|m| m.is_dir())
        .unwrap_or(false);

    if src_is_dir {
        if let Err(e) = std::fs::create_dir_all(&dst) {
            let _ = progress_tx.send(CopyEvent::Failed {
                message: e.to_string(),
            });
            return;
        }
        for dir in &plan.dirs {
            if dir == &src {
                continue;
            }
            let rel = relative_path_for(&src, dir);
            let target_dir = dst.join(rel);
            if let Err(e) = std::fs::create_dir_all(&target_dir) {
                let _ = progress_tx.send(CopyEvent::Failed {
                    message: e.to_string(),
                });
                return;
            }
        }
    }

    let bytes_done_total = Arc::new(AtomicU64::new(0));
    let mut files_done: u64 = 0;
    let mut last_emit = Instant::now();
    let mut bytes_at_last_emit: u64 = 0;

    for file_path in &plan.files {
        if cancel.is_cancelled() {
            let _ = progress_tx.send(CopyEvent::Cancelled);
            cleanup_partial_destination(&dst, src_is_dir);
            return;
        }

        let target_path = if src_is_dir {
            dst.join(relative_path_for(&src, file_path))
        } else {
            dst.clone()
        };

        let display_name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let _ = progress_tx.send(CopyEvent::FileStarted { name: display_name });

        if let Some(parent) = target_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let metadata = std::fs::symlink_metadata(file_path);
        let is_symlink = metadata.map(|m| m.file_type().is_symlink()).unwrap_or(false);

        let copy_result = if is_symlink {
            std::fs::read_link(file_path).and_then(|target| {
                #[cfg(unix)]
                {
                    std::os::unix::fs::symlink(target, &target_path)
                }
                #[cfg(not(unix))]
                {
                    std::fs::copy(file_path, &target_path).map(|_| ())
                }
            })
        } else {
            copy_file_chunked(file_path, &target_path, &bytes_done_total, &cancel)
        };

        if let Err(e) = copy_result {
            if e.kind() == std::io::ErrorKind::Interrupted {
                let _ = progress_tx.send(CopyEvent::Cancelled);
                cleanup_partial_destination(&dst, src_is_dir);
                return;
            }
            let _ = progress_tx.send(CopyEvent::Failed {
                message: format!("{}: {}", file_path.display(), e),
            });
            return;
        }

        files_done += 1;
        let _ = progress_tx.send(CopyEvent::FileFinished);

        let elapsed_since_last = last_emit.elapsed();
        if elapsed_since_last.as_millis() >= 120 || files_done == plan.total_files {
            let bytes_done = bytes_done_total.load(Ordering::Relaxed);
            let interval_secs = elapsed_since_last.as_secs_f64().max(0.001);
            let bytes_in_interval = bytes_done.saturating_sub(bytes_at_last_emit);
            let bytes_per_second = bytes_in_interval as f64 / interval_secs;

            let _ = progress_tx.send(CopyEvent::Progress {
                bytes_done,
                total_bytes: plan.total_bytes,
                files_done,
                total_files: plan.total_files,
                bytes_per_second,
            });
            last_emit = Instant::now();
            bytes_at_last_emit = bytes_done;
        }
    }

    let _ = progress_tx.send(CopyEvent::Finished { destination: dst });
}

pub fn start_copy_with_progress(
    src: PathBuf,
    dst_dir: PathBuf,
) -> (
    tokio::sync::mpsc::UnboundedReceiver<CopyEvent>,
    CancelToken,
) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let cancel = CancelToken::new();
    let cancel_for_task = cancel.clone();

    let file_name = src
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    let mut dst = dst_dir.join(&file_name);

    if dst.exists() {
        dst = crate::fs::ops::suggest_unique_name(
            &dst_dir,
            &file_name.to_string_lossy(),
        );
    }

    tokio::task::spawn_blocking(move || {
        run_copy_blocking(src, dst, cancel_for_task, tx);
    });

    (rx, cancel)
}
