use crate::utils::logger::{LogLevel, Logger};

lazy_static::lazy_static! {
    static ref LOG: Logger = Logger::new("applications", LogLevel::Debug);
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

pub fn run_in_user_terminal(command: &str) {
    let terminal_emulators = ["alacritty", "kitty", "foot", "wezterm", "gnome-terminal", "konsole", "xterm"];
    let terminal = terminal_emulators
        .iter()
        .find(|&&t| which::which(t).is_ok())
        .unwrap_or(&"xterm");

    let mut cmd = std::process::Command::new(terminal);

    if *terminal == "gnome-terminal" {
        cmd.args(&["--", "sh", "-c", command]);
    } else if *terminal == "konsole" {
        cmd.args(&["-e", command]);
    } else {
        cmd.args(&["-e", "sh", "-c", command]);
    }

    if let Err(e) = cmd.spawn() {
        LOG.error(&format!("Failed to run command: {:?}", e));
    }
}
