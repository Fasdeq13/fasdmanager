use crate::fs::distro_detect::PackageManagerKind;
use std::path::Path;
use std::process::Command;

pub fn find_owning_package(kind: PackageManagerKind, file_path: &Path) -> Option<String> {
    let path_str = file_path.to_string_lossy().to_string();

    let output = match kind {
        PackageManagerKind::Pacman => Command::new("pacman")
            .arg("-Qoq")
            .arg(&path_str)
            .output()
            .ok()?,
        PackageManagerKind::AptDpkg => Command::new("dpkg")
            .arg("-S")
            .arg(&path_str)
            .output()
            .ok()?,
        PackageManagerKind::Dnf => Command::new("rpm")
            .arg("-qf")
            .arg("--qf")
            .arg("%{NAME}\n")
            .arg(&path_str)
            .output()
            .ok()?,
        PackageManagerKind::Eopkg => Command::new("eopkg")
            .arg("search-file")
            .arg(&path_str)
            .output()
            .ok()?,
        PackageManagerKind::Xbps => Command::new("xbps-query")
            .arg("-o")
            .arg(&path_str)
            .output()
            .ok()?,
        PackageManagerKind::Zypper => Command::new("rpm")
            .arg("-qf")
            .arg("--qf")
            .arg("%{NAME}\n")
            .arg(&path_str)
            .output()
            .ok()?,
    };

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_package_name(kind, &stdout)
}

fn parse_package_name(kind: PackageManagerKind, stdout: &str) -> Option<String> {
    let first_line = stdout.lines().next()?.trim();
    if first_line.is_empty() {
        return None;
    }

    match kind {
        PackageManagerKind::Pacman => Some(first_line.to_string()),
        PackageManagerKind::AptDpkg => {
            let package_part = first_line.split(':').next()?;
            let package_name = package_part.split(',').next()?.trim();
            if package_name.is_empty() {
                None
            } else {
                Some(package_name.to_string())
            }
        }
        PackageManagerKind::Dnf | PackageManagerKind::Zypper => Some(first_line.to_string()),
        PackageManagerKind::Eopkg => {
            let package_name = first_line.split_whitespace().next()?;
            Some(package_name.trim_end_matches(':').to_string())
        }
        PackageManagerKind::Xbps => {
            let package_with_version = first_line.split(':').nth(1)?.trim();
            strip_version_suffix(package_with_version)
        }
    }
}

fn strip_version_suffix(package_with_version: &str) -> Option<String> {
    let mut parts: Vec<&str> = package_with_version.split('-').collect();
    if parts.len() < 2 {
        return Some(package_with_version.to_string());
    }
    parts.pop();
    let name = parts.join("-");
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

pub async fn find_owning_package_async(
    kind: PackageManagerKind,
    file_path: std::path::PathBuf,
) -> Option<String> {
    let handle =
        tokio::task::spawn_blocking(move || find_owning_package(kind, &file_path));
    handle.await.ok().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pacman_output() {
        assert_eq!(
            parse_package_name(PackageManagerKind::Pacman, "firefox\n"),
            Some("firefox".to_string())
        );
    }

    #[test]
    fn parses_dpkg_output() {
        assert_eq!(
            parse_package_name(
                PackageManagerKind::AptDpkg,
                "firefox: /usr/bin/firefox\n"
            ),
            Some("firefox".to_string())
        );
    }

    #[test]
    fn parses_rpm_output() {
        assert_eq!(
            parse_package_name(PackageManagerKind::Dnf, "firefox\n"),
            Some("firefox".to_string())
        );
    }

    #[test]
    fn parses_xbps_output() {
        assert_eq!(
            parse_package_name(PackageManagerKind::Xbps, "pkgname: firefox-128.0_1\n"),
            Some("firefox".to_string())
        );
    }

    #[test]
    fn empty_output_returns_none() {
        assert_eq!(parse_package_name(PackageManagerKind::Pacman, "\n"), None);
    }
}
