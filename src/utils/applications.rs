use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tokio::fs as async_fs;
use tokio::task;
use futures::future::join_all;
use crate::utils::logger::{LogLevel, Logger};

lazy_static::lazy_static! {
    static ref LOG: Logger = Logger::new("applications", LogLevel::Debug);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopApplication {
    pub name: String,
    pub generic_name: Option<String>,
    pub comment: Option<String>,
    pub exec: String,
    pub icon: Option<String>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub mime_types: Vec<String>,
    pub startup_notify: bool,
    pub no_display: bool,
    pub hidden: bool,
    pub terminal: bool,
    pub startup_wm_class: Option<String>,
    pub desktop_file_path: PathBuf,
    pub try_exec: Option<String>,
    pub path: Option<String>,
    pub actions: Vec<String>,
}

impl Default for DesktopApplication {
    fn default() -> Self {
        Self {
            name: String::new(),
            generic_name: None,
            comment: None,
            exec: String::new(),
            icon: None,
            categories: Vec::new(),
            keywords: Vec::new(),
            mime_types: Vec::new(),
            startup_notify: true,
            no_display: false,
            hidden: false,
            terminal: false,
            startup_wm_class: None,
            desktop_file_path: PathBuf::new(),
            try_exec: None,
            path: None,
            actions: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum ApplicationError {
    IoError(std::io::Error),
    ParseError(String),
    InvalidDesktopFile(String),
}

impl From<std::io::Error> for ApplicationError {
    fn from(err: std::io::Error) -> Self {
        ApplicationError::IoError(err)
    }
}

pub struct ApplicationManager {
    applications: HashMap<String, DesktopApplication>,
    search_paths: Vec<PathBuf>,
}

impl ApplicationManager {
    pub fn new() -> Self {
        let mut search_paths = Vec::new();
        
        if let Ok(data_dirs) = std::env::var("XDG_DATA_DIRS") {
            for dir in data_dirs.split(':') {
                search_paths.push(PathBuf::from(dir).join("applications"));
            }
        } else {
            search_paths.push(PathBuf::from("/usr/share/applications"));
            search_paths.push(PathBuf::from("/usr/local/share/applications"));
        }
        
        if let Ok(home) = std::env::var("HOME") {
            search_paths.push(PathBuf::from(home).join(".local/share/applications"));
        }
        
        // flatpak apps
        if let Ok(home) = std::env::var("HOME") {
            search_paths.push(PathBuf::from(home).join(".local/share/flatpak/exports/share/applications"));
        }
        search_paths.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));

        Self {
            applications: HashMap::new(),
            search_paths,
        }
    }

    /// load all desktop applications from search paths asynchronously
    pub async fn load_applications(&mut self) -> Result<(), ApplicationError> {
        LOG.debug("Starting to load desktop applications...");
        
        let mut tasks = Vec::new();
        
        for path in &self.search_paths {
            if path.exists() {
                let path_clone = path.clone();
                let task = task::spawn(async move {
                    Self::scan_directory(path_clone).await
                });
                tasks.push(task);
            }
        }
        
        let results = join_all(tasks).await;
        let mut total_loaded = 0;
        
        for result in results {
            match result {
                Ok(Ok(apps)) => {
                    for app in apps {
                        // Use desktop file name as key for deduplication
                        let key = app.desktop_file_path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        self.applications.insert(key, app);
                        total_loaded += 1;
                    }
                }
                Ok(Err(e)) => {
                    LOG.error(&format!("Error loading applications: {:?}", e));
                }
                Err(e) => {
                    LOG.error(&format!("Task error: {:?}", e));
                }
            }
        }
        
        LOG.debug(&format!("Loaded {} applications", total_loaded));
        Ok(())
    }

    /// scan dirs for .desktop files
    async fn scan_directory(path: PathBuf) -> Result<Vec<DesktopApplication>, ApplicationError> {
        LOG.debug(&format!("Scanning directory: {:?}", path));
        let mut applications = Vec::new();
        
        let mut entries = async_fs::read_dir(&path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "desktop") {
                match Self::parse_desktop_file(&path).await {
                    Ok(Some(app)) => applications.push(app),
                    Ok(None) => {
                        LOG.debug(&format!("Skipped desktop file: {:?}", path));
                    }
                    Err(e) => {
                        LOG.warn(&format!("Failed to parse {:?}: {:?}", path, e));
                    }
                }
            }
        }
        
        Ok(applications)
    }

    /// parse a .desktop file into a DesktopApplication
    async fn parse_desktop_file(path: &Path) -> Result<Option<DesktopApplication>, ApplicationError> {
        let content = async_fs::read_to_string(path).await?;
        
        let mut app = DesktopApplication::default();
        app.desktop_file_path = path.to_path_buf();
        
        let mut in_desktop_entry = false;
        let mut found_desktop_entry = false;
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if line.starts_with('[') && line.ends_with(']') {
                in_desktop_entry = line == "[Desktop Entry]";
                if in_desktop_entry {
                    found_desktop_entry = true;
                }
                continue;
            }
            
            if !in_desktop_entry {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                
                match key {
                    "Name" => app.name = value.to_string(),
                    "GenericName" => app.generic_name = Some(value.to_string()),
                    "Comment" => app.comment = Some(value.to_string()),
                    "Exec" => app.exec = value.to_string(),
                    "Icon" => app.icon = Some(value.to_string()),
                    "Categories" => {
                        app.categories = value.split(';')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect();
                    }
                    "Keywords" => {
                        app.keywords = value.split(';')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect();
                    }
                    "MimeType" => {
                        app.mime_types = value.split(';')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect();
                    }
                    "StartupNotify" => app.startup_notify = value.eq_ignore_ascii_case("true"),
                    "NoDisplay" => app.no_display = value.eq_ignore_ascii_case("true"),
                    "Hidden" => app.hidden = value.eq_ignore_ascii_case("true"),
                    "Terminal" => app.terminal = value.eq_ignore_ascii_case("true"),
                    "StartupWMClass" => app.startup_wm_class = Some(value.to_string()),
                    "TryExec" => app.try_exec = Some(value.to_string()),
                    "Path" => app.path = Some(value.to_string()),
                    "Actions" => {
                        app.actions = value.split(';')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect();
                    }
                    _ => {} // ignore unknown keys
                }
            }
        }
        
        if !found_desktop_entry {
            return Err(ApplicationError::InvalidDesktopFile(
                "No [Desktop Entry] section found".to_string()
            ));
        }
        
        // skip applications that shouldn't be displayed
        if app.no_display || app.hidden || app.name.is_empty() || app.exec.is_empty() {
            return Ok(None);
        }
        
        // check if TryExec is present
        if let Some(try_exec) = &app.try_exec {
            if !Self::command_exists(try_exec) {
                LOG.debug(&format!("TryExec command not found: {}", try_exec));
                return Ok(None);
            }
        }
        
        Ok(Some(app))
    }

    fn command_exists(command: &str) -> bool {
        which::which(command).is_ok()
    }

    pub fn get_applications(&self) -> Vec<&DesktopApplication> {
        self.applications.values().collect()
    }

    /// search applications by name, description, or keywords
    pub fn search_applications(&self, query: &str) -> Vec<&DesktopApplication> {
        let query = query.to_lowercase();
        
        self.applications
            .values()
            .filter(|app| {
                app.name.to_lowercase().contains(&query)
                    || app.generic_name.as_ref().map_or(false, |name| name.to_lowercase().contains(&query))
                    || app.comment.as_ref().map_or(false, |comment| comment.to_lowercase().contains(&query))
                    || app.keywords.iter().any(|keyword| keyword.to_lowercase().contains(&query))
                    || app.categories.iter().any(|category| category.to_lowercase().contains(&query))
            })
            .collect()
    }

    pub fn get_applications_by_category(&self, category: &str) -> Vec<&DesktopApplication> {
        self.applications
            .values()
            .filter(|app| app.categories.iter().any(|cat| cat.eq_ignore_ascii_case(category)))
            .collect()
    }

    pub fn get_application(&self, name: &str) -> Option<&DesktopApplication> {
        self.applications.get(name)
    }

    /// launch an application
    pub async fn launch_application(&self, app: &DesktopApplication) -> Result<(), ApplicationError> {
        LOG.debug(&format!("Launching application: {}", app.name));
        
        let mut command = self.parse_exec_command(&app.exec);
        
        // Set working directory if specified
        if let Some(path) = &app.path {
            command.current_dir(path);
        }
        
        // handle terminal applications
        if app.terminal {

            // try to find a terminal emulator to launch terminal apps
            let terminal_emulators = ["gnome-terminal", "konsole", "foot", "alacritty", "kitty"];
            let mut terminal_cmd = None;
            
            for terminal in &terminal_emulators {
                if Self::command_exists(terminal) {
                    terminal_cmd = Some(*terminal);
                    break;
                }
            }
            
            if let Some(terminal) = terminal_cmd {
                command = tokio::process::Command::new(terminal);
                command.args(&["-e", &app.exec]);
            }
        }
        
        let result = command.spawn();
        
        match result {
            Ok(mut child) => {

                // don't wait for the child process to complete
                task::spawn(async move {
                    let _ = child.wait().await;
                });
                Ok(())
            }
            Err(e) => {
                LOG.error(&format!("Failed to launch {}: {}", app.name, e));
                Err(ApplicationError::IoError(e))
            }
        }
    }

    fn parse_exec_command(&self, exec: &str) -> tokio::process::Command {
        let cleaned_exec = exec
            .replace("%f", "")  // single file
            .replace("%F", "")  // multiple files  
            .replace("%u", "")  // single URL
            .replace("%U", "")  // multiple URLs
            .replace("%d", "")  // directory
            .replace("%D", "")  // multiple directories
            .replace("%n", "")  // single filename
            .replace("%N", "")  // multiple filenames
            .replace("%i", "")  // icon
            .replace("%c", "")  // translated name
            .replace("%k", "")  // desktop file location
            .replace("%v", "")  // device
            .replace("%%", "%"); // literal %
        
        let parts: Vec<&str> = cleaned_exec.split_whitespace().collect();
        let mut command = tokio::process::Command::new(parts[0]);
        
        if parts.len() > 1 {
            command.args(&parts[1..]);
        }
        
        command
    }

    /// refresh applications from disk
    pub async fn refresh(&mut self) -> Result<(), ApplicationError> {
        LOG.debug("Refreshing applications...");
        self.applications.clear();
        self.load_applications().await
    }

    pub fn count(&self) -> usize {
        self.applications.len()
    }
}
