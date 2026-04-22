mod config;
mod ui;

use gtk4::{
    Application,
    gio::prelude::{ApplicationExt, ApplicationExtManual},
};

use crate::ui::build_ui;

fn main() {
    let app = Application::builder()
        .application_id("ch.skew.dragbox")
        .build();

    app.connect_activate(move |app| build_ui(app));

    app.run();
}
