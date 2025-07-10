use adw::prelude::AdwApplicationWindowExt;
use gtk::prelude::*;


fn main(){
    gtk::init().expect("Failed to init GTK");
    adw::init().expect("Failed to init ADW");

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
