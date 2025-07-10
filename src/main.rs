pub mod utils;

use adw::prelude::AdwApplicationWindowExt;
use clap::{Parser, ArgAction};
use gtk::{gdk::Key, prelude::*, EventControllerKey};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use lazy_static::lazy_static;

use crate::utils::logger::{LogLevel, Logger};

lazy_static! {
    static ref LOG: Logger = Logger::new("main",LogLevel::Debug);
}

// args 
#[derive(Parser, Debug)]
#[clap(author, version, long_about = None)]
struct Args{
    /// show debug logs
    #[clap(short, long, action = ArgAction::SetTrue)]
    debug: bool,
}

fn main(){

    let args = Args::parse();

    Logger::set_logging_enabled(args.debug);

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
    let args: Vec<String> = Vec::new();
    app.run_with_args(&args);
}

pub fn build_ui(app: &adw::Application){

    let window = adw::ApplicationWindow::new(app);
    window.set_title(Some("starlight"));
    window.set_default_size(600, 600);

    setup_layer_shell(&window);
    LOG.debug("Window layer setup complete");

    // Close app when presses ESCAPE button
    let key_controller = EventControllerKey::new();
    let window_clone = window.clone();
    key_controller.connect_key_pressed(move |_controller , _key, _keycode , _state| match _key{
        Key::Escape => {
            LOG.debug("Application closed");
            window_clone.close();
            true.into()
        }
        _ => false.into(),
    });
    let search_entry = gtk::Entry::new();
    search_entry.set_can_focus(false);
    
    let main_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    main_box.set_size_request(200, 400);
    main_box.append(&search_entry);
    window.set_content(Some(&main_box));
    window.add_controller(key_controller);

    window.present(); 
}

// Setup as a layershell rather than a normal window
fn setup_layer_shell(window: &adw::ApplicationWindow) {
    LayerShell::init_layer_shell(window);
    LayerShell::set_layer(window, Layer::Overlay);
    LayerShell::set_keyboard_mode(window, KeyboardMode::OnDemand);
}

