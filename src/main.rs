mod config;
mod logging;
mod ui;

use std::path::PathBuf;

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

    /// Path to config file
    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    logging::set_verbose(args.verbose);

    let config_path = args.config;

    let app = Application::builder()
        .application_id("ch.skew.dragbox")
        .build();

    app.connect_activate(move |app| build_ui(app, config_path.as_deref()));

    app.run_with_args::<String>(&[]);
}
