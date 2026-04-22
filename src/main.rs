mod config;
mod logging;
mod ui;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
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

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Writes default config. If config already exists, this will fill out missing fields, delete
    /// invalid fields, and write the result back.
    Config,
}

fn main() {
    let args = Args::parse();
    logging::set_verbose(args.verbose);

    let config_path = args.config;

    match args.command {
        Some(Command::Config) => {
            config::write_config(config_path.as_deref());
        }
        None => {
            let app = Application::builder()
                .application_id("ch.skew.dragbox")
                .build();

            app.connect_activate(move |app| build_ui(app, config_path.as_deref()));

            app.run_with_args::<String>(&[]);
        }
    }
}
