use crate::utils::{
    applications::{ApplicationManager, DesktopApplication},
    logger::{LogLevel, Logger},
};
use adw::{ApplicationWindow, prelude::AdwApplicationWindowExt};
use gtk::{
    Box, Entry, EventControllerKey, Label, ListBox, ScrolledWindow, Spinner, gdk::Key, prelude::*,
};
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::process::exit;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static! {
    static ref LOG: Logger = Logger::new("ui", LogLevel::Debug);
}

pub struct AppState {
    pub app_manager: Arc<RwLock<ApplicationManager>>,
    pub filtered_apps: RefCell<Vec<DesktopApplication>>,
    pub current_search: RefCell<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            app_manager: Arc::new(RwLock::new(ApplicationManager::new())),
            filtered_apps: RefCell::new(Vec::new()),
            current_search: RefCell::new(String::new()),
        }
    }
}

pub fn build_main_ui(app: &adw::Application) -> ApplicationWindow {
    let window = adw::ApplicationWindow::new(app);
    window.set_title(Some("starlight"));
    window.set_size_request(600, 600);
    LOG.debug("window layer setup complete");

    // create app state
    let app_state = Rc::new(AppState::new());

    // Close app when presses ESCAPE button
    let key_controller = EventControllerKey::new();
    let window_clone = window.clone();
    key_controller.connect_key_pressed(move |_controller, key, _keycode, _state| match key {
        Key::Escape => {
            LOG.debug("application closed");
            window_clone.close();
            exit(0);
        }
        _ => false.into(),
    });

    // mainbox that holds all components
    let content = Box::new(gtk::Orientation::Vertical, 10);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_css_classes(&["content"]);

    // search entry
    let search_box = Box::new(gtk::Orientation::Horizontal, 0);
    let search_entry = Entry::new();
    search_entry.set_placeholder_text(Some("Search applications..."));
    search_entry.set_icon_from_icon_name(
        gtk::EntryIconPosition::Primary,
        Some("system-search-symbolic"),
    );
    search_entry.add_css_class("search-entry");
    search_entry.set_hexpand(true);
    search_box.append(&search_entry);

    // Loading indicator
    let loading_box = Box::new(gtk::Orientation::Horizontal, 10);
    loading_box.set_halign(gtk::Align::Center);
    loading_box.set_valign(gtk::Align::Center);
    loading_box.set_vexpand(true);

    let spinner = Spinner::new();
    spinner.set_spinning(true);
    let loading_label = Label::new(Some("Loading applications..."));
    loading_label.add_css_class("dim-label");

    loading_box.append(&spinner);
    loading_box.append(&loading_label);

    // list box holding apps
    let list_box = ListBox::new();
    list_box.set_vexpand(true);
    list_box.add_css_class("apps-list");

    // status label for when no apps are found
    let status_label = Label::new(Some("No applications found"));
    status_label.add_css_class("dim-label");
    status_label.set_halign(gtk::Align::Center);
    status_label.set_valign(gtk::Align::Center);
    status_label.set_visible(false);

    let scroll_content = Box::new(gtk::Orientation::Vertical, 10);
    scroll_content.append(&loading_box);
    scroll_content.append(&list_box);
    scroll_content.append(&status_label);

    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_css_classes(&["scrolled-window"]);
    scrolled_window.set_child(Some(&scroll_content));

    content.append(&search_box);
    content.append(&scrolled_window);

    window.add_controller(key_controller);
    window.set_content(Some(&content));

    // setup search functionality
    let app_state_search = app_state.clone();
    let list_box_search = list_box.clone();
    let status_label_search = status_label.clone();

    search_entry.connect_changed(move |entry| {
        let query = entry.text().to_string();
        app_state_search.current_search.replace(query.clone());

        // update the list based on search
        let manager = app_state_search.app_manager.clone();
        let list_box_clone = list_box_search.clone();
        let status_label_clone = status_label_search.clone();

        glib::spawn_future_local(async move {
            let manager = manager.read().await;
            let apps = if query.is_empty() {
                manager.get_applications()
            } else {
                manager.search_applications(&query)
            };

            // clear existing items
            while let Some(child) = list_box_clone.first_child() {
                list_box_clone.remove(&child);
            }

            if apps.is_empty() {
                status_label_clone.set_visible(true);
                list_box_clone.set_visible(false);
                if query.is_empty() {
                    status_label_clone.set_text("No applications installed");
                } else {
                    status_label_clone.set_text(&format!("No applications found for '{}'", query));
                }
            } else {
                status_label_clone.set_visible(false);
                list_box_clone.set_visible(true);

                for app in apps {
                    let row = create_app_row(app);
                    list_box_clone.append(&row);
                }
            }
        });
    });

    // Set up app launch functionality
    let app_state_launch = app_state.clone();
    list_box.connect_row_activated(move |_list_box, row| {
        if let Some(app_name) = Some(row.widget_name().to_string()) {
            let manager = app_state_launch.app_manager.clone();
            let app_name = app_name.to_string();

            glib::spawn_future_local(async move {
                let manager = manager.read().await;
                if let Some(app) = manager.get_application(&app_name) {
                    if let Err(e) = manager.launch_application(app).await {
                        LOG.error(&format!("Failed to launch application: {:?}", e));
                    }
                }
            });
        }
    });

    // load applications asynchronously
    let app_state_load = app_state.clone();
    let search_entry_load = search_entry.clone();
    let loading_box_load = loading_box.clone();
    let list_box_load = list_box.clone();
    let status_label_load = status_label.clone();

    glib::spawn_future_local(async move {
        LOG.debug("Starting application loading...");

        let mut manager = app_state_load.app_manager.write().await;
        match manager.load_applications().await {
            Ok(_) => {
                LOG.debug(&format!(
                    "Successfully loaded {} applications",
                    manager.count()
                ));

                loading_box_load.set_visible(false);

                // show the loaded list
                list_box_load.set_visible(true);

                // initial trigger for search
                // search_entry_load.set_text(&search_entry.text());

                search_entry_load.grab_focus();
            }
            Err(e) => {
                LOG.error(&format!("Failed to load applications: {:?}", e));
                loading_box_load.set_visible(false);
                status_label_load.set_text("Failed to load applications");
                status_label_load.set_visible(true);
            }
        }
    });

    window.present();
    window
}

