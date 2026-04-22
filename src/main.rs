mod config;
mod logging;
mod ui;

use clap::Parser;
use gtk4::{
    Application,
    gio::prelude::{ApplicationExt, ApplicationExtManual},
};

use crate::ui::build_ui;

#[derive(Parser)]
#[command(
    name = "dragbox",
    about = "Quickly drag and drop across multiple windows without having to hold your left mouse button down!"
)]
struct Args {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    logging::set_verbose(args.verbose);

    let app = Application::builder()
        .application_id("ch.skew.dragbox")
        .build();

    app.connect_activate(move |app| build_ui(app));

    app.run_with_args::<String>(&[]);
}
