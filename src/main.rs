pub mod utils;

use adw::prelude::AdwApplicationWindowExt;
use gtk::prelude::*;
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
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

    setup_layer_shell(&window);
    LOG.debug("Window layer setup complete");

    let search_entry = gtk::Entry::new();
    window.set_content(Some(&search_entry));

    window.present(); 
}

// Setup as a layershell rather than a normal window
fn setup_layer_shell(window: &adw::ApplicationWindow) {
    LayerShell::init_layer_shell(window);
    LayerShell::set_layer(window, Layer::Overlay);
    LayerShell::set_keyboard_mode(window, KeyboardMode::OnDemand);
}

