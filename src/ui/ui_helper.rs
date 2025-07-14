use crate::utils::{applications::DesktopApplication, logger::{LogLevel, Logger}, web::WebSearchResult};
use gtk::{prelude::*, Box, Label};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref LOG: Logger = Logger::new("ui_helper", LogLevel::Debug);
}

pub fn create_app_row(app: &DesktopApplication) -> gtk::ListBoxRow {
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

pub fn create_icon_from_theme(icon_name: &str) -> gtk::Image {
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

pub fn create_web_search_row(result: &WebSearchResult, _query: &str) -> gtk::ListBoxRow {
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

pub fn create_search_engine_icon(engine: &str) -> gtk::Image {
    let image = gtk::Image::new();

    let icon_path = match engine {
        "google" => "data/icons/google.png",
        "duckduckgo" => "data/icons/duckduckgo.svg",
        "youtube" => "data/icons/youtube.png",
        "stackoverflow" => "data/icons/stackoverflow.svg",
        _ => "web-browser",
    };
    if std::path::Path::new(icon_path).exists() {
        LOG.debug("using custom icons for search engines");
        image.set_from_file(Some(icon_path));
    } else {
        // fallback to deafault icon
        LOG.debug("failed to get custom icon using default icon for search engines");
        image.set_icon_name(Some("web-browser"));
    }

    image.set_icon_size(gtk::IconSize::Large);
    image.set_margin_start(5);
    image
}
