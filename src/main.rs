pub mod style;
pub mod ui;
pub mod utils;
use crate::{
    style::style::load_css,
    ui::ui::build_main_ui,
    utils::logger::{LogLevel, Logger},
};
use clap::{ArgAction, Parser};
use gtk::prelude::*;
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use lazy_static::lazy_static;

lazy_static! {
    static ref LOG: Logger = Logger::new("main", LogLevel::Debug);
}

// args
#[derive(Parser, Debug)]
#[clap(author, version, long_about = None)]
struct Args {
    /// show debug logs
    #[clap(short, long, action = ArgAction::SetTrue)]
    debug: bool,

    /// start with web search mode
    #[clap(short = 'w', long, action = ArgAction::SetTrue)]
    web: bool,

    /// start with run command mode
    #[clap(short = 'r', long, action = ArgAction::SetTrue)]
    run: bool,
}

#[derive(Clone, Copy)]
pub enum StartMode {
    Default,
    Web,
    Run,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    Logger::set_logging_enabled(args.debug);

    // determine start mode
    let start_mode = if args.web {
        StartMode::Web
    } else if args.run {
        StartMode::Run
    } else {
        StartMode::Default
    };

    gtk::init().unwrap_or_else(|e| {
        LOG.error("Failed to initialize GTK");
        LOG.error(&format!("Error: {}", e));
    });

    adw::init().unwrap_or_else(|e| {
        LOG.error("Failed to initialize ADW");
        LOG.error(&format!("Error: {}", e));
    });

    LOG.debug("application initialized");
    let app = adw::Application::new(Some("com.btde.starlight"), Default::default());

    LOG.debug("building ui");
    app.connect_activate(move |app| {
        let window = build_main_ui(app, start_mode.clone());
        setup_layer_shell(&window);
        LOG.debug("window layer setup complete");
        load_css();
        window.present();
    });

    LOG.debug("running main app");
    let args: Vec<String> = Vec::new();
    app.run_with_args(&args);
}

// Setup as a layershell rather than a normal window
fn setup_layer_shell(window: &adw::ApplicationWindow) {
    LayerShell::init_layer_shell(window);
    LayerShell::set_layer(window, Layer::Overlay);
    LayerShell::set_keyboard_mode(window, KeyboardMode::OnDemand);
}
