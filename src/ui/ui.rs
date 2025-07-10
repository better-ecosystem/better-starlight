use std::process::exit;

use adw::{ApplicationWindow, prelude::AdwApplicationWindowExt};
use gtk::{
    gdk::Key, prelude::{BoxExt, EntryExt, GtkWindowExt, ListBoxRowExt, WidgetExt}, Box, EventControllerKey
};
use lazy_static::lazy_static;

use crate::utils::logger::{LogLevel, Logger};

lazy_static! {
    static ref LOG: Logger = Logger::new("ui", LogLevel::Debug);
}

pub fn build_main_ui(app: &adw::Application) -> ApplicationWindow {
    let window = adw::ApplicationWindow::new(app);
    window.set_title(Some("starlight"));
    window.set_size_request(600, 600);

    LOG.debug("window layer setup complete");

    // Close app when presses ESCAPE button
    let key_controller = EventControllerKey::new();
    let window_clone = window.clone();
    key_controller.connect_key_pressed(move |_controller, _key, _keycode, _state| match _key {
        Key::Escape => {
            LOG.debug("application closed");
            window_clone.close();
            true;
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

    // search entry
    let search_entry = gtk::Entry::new();
    search_entry.set_placeholder_text(Some("Search..."));
    search_entry.set_icon_from_icon_name(
        gtk::EntryIconPosition::Primary,
        Some("system-search-symbolic"),
    );
    search_entry.add_css_class("flat");
    search_entry.add_css_class("search-entry");
    search_entry.set_hexpand(true);

    // list box holding apps
    let list_box = gtk::ListBox::new();
    list_box.set_vexpand(true);
    list_box.add_css_class("apps-list");

    // just for test
    for _ in 0..4 {
        let row = create_app_row("App name", "description if have any");
        list_box.append(&row);
    }

    content.append(&search_entry);
    content.append(&list_box);

    window.add_controller(key_controller);
    window.set_content(Some(&content));
    window.present();

    window
}


fn create_app_row(name: &str, description: &str) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.set_margin_top(4);
    row.set_margin_bottom(4);
    row.set_margin_start(8);
    row.set_margin_end(8);


    let row_box = Box::new(gtk::Orientation::Horizontal, 0);
    row_box.set_margin_top(8);
    row_box.set_margin_bottom(8);
    row_box.set_margin_start(8);
    row_box.set_margin_end(8);

    // app icon
    let icon = gtk::Image::from_icon_name("applications-system-symbolic");
    icon.set_pixel_size(48);

    // app name and description
    let app_box = Box::new(gtk::Orientation::Vertical, 2);
    app_box.set_margin_top(8);
    app_box.set_margin_bottom(8);
    app_box.set_margin_start(8);
    app_box.set_margin_end(8);

    let app_title = gtk::Label::new(Some(name));
    app_title.add_css_class("title");

    let app_desc = gtk::Label::new(Some(description));
    app_desc.add_css_class("dim-label");

    app_box.append(&app_title);
    app_box.append(&app_desc);

    row_box.append(&icon);
    row_box.append(&app_box);

    row.set_child(Some(&row_box));
    row.add_css_class("card");
    row
}
