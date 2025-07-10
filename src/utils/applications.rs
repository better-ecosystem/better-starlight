use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopEntry {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub comment: Option<String>,
    pub categories: Vec<String>,
    pub mime_type: Vec<String>,
    pub no_display: bool,
    pub hidden: bool,
    pub startup_notify: bool,
    pub terminal: bool,
    pub version: Option<String>,
    pub file_path: PathBuf,
    pub generic_name: Option<String>,
    pub keywords: Vec<String>,
    pub startup_wm_class: Option<String>,
    pub try_exec: Option<String>,
    pub r#type: String,
}

impl Default for DesktopEntry {
    fn default() -> Self {
        Self {
            name: String::new(),
            exec: String::new(),
            icon: None,
            comment: None,
            categories: Vec::new(),
            mime_type: Vec::new(),
            no_display: false,
            hidden: false,
            terminal: false,
            version: None,
            startup_notify: false,
            file_path: PathBuf::new(),
            generic_name: None,
            keywords: Vec::new(),
            startup_wm_class: None,
            try_exec: None,
            r#type: "Application".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct DesktopFileReader {
    search_paths: Vec<PathBuf>,
}

impl DesktopFileReader {
    pub fn new() -> Self {
        let mut search_paths = Vec::new();

        if let Some(home) = std::env::var_os("HOME") {
            let mut user_apps = PathBuf::from(home);
            user_apps.push(".local/share/applications");
            search_paths.push(user_apps);
        }

        let xdg_data_dirs = std::env::var("XDG_DATA_DIRS")
            .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());

        for dir in xdg_data_dirs.split(':') {
            let mut path = PathBuf::from(dir);
            path.push("applications");
            search_paths.push(path);
        }

        // flatpak directories
        if let Some(home) = std::env::var_os("HOME") {
            let mut flatpak_user = PathBuf::from(home);
            flatpak_user.push(".local/share/flatpak/exports/share/applications");
            search_paths.push(flatpak_user);
        }

        search_paths.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));

        Self { search_paths }
    }

    pub fn with_custom_paths(paths: Vec<PathBuf>) -> Self {
        Self {
            search_paths: paths,
        }
    }

    pub async fn read_all_desktop_files(
        &self,
    ) -> Result<Vec<DesktopEntry>, Box<dyn std::error::Error>> {
        let mut entries = Vec::new();
        let mut seen_files = std::collections::HashSet::new();

        for path in &self.search_paths {
            if path.exists() {
                let mut dir_entries = self.read_desktop_files_from_dir(path).await?;

                // filter duplicates names
                dir_entries.retain(|entry| {
                    if let Some(filename) = entry.file_path.file_name() {
                        seen_files.insert(filename.to_owned())
                    } else {
                        true
                    }
                });

                entries.extend(dir_entries);
            }
        }

        Ok(entries)
    }

    async fn read_desktop_files_from_dir(
        &self,
        dir: &Path,
    ) -> Result<Vec<DesktopEntry>, Box<dyn std::error::Error>> {
        let mut entries = Vec::new();
        let mut dir_reader = fs::read_dir(dir).await?;

        while let Some(entry) = dir_reader.next_entry().await? {
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                match self.parse_desktop_file(&path).await {
                    Ok(desktop_entry) => {

                        // skip hidden or no-display entries
                        if !desktop_entry.hidden && !desktop_entry.no_display {
                            entries.push(desktop_entry);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error parsing desktop file {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(entries)
    }

    async fn parse_desktop_file(
        &self,
        file_path: &Path,
    ) -> Result<DesktopEntry, Box<dyn std::error::Error>> {
        let file = fs::File::open(file_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut entry = DesktopEntry::default();
        entry.file_path = file_path.to_path_buf();

        let mut in_desktop_entry_section = false;

        while let Some(line) = lines.next_line().await? {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line == "[Desktop Entry]" {
                in_desktop_entry_section = true;
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                in_desktop_entry_section = false;
                continue;
            }

            if !in_desktop_entry_section {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "Name" => entry.name = value.to_string(),
                    "Exec" => entry.exec = value.to_string(),
                    "Icon" => entry.icon = Some(value.to_string()),
                    "Comment" => entry.comment = Some(value.to_string()),
                    "Categories" => {
                        entry.categories = value
                            .split(';')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect()
                    }
                    "MimeType" => {
                        entry.mime_type = value
                            .split(';')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect()
                    }
                    "NoDisplay" => entry.no_display = value.to_lowercase() == "true",
                    "Hidden" => entry.hidden = value.to_lowercase() == "true",
                    "Terminal" => entry.terminal = value.to_lowercase() == "true",
                    "StartupNotify" => entry.startup_notify = value.to_lowercase() == "true",
                    "GenericName" => entry.generic_name = Some(value.to_string()),
                    "Keywords" => {
                        entry.keywords = value
                            .split(';')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect()
                    }
                    "StartupWMClass" => entry.startup_wm_class = Some(value.to_string()),
                    "TryExec" => entry.try_exec = Some(value.to_string()),
                    "Version" => entry.version = Some(value.to_string()),
                    "Type" => entry.r#type = value.to_string(),
                    _ => {} // ignore unknown one
                }
            }
        }

        if entry.name.is_empty() {
            return Err("Desktop file missing required Name field".into());
        }

        if entry.exec.is_empty() {
            return Err("Desktop file missing required Exec field".into());
        }

        Ok(entry)
    }

    pub async fn find_desktop_file_by_name(
        &self,
        name: &str,
    ) -> Result<Option<DesktopEntry>, Box<dyn std::error::Error>> {
        let entries = self.read_all_desktop_files().await?;
        Ok(entries.into_iter().find(|entry| entry.name == name))
    }

    pub async fn search_desktop_files(
        &self,
        query: &str,
    ) -> Result<Vec<DesktopEntry>, Box<dyn std::error::Error>> {
        let entries = self.read_all_desktop_files().await?;
        let query_lower = query.to_lowercase();

        Ok(entries
            .into_iter()
            .filter(|entry| {
                entry.name.to_lowercase().contains(&query_lower)
                    || entry
                        .comment
                        .as_ref()
                        .map_or(false, |c| c.to_lowercase().contains(&query_lower))
                    || entry
                        .generic_name
                        .as_ref()
                        .map_or(false, |g| g.to_lowercase().contains(&query_lower))
                    || entry
                        .keywords
                        .iter()
                        .any(|k| k.to_lowercase().contains(&query_lower))
            })
            .collect())
    }
}
