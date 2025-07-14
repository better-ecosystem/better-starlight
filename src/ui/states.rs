use crate::utils::applications::ApplicationManager;
use std::{cell::RefCell, rc::Rc, sync::Arc};
use tokio::sync::RwLock;

pub struct AppState {
    pub app_manager: Arc<RwLock<ApplicationManager>>,
    pub filtered_apps: RefCell<Vec<crate::utils::applications::DesktopApplication>>,
    pub current_search: RefCell<String>,
}

impl AppState {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            app_manager: Arc::new(RwLock::new(ApplicationManager::new())),
            filtered_apps: RefCell::new(Vec::new()),
            current_search: RefCell::new(String::new()),
        })
    }
}
