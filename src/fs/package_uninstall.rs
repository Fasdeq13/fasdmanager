use crate::fs::distro_detect::PackageManagerKind;

#[derive(Debug, Clone)]
pub struct UninstallStep {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UninstallCommand {
    pub steps: Vec<UninstallStep>,
    pub display_command: String,
}

pub fn build_uninstall_command(kind: PackageManagerKind, package_name: &str) -> UninstallCommand {
    match kind {
        PackageManagerKind::Pacman => UninstallCommand {
            steps: vec![UninstallStep {
                program: "pacman".to_string(),
                args: vec![
                    "-Rns".to_string(),
                    "--noconfirm".to_string(),
                    package_name.to_string(),
                ],
            }],
            display_command: format!("pacman -Rns {package_name}"),
        },
        PackageManagerKind::AptDpkg => UninstallCommand {
            steps: vec![
                UninstallStep {
                    program: "apt-get".to_string(),
                    args: vec![
                        "remove".to_string(),
                        "--purge".to_string(),
                        "-y".to_string(),
                        package_name.to_string(),
                    ],
                },
                UninstallStep {
                    program: "apt-get".to_string(),
                    args: vec!["autoremove".to_string(), "--purge".to_string(), "-y".to_string()],
                },
            ],
            display_command: format!(
                "apt-get remove --purge -y {package_name} && apt-get autoremove --purge -y"
            ),
        },
        PackageManagerKind::Dnf => UninstallCommand {
            steps: vec![UninstallStep {
                program: "dnf".to_string(),
                args: vec![
                    "remove".to_string(),
                    "-y".to_string(),
                    package_name.to_string(),
                ],
            }],
            display_command: format!("dnf remove -y {package_name}"),
        },
        PackageManagerKind::Eopkg => UninstallCommand {
            steps: vec![UninstallStep {
                program: "eopkg".to_string(),
                args: vec![
                    "remove".to_string(),
                    "-y".to_string(),
                    package_name.to_string(),
                ],
            }],
            display_command: format!("eopkg remove -y {package_name}"),
        },
        PackageManagerKind::Xbps => UninstallCommand {
            steps: vec![UninstallStep {
                program: "xbps-remove".to_string(),
                args: vec![
                    "-R".to_string(),
                    "-y".to_string(),
                    package_name.to_string(),
                ],
            }],
            display_command: format!("xbps-remove -R -y {package_name}"),
        },
        PackageManagerKind::Zypper => UninstallCommand {
            steps: vec![UninstallStep {
                program: "zypper".to_string(),
                args: vec![
                    "remove".to_string(),
                    "-y".to_string(),
                    package_name.to_string(),
                ],
            }],
            display_command: format!("zypper remove -y {package_name}"),
        },
    }
}

pub fn is_running_as_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[derive(Debug)]
pub enum UninstallError {
    Io(String),
    NonZeroExit { stderr: String },
}

impl std::fmt::Display for UninstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UninstallError::Io(s) => write!(f, "{s}"),
            UninstallError::NonZeroExit { stderr } => write!(f, "{stderr}"),
        }
    }
}

fn run_uninstall_with_pkexec_blocking(
    command: &UninstallCommand,
) -> Result<(), UninstallError> {
    for (index, step) in command.steps.iter().enumerate() {
        let mut full_args = vec![step.program.clone()];
        full_args.extend(step.args.clone());

        let output = std::process::Command::new("pkexec")
            .args(&full_args)
            .output()
            .map_err(|e| UninstallError::Io(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if index == 0 {
                return Err(UninstallError::NonZeroExit { stderr });
            }
            log::warn!("uninstall cleanup step failed (non-fatal): {stderr}");
        }
    }
    Ok(())
}

fn run_uninstall_as_root_blocking(command: &UninstallCommand) -> Result<(), UninstallError> {
    for (index, step) in command.steps.iter().enumerate() {
        let output = std::process::Command::new(&step.program)
            .args(&step.args)
            .output()
            .map_err(|e| UninstallError::Io(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if index == 0 {
                return Err(UninstallError::NonZeroExit { stderr });
            }
            log::warn!("uninstall cleanup step failed (non-fatal): {stderr}");
        }
    }
    Ok(())
}

pub async fn run_uninstall_async(command: UninstallCommand) -> Result<(), UninstallError> {
    let already_root = is_running_as_root();
    let handle = tokio::task::spawn_blocking(move || {
        if already_root {
            run_uninstall_as_root_blocking(&command)
        } else {
            run_uninstall_with_pkexec_blocking(&command)
        }
    });
    handle
        .await
        .map_err(|e| UninstallError::Io(e.to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_pacman_command() {
        let cmd = build_uninstall_command(PackageManagerKind::Pacman, "firefox");
        assert_eq!(cmd.steps.len(), 1);
        assert_eq!(cmd.steps[0].program, "pacman");
        assert_eq!(cmd.steps[0].args, vec!["-Rns", "--noconfirm", "firefox"]);
    }

    #[test]
    fn builds_apt_command_with_autoremove_step() {
        let cmd = build_uninstall_command(PackageManagerKind::AptDpkg, "firefox");
        assert_eq!(cmd.steps.len(), 2);
        assert_eq!(cmd.steps[0].program, "apt-get");
        assert!(cmd.steps[0].args.contains(&"--purge".to_string()));
        assert!(cmd.steps[1].args.contains(&"autoremove".to_string()));
    }

    #[test]
    fn builds_xbps_command() {
        let cmd = build_uninstall_command(PackageManagerKind::Xbps, "firefox");
        assert_eq!(cmd.steps.len(), 1);
        assert_eq!(cmd.steps[0].program, "xbps-remove");
        assert!(cmd.steps[0].args.contains(&"-R".to_string()));
    }
}