fn create_app_row(app: &DesktopApplication) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.set_margin_top(4);
    row.set_margin_bottom(4);
    row.set_margin_start(8);
    row.set_margin_end(8);

    // Set widget name to app identifier for launch functionality
    let app_id = app
        .desktop_file_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    row.set_widget_name(&app_id);

    let row_box = Box::new(gtk::Orientation::Horizontal, 0);
    row_box.set_margin_top(8);
    row_box.set_margin_bottom(8);
    row_box.set_margin_start(8);
    row_box.set_margin_end(8);

    // app icon
    let icon = if let Some(icon_name) = &app.icon {
        // try to load the specific icon, fallback to default
        let image = gtk::Image::new();
        if icon_name.starts_with('/') {
            image.set_from_file(Some(icon_name));
        } else {
            image.set_icon_name(Some(icon_name));
        }

        // if failed to load or no icon found use default icon
        if image.paintable().is_none() {
            image.set_icon_name(Some("application-x-executable"));
        }

        image
    } else {
        gtk::Image::from_icon_name("application-x-executable")
    };

    icon.set_pixel_size(32);

    // app name and description
    let app_box = Box::new(gtk::Orientation::Vertical, 2);
    app_box.set_margin_top(8);
    app_box.set_margin_bottom(8);
    app_box.set_margin_start(8);
    app_box.set_margin_end(8);
    app_box.set_hexpand(true);

    let app_title = Label::new(Some(&app.name));
    app_title.set_halign(gtk::Align::Start);
    app_title.add_css_class("title");

    // use app comments or GenericName as description
    let description = app
        .comment
        .as_ref()
        .or(app.generic_name.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("Application");

    let app_desc = Label::new(Some(description));
    app_desc.set_halign(gtk::Align::Start);
    app_desc.add_css_class("dim-label");
    app_desc.set_ellipsize(gtk::pango::EllipsizeMode::End);

    app_box.append(&app_title);
    app_box.append(&app_desc);

    if !app.categories.is_empty() {
        let categories_text = app.categories.join(", ");
        let categories_label = Label::new(Some(&categories_text));
        categories_label.set_halign(gtk::Align::Start);
        categories_label.add_css_class("dim-label");
        categories_label.add_css_class("caption");
        categories_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        app_box.append(&categories_label);
    }

    row_box.append(&icon);
    row_box.append(&app_box);

    row.set_child(Some(&row_box));
    row.add_css_class("card");
    row.set_activatable(true);

    row
}
