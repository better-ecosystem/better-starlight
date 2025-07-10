pub mod utils;

use adw::prelude::AdwApplicationWindowExt;
use gtk::prelude::*;
use lazy_static::lazy_static;
use crate::utils::logger::{LogLevel, Logger};

lazy_static! {
    static ref LOG: Logger = Logger::new("main",LogLevel::Debug);
}

fn main(){
    gtk::init().unwrap_or_else(|e|{
        LOG.error("Failed to initialize GTK");
        LOG.error(&format!("Error: {}", e));
    });
    adw::init().unwrap_or_else(|e|{
        LOG.error("Failed to initialize ADW");
        LOG.error(&format!("Error: {}", e));
    });

    let app = adw::Application::new(Some("com.btde.starlight"), Default::default());

    app.connect_activate(build_ui);
    app.run();
}

pub fn build_ui(app: &adw::Application){

    let window = adw::ApplicationWindow::new(app);
    window.set_title(Some("starlight"));
    window.set_default_size(600, 600);

    let search_entry = gtk::Entry::new();
    window.set_content(Some(&search_entry));

    window.present(); 
}
