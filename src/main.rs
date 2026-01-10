mod config;
mod ui;

use gtk4::{
    Application,
    gio::prelude::{ApplicationExt, ApplicationExtManual},
};

fn main() {
    let app = Application::builder()
        .application_id("ch.skew.dragbox")
        .build();

    app.connect_activate(ui::build_ui);

    app.run();
}
