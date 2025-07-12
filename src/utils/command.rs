use crate::utils::logger::{LogLevel, Logger};

lazy_static::lazy_static! {
    static ref LOG: Logger = Logger::new("command", LogLevel::Debug);
}

pub async fn get_executables_from_path() -> Vec<String> {
    let paths = std::env::var("PATH").unwrap_or_default();
    let mut executables = Vec::new();

    for path in paths.split(':') {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let file = entry.path();
                if file.is_file() && is_executable(&file) {
                    if let Some(name) = file.file_name().and_then(|n| n.to_str()) {
                        executables.push(name.to_string());
                    }
                }
            }
        }
    }

    executables.sort_unstable();
    executables.dedup();
    executables
}

fn is_executable(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

pub fn run_command(command: &str) {
    let mut cmd = std::process::Command::new("sh");
    cmd.arg("-c").arg(command);

    cmd.stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null());

    if let Err(e) = cmd.spawn() {
        LOG.error(&format!("failed to run '{}': {:?}", command, e));
    } else {
        LOG.debug(&format!("launched '{}' successfully", command));
    }
}

