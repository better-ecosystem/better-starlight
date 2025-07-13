use crate::utils::logger::{LogLevel, Logger};
use glib::Bytes;
use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_USER, gdk};
use lazy_static::lazy_static;

lazy_static! {
    static ref LOG: Logger = Logger::new("style", LogLevel::Debug);
}

pub fn load_css() {
    let css_provider = CssProvider::new();
    let custom_css = dirs::config_dir()
        .unwrap_or_default()
        .join("starlight")
        .join("starlight.css");

    if custom_css.exists() {
        css_provider.load_from_path(&custom_css);
        LOG.debug(&format!("loading css from {}", custom_css.display()));
    } else {
        let default_css = include_str!("style.css");
        let bytes = Bytes::from(default_css.as_bytes());
        css_provider.load_from_bytes(&bytes);
        LOG.debug("loading default css");
    }

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to display"),
        &css_provider,
        STYLE_PROVIDER_PRIORITY_USER,
    );
}
