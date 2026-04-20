mod config;
mod ui;

use gtk4::{
    Application,
    gio::prelude::{ApplicationExt, ApplicationExtManual},
};

use crate::{config::load_config, ui::build_ui};

fn main() {
    let app = Application::builder()
        .application_id("ch.skew.dragbox")
        .build();

    let config = load_config();

    app.connect_activate(move |app| build_ui(app, &config));

    app.run();
}
