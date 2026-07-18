use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManagerKind {
    Pacman,
    AptDpkg,
    Dnf,
    Eopkg,
    Xbps,
    Zypper,
}

impl PackageManagerKind {
    pub fn display_name(self) -> &'static str {
        match self {
            PackageManagerKind::Pacman => "pacman",
            PackageManagerKind::AptDpkg => "apt",
            PackageManagerKind::Dnf => "dnf",
            PackageManagerKind::Eopkg => "eopkg",
            PackageManagerKind::Xbps => "xbps",
            PackageManagerKind::Zypper => "zypper",
        }
    }
}

fn parse_os_release(content: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some(eq_pos) = line.find('=') else {
            continue;
        };
        let key = line[..eq_pos].trim().to_string();
        let mut value = line[eq_pos + 1..].trim().to_string();
        if (value.starts_with('"') && value.ends_with('"') && value.len() >= 2)
            || (value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2)
        {
            value = value[1..value.len() - 1].to_string();
        }
        result.insert(key, value);
    }
    result
}

fn detect_from_id_fields(id: &str, id_like: &str) -> Option<PackageManagerKind> {
    let haystack = format!("{id} {id_like}").to_lowercase();

    if haystack.contains("arch") || haystack.contains("manjaro") || haystack.contains("endeavouros")
    {
        return Some(PackageManagerKind::Pacman);
    }
    if haystack.contains("debian") || haystack.contains("ubuntu") {
        return Some(PackageManagerKind::AptDpkg);
    }
    if haystack.contains("fedora") || haystack.contains("rhel") || haystack.contains("centos") {
        return Some(PackageManagerKind::Dnf);
    }
    if haystack.contains("solus") {
        return Some(PackageManagerKind::Eopkg);
    }
    if haystack.contains("void") {
        return Some(PackageManagerKind::Xbps);
    }
    if haystack.contains("suse") || haystack.contains("opensuse") {
        return Some(PackageManagerKind::Zypper);
    }

    None
}

pub fn detect_package_manager() -> Option<PackageManagerKind> {
    let os_release_path = Path::new("/etc/os-release");
    let content = std::fs::read_to_string(os_release_path).ok()?;
    let fields = parse_os_release(&content);

    let id = fields.get("ID").cloned().unwrap_or_default();
    let id_like = fields.get("ID_LIKE").cloned().unwrap_or_default();

    if let Some(kind) = detect_from_id_fields(&id, &id_like) {
        return Some(kind);
    }

    detect_package_manager_by_binary_presence()
}

fn detect_package_manager_by_binary_presence() -> Option<PackageManagerKind> {
    let candidates: [(&str, PackageManagerKind); 6] = [
        ("pacman", PackageManagerKind::Pacman),
        ("apt", PackageManagerKind::AptDpkg),
        ("dnf", PackageManagerKind::Dnf),
        ("eopkg", PackageManagerKind::Eopkg),
        ("xbps-remove", PackageManagerKind::Xbps),
        ("zypper", PackageManagerKind::Zypper),
    ];

    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        for (binary_name, kind) in candidates {
            if dir.join(binary_name).is_file() {
                return Some(kind);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_arch_from_id() {
        assert_eq!(
            detect_from_id_fields("arch", ""),
            Some(PackageManagerKind::Pacman)
        );
    }

    #[test]
    fn detects_manjaro_from_id_like() {
        assert_eq!(
            detect_from_id_fields("manjaro", "arch"),
            Some(PackageManagerKind::Pacman)
        );
    }

    #[test]
    fn detects_ubuntu_from_id_like_debian() {
        assert_eq!(
            detect_from_id_fields("ubuntu", "debian"),
            Some(PackageManagerKind::AptDpkg)
        );
    }

    #[test]
    fn detects_void() {
        assert_eq!(
            detect_from_id_fields("void", ""),
            Some(PackageManagerKind::Xbps)
        );
    }

    #[test]
    fn detects_solus() {
        assert_eq!(
            detect_from_id_fields("solus", ""),
            Some(PackageManagerKind::Eopkg)
        );
    }

    #[test]
    fn unknown_distro_returns_none() {
        assert_eq!(detect_from_id_fields("someunknowndistro", ""), None);
    }

    #[test]
    fn parses_quoted_os_release_values() {
        let content = "ID=ubuntu\nID_LIKE=debian\nPRETTY_NAME=\"Ubuntu 24.04\"\n";
        let fields = parse_os_release(content);
        assert_eq!(fields.get("ID").map(|s| s.as_str()), Some("ubuntu"));
        assert_eq!(
            fields.get("PRETTY_NAME").map(|s| s.as_str()),
            Some("Ubuntu 24.04")
        );
    }
}
