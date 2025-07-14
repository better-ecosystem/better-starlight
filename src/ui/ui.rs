use crate::utils::{
    applications::{ApplicationManager, DesktopApplication},
    command::{get_executables_from_path, run_command},
    logger::{LogLevel, Logger},
    web::{WebSearchManager, WebSearchResult},
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
    window.set_icon_name(Some("starlight"));
    window.set_size_request(600, 80);
    LOG.debug("window layer setup complete");

    const PAGE_SIZE: usize = 50;

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
    let content = Box::new(gtk::Orientation::Vertical, 12);
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
    scrolled_window.set_height_request(500);
    scrolled_window.set_css_classes(&["scrolled-window"]);
    scrolled_window.set_child(Some(&scroll_content));

    content.append(&search_box);

    window.add_controller(key_controller);
    window.set_content(Some(&content));

    // setup search functionality
    let app_state_search = app_state.clone();
    let list_box_search = list_box.clone();
    let status_label_search = status_label.clone();

    let window_search = window.clone();
    let content_search = content.clone();
    let scrolled_window_search = scrolled_window.clone();

    search_entry.connect_changed(move |entry| {
        let query = entry.text().to_string();

        if query.starts_with("r:") || query.starts_with("run:") {
            let cmd_query = query
                .trim_start_matches("r:")
                .trim_start_matches("run:")
                .trim()
                .to_string();

            let list_box_clone = list_box_search.clone();
            let status_label_clone = status_label_search.clone();
            let content_clone = content_search.clone();
            let scrolled_window_clone_query = scrolled_window_search.clone();
            let window_clone2 = window_search.clone(); // Create a new clone for this scope

            glib::spawn_future_local(async move {
                let commands = get_executables_from_path().await;
                let filtered = commands
                    .iter()
                    .filter(|cmd| cmd.contains(&cmd_query))
                    .take(1000)
                    .cloned()
                    .collect::<Vec<_>>();

                let total_filtered = filtered.len();
                let commands_to_show = &filtered[..PAGE_SIZE.min(total_filtered)];

                while let Some(child) = list_box_clone.first_child() {
                    list_box_clone.remove(&child);
                }

                if filtered.is_empty() {
                    status_label_clone.set_text(&format!("No matching commands '{}'", cmd_query));
                    status_label_clone.set_visible(true);
                    list_box_clone.set_visible(false);
                } else {
                    status_label_clone.set_visible(false);
                    list_box_clone.set_visible(true);

                    for cmd in commands_to_show {
                        let row = gtk::ListBoxRow::new();
                        row.set_widget_name(&cmd);
                        let label = Label::new(Some(&cmd));
                        label.set_halign(gtk::Align::Start);
                        label.add_css_class("title");
                        row.set_child(Some(&label));
                        row.add_css_class("card");
                        row.set_activatable(true);
                        list_box_clone.append(&row);
                    }

                    let filtered_commands = Rc::new(RefCell::new(filtered.clone()));
                    let offset = Rc::new(RefCell::new(PAGE_SIZE.min(filtered.len())));

                    let list_box_clone2 = list_box_clone.clone();
                    let filtered_commands_clone = filtered_commands.clone();
                    let offset_clone = offset.clone();
                    let scrolled_window_clone = scrolled_window_clone_query.clone();

                    scrolled_window_clone
                        .vadjustment()
                        .connect_value_changed(move |adjustment| {
                            let threshold = 30.0;
                            if adjustment.value() + adjustment.page_size()
                                >= adjustment.upper() - threshold
                            {
                                let offset_val = *offset_clone.borrow();
                                let filtered = filtered_commands_clone.borrow();

                                if offset_val < filtered.len() {
                                    let next_offset = (offset_val + PAGE_SIZE).min(filtered.len());

                                    for cmd in &filtered[offset_val..next_offset] {
                                        let row = gtk::ListBoxRow::new();
                                        row.set_widget_name(&cmd);
                                        let label = Label::new(Some(&cmd));
                                        label.set_halign(gtk::Align::Start);
                                        label.add_css_class("title");
                                        row.set_child(Some(&label));
                                        row.add_css_class("card");
                                        row.set_activatable(true);
                                        list_box_clone2.append(&row);
                                    }

                                    *offset_clone.borrow_mut() = next_offset;
                                }
                            }
                        });
                }

                if scrolled_window_clone_query.parent().is_none() {
                    content_clone.append(&scrolled_window_clone_query);
                }
                scrolled_window_clone_query.set_visible(true);
                animate_window_height(&window_clone2, 80, 500);
            });
        } else if query.starts_with("w:") || query.starts_with("web:") {
            let web_query = query
                .trim_start_matches("w:")
                .trim_start_matches("web:")
                .trim()
                .to_string();

            let web_list_box = list_box_search.clone();
            let web_status_label = status_label_search.clone();
            let web_content = content_search.clone();
            let web_scrolled_window = scrolled_window_search.clone();
            let web_window = window_search.clone();

            glib::spawn_future_local(async move {
                let web_manager = WebSearchManager::new();
                let search_results = if web_query.is_empty() {
                    web_manager.search_engines_for_query("")
                } else {
                    web_manager.search_engines_for_query(&web_query)
                };

                while let Some(child) = web_list_box.first_child() {
                    web_list_box.remove(&child);
                }

                web_status_label.set_visible(false);
                web_list_box.set_visible(true);

                for result in search_results {
                    let row = create_web_search_row(&result, &web_query);
                    web_list_box.append(&row);
                }

                if web_scrolled_window.parent().is_none() {
                    web_content.append(&web_scrolled_window);
                }
                web_scrolled_window.set_visible(true);
                animate_window_height(&web_window, 80, 500);
            });
        } else {
            app_state_search.current_search.replace(query.clone());

            if query.is_empty() {
                if scrolled_window_search.parent().is_none() {
                    content_search.remove(&scrolled_window_search);
                }
                scrolled_window_search.set_visible(false);
                animate_window_height(&window_search, 500, 80);
                return;
            } else {
                if !scrolled_window_search.parent().is_some() {
                    content_search.append(&scrolled_window_search);
                }
                scrolled_window_search.set_visible(true);
                animate_window_height(&window_search, 80, 500);
            }

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
                        status_label_clone
                            .set_text(&format!("No applications found for '{}'", query));
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
        }
    });

    // Set up app launch functionality
    let app_state_launch = app_state.clone();
    let window_launch = window.clone();
    let search_entry_launch = search_entry.clone();

    list_box.connect_row_activated(move |_list_box, row| {
        let search_entry_clone = search_entry_launch.clone();
        let query = search_entry_clone.text().to_string();

        if query.starts_with("r:") || query.starts_with("run:") {
            let cmd = row.widget_name().to_string();
            let search_term = query
                .trim_start_matches("r:")
                .trim_start_matches("run:")
                .trim()
                .to_string();

            // If user added args like: 'better-bar -d' use them
            let full_cmd = if search_term.contains(' ') && !cmd.contains(' ') {
                search_term
            } else if cmd.starts_with(&search_term) || search_term.len() < cmd.len() {
                cmd
            } else {
                search_term
            };

            run_command(&full_cmd);
            window_launch.close();
            exit(0);
        } else if query.starts_with("w:") || query.starts_with("web:") {
            let url = row.widget_name().to_string();
            let web_manager = WebSearchManager::new();

            match web_manager.open_url(&url) {
                Ok(_) => {
                    LOG.debug(&format!("Opened URL: {}", url));
                    window_launch.close();
                    exit(0);
                }
                Err(e) => {
                    LOG.error(&format!("Failed to open URL: {:?}", e));
                    window_launch.close();
                    exit(0);
                }
            }
        } else {
            if let Some(app_name) = Some(row.widget_name().to_string()) {
                let manager = app_state_launch.app_manager.clone();
                let app_name = app_name.to_string();
                let window_to_close = window_launch.clone();

                glib::spawn_future_local(async move {
                    let manager = manager.read().await;
                    if let Some(app) = manager.get_application(&app_name) {
                        match manager.launch_application(app).await {
                            Ok(_) => {
                                LOG.debug(&format!("launched {} sucessfully", app_name));
                                window_to_close.close();
                                exit(0);
                            }
                            Err(e) => {
                                LOG.error(&format!("Failed to launch application: {:?}", e));
                                window_to_close.close();
                                exit(0);
                            }
                        }
                    }
                });
            }
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
        if icon_name.starts_with('/') {
            let image = gtk::Image::new();
            image.set_from_file(Some(icon_name));
            if image.paintable().is_some() {
                image.set_icon_size(gtk::IconSize::Large);
                image.set_margin_start(5);
                image
            } else {
                create_icon_from_theme(icon_name)
            }
        } else {
            create_icon_from_theme(icon_name)
        }
    } else {
        LOG.warn(&format!("Failed to get icon"));
        LOG.warn(&format!("Falling back to default"));
        create_default_icon()
    };

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

fn create_icon_from_theme(icon_name: &str) -> gtk::Image {
    let image = gtk::Image::new();

    if let Some(display) = gtk::gdk::Display::default() {
        let icon_theme = gtk::IconTheme::for_display(&display);

        if icon_theme.has_icon(icon_name) {
            image.set_icon_name(Some(icon_name));
            image.set_icon_size(gtk::IconSize::Large);
            image.set_margin_start(5);
            return image;
        }
    }

    let fallback_icons = [
        icon_name,
        &format!("{}-symbolic", icon_name),
        &icon_name.to_lowercase(),
        "application-x-executable",
    ];

    for fallback in &fallback_icons {
        if let Some(display) = gtk::gdk::Display::default() {
            let icon_theme = gtk::IconTheme::for_display(&display);
            if icon_theme.has_icon(fallback) {
                image.set_icon_name(Some(fallback));
                image.set_icon_size(gtk::IconSize::Large);
                image.set_margin_start(5);
                return image;
            } else {
                LOG.warn(&format!(
                    "Failed to get icon for {} falling back to default {}",
                    icon_name, fallback
                ));
            }
        }
    }

    create_default_icon()
}

fn create_default_icon() -> gtk::Image {
    let image = gtk::Image::from_icon_name("application-x-executable");
    image.set_icon_size(gtk::IconSize::Large);
    image.set_margin_start(5);
    image
}

fn create_web_search_row(result: &WebSearchResult, _query: &str) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.set_margin_bottom(4);
    row.set_margin_bottom(4);
    row.set_margin_start(8);
    row.set_margin_end(8);

    row.set_widget_name(&result.url);

    let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    row_box.set_margin_top(8);
    row_box.set_margin_bottom(8);
    row_box.set_margin_start(8);
    row_box.set_margin_end(8);

    let icon = create_search_engine_icon(&result.search_engine);

    let content_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    content_box.set_margin_top(8);
    content_box.set_margin_bottom(8);
    content_box.set_margin_start(8);
    content_box.set_margin_end(8);
    content_box.set_hexpand(true);

    let title_label = gtk::Label::new(Some(&result.title));
    title_label.set_halign(gtk::Align::Start);
    title_label.add_css_class("title");

    let desc_label = gtk::Label::new(Some(&result.description));
    desc_label.set_halign(gtk::Align::Start);
    desc_label.add_css_class("dim-label");
    desc_label.set_ellipsize(gtk::pango::EllipsizeMode::End);

    let url_label = gtk::Label::new(Some(&result.url));
    url_label.set_halign(gtk::Align::Start);
    url_label.add_css_class("dim-label");
    url_label.add_css_class("caption");
    url_label.set_ellipsize(gtk::pango::EllipsizeMode::End);

    content_box.append(&title_label);
    content_box.append(&desc_label);
    content_box.append(&url_label);

    row_box.append(&icon);
    row_box.append(&content_box);

    row.set_child(Some(&row_box));
    row.add_css_class("card");
    row.set_activatable(true);

    row
}

fn create_search_engine_icon(engine: &str) -> gtk::Image {
    let image = gtk::Image::new();

    let icon_path = match engine {
        "google" => "data/icons/google.png",
        "duckduckgo" => "data/icons/duckduckgo.svg",
        "youtube" => "data/icons/yt.png",
        "stackoverflow" => "data/icons/stackoverflow.svg",
        _ => "web-browser",
    };
    if std::path::Path::new(icon_path).exists() {
        image.set_from_file(Some(icon_path));
    } else {
        // fallback to symbolic icon
        image.set_icon_name(Some("web-browser"));
    }

    image.set_icon_size(gtk::IconSize::Large);
    image.set_margin_start(5);
    image
}

fn animate_window_height(window: &ApplicationWindow, from: i32, to: i32) {
    let window = window.clone();
    let step = if to > from { 10 } else { -10 };
    let mut current = from;

    glib::timeout_add_local(std::time::Duration::from_millis(5), move || {
        current += step;
        window.set_default_size(600, current);

        if (step > 0 && current >= to) || (step < 0 && current <= to) {
            window.set_default_size(600, to);
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}
