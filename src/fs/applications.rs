use crate::fs::desktop_entry::{scan_desktop_applications_async, DesktopEntry};
use crate::fs::entry::FileEntry;
use crate::fs::reader::read_dir_async;
use std::path::{Path, PathBuf};

pub const CUSTOM_DISTRO_APPLICATIONS_PATH: &str = "/Applications";
pub const FALLBACK_BIN_PATH: &str = "/usr/bin";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationsSource {
    NativeApplicationsFolder,
    DesktopEntriesFallback,
}

pub fn detect_applications_source() -> ApplicationsSource {
    if Path::new(CUSTOM_DISTRO_APPLICATIONS_PATH).is_dir() {
        ApplicationsSource::NativeApplicationsFolder
    } else {
        ApplicationsSource::DesktopEntriesFallback
    }
}

pub fn applications_tab_target_path() -> PathBuf {
    match detect_applications_source() {
        ApplicationsSource::NativeApplicationsFolder => {
            PathBuf::from(CUSTOM_DISTRO_APPLICATIONS_PATH)
        }
        ApplicationsSource::DesktopEntriesFallback => PathBuf::from(FALLBACK_BIN_PATH),
    }
}

fn desktop_entry_to_file_entry(entry: DesktopEntry) -> FileEntry {
    let icon_name = entry
        .icon
        .clone()
        .unwrap_or_else(|| "application-x-executable-symbolic".to_string());

    let resolved_icon = crate::util::icons::resolve_desktop_icon_name(&icon_name);

    FileEntry::from_app_entry(
        entry.name,
        resolved_icon,
        entry.exec.unwrap_or_default(),
        entry
            .source_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default(),
        entry.source_path,
    )
}

pub async fn load_applications_listing(lang_code: String) -> Vec<FileEntry> {
    match detect_applications_source() {
        ApplicationsSource::NativeApplicationsFolder => {
            read_dir_async(applications_tab_target_path()).await.unwrap_or_default()
        }
        ApplicationsSource::DesktopEntriesFallback => {
            let entries = scan_desktop_applications_async(lang_code).await;
            entries.into_iter().map(desktop_entry_to_file_entry).collect()
        }
    }
}

pub fn launch_application(entry: &FileEntry) -> std::io::Result<()> {
    match detect_applications_source() {
        ApplicationsSource::NativeApplicationsFolder => {
            open::that_detached(&entry.path)
        }
        ApplicationsSource::DesktopEntriesFallback => {
            let Some(exec) = entry.exec_command.as_ref().filter(|s| !s.is_empty()) else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "empty exec command",
                ));
            };
            let parts = shell_words_split(exec);
            let Some((program, args)) = parts.split_first() else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "empty exec command",
                ));
            };
            std::process::Command::new(program)
                .args(args)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .map(|_| ())
        }
    }
}

fn shell_words_split(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';

    for c in input.chars() {
        if in_quotes {
            if c == quote_char {
                in_quotes = false;
            } else {
                current.push(c);
            }
        } else if c == '"' || c == '\'' {
            in_quotes = true;
            quote_char = c;
        } else if c.is_whitespace() {
            if !current.is_empty() {
                result.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}
