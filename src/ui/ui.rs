use crate::{
    StartMode,
    ui::{
        states::AppState,
        ui_helper::{create_app_row, create_web_search_row},
    },
    utils::{
        command::{get_executables_from_path, run_command},
        logger::{LogLevel, Logger},
        web::WebSearchManager,
    },
};
use adw::{ApplicationWindow, prelude::AdwApplicationWindowExt};
use gtk::{
    Box, Entry, EventControllerKey, Label, ListBox, ScrolledWindow, Spinner, gdk::Key, prelude::*,
};
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::process::exit;
use std::rc::Rc;

lazy_static! {
    static ref LOG: Logger = Logger::new("ui", LogLevel::Debug);
}

pub fn build_main_ui(app: &adw::Application, start_mode: StartMode) -> ApplicationWindow {
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
    let prefix_label = Label::new(None);
    prefix_label.add_css_class("search-prefix");
    let search_entry = Entry::new();

    match start_mode {
        StartMode::Web => {
            prefix_label.set_text("web:");
            search_entry.set_placeholder_text(Some("web: Search the web..."));
        }
        StartMode::Run => {
            prefix_label.set_text("run:");
            search_entry.set_placeholder_text(Some("run: Run command..."));
        }
        StartMode::Default => {
            prefix_label.set_visible(false);
            search_entry.set_placeholder_text(Some("Search applications..."));
        }
    }

    search_entry.set_icon_from_icon_name(
        gtk::EntryIconPosition::Primary,
        Some("system-search-symbolic"),
    );
    search_entry.add_css_class("search-entry");
    search_entry.set_hexpand(true);
    search_box.append(&prefix_label);
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
    let prefix_clone = prefix_label.clone();

    search_entry.connect_changed(move |entry| {
        let query = format!("{}{}", prefix_clone.text(), entry.text());

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
            let window_clone2 = window_search.clone();

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
                    if let Some(first_row) = list_box_clone
                        .first_child()
                        .and_then(|c| c.downcast::<gtk::ListBoxRow>().ok())
                    {
                        list_box_clone.select_row(Some(&first_row));
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
                let search_results = if !web_query.is_empty() {
                    web_manager.search_engines_for_query(&web_query)
                } else {
                    web_status_label.set_text("Enter your query to search on web.");
                    Vec::new()
                };

                while let Some(child) = web_list_box.first_child() {
                    web_list_box.remove(&child);
                }

                if web_query.is_empty() {
                    web_status_label.set_visible(true);
                    web_list_box.set_visible(false);
                } else {
                    web_status_label.set_visible(false);
                    web_list_box.set_visible(true);
                }

                for result in search_results {
                    let row = create_web_search_row(&result, &web_query);
                    web_list_box.append(&row);
                }
                if let Some(first_row) = web_list_box
                    .first_child()
                    .and_then(|c| c.downcast::<gtk::ListBoxRow>().ok())
                {
                    web_list_box.select_row(Some(&first_row));
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
                    if let Some(first_row) = list_box_clone
                        .first_child()
                        .and_then(|c| c.downcast::<gtk::ListBoxRow>().ok())
                    {
                        list_box_clone.select_row(Some(&first_row));
                    }
                }
            });
        }
    });

    let list_box_clone = list_box.clone();
    search_entry.connect_activate(move |_| {
        if let Some(selected_row) = list_box_clone.selected_row() {
            selected_row.activate();
        }
    });

    // Set up app launch functionality
    let app_state_launch = app_state.clone();
    let window_launch = window.clone();
    let search_entry_launch = search_entry.clone();

    list_box.connect_row_activated(move |_list_box, row| {
        let search_entry_clone = search_entry_launch.clone();
        let query = format!("{}{}", prefix_label.text(), search_entry_clone.text());

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

                // Trigger search after loading if we have a prefix
                let current_text = search_entry_load.text().to_string();
                if !current_text.is_empty() {
                    search_entry_load.emit_activate();
                }

                search_entry_load.grab_focus();

                // Position cursor at end if we have prefilled text
                if !current_text.is_empty() {
                    search_entry_load.set_position(-1);
                }
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
