mod config;
mod logging;
mod state;
mod ui;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use gtk4::{
    Application,
    gio::prelude::{ApplicationExt, ApplicationExtManual},
};

use crate::{config::load_config, state::load_state, ui::build_ui};

#[derive(Parser)]
#[command(
    name = "dragevac",
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

#[derive(Subcommand, Default)]
enum Command {
    /// Writes default config. If config already exists, this will fill out missing fields, delete
    /// invalid fields, and write the result back.
    Config,
    /// Default execution method. The entry list is completely empty, and can be filled by dropping
    /// items into it. The list is not saved when the program exits.
    #[default]
    NoSave,
    // /// The entry list is saved into a specific directory in /tmp. The entry list state file is
    // /// updated whenever user adds or removes entries from them. This allows user to have a
    // /// persistent list that automatically resets when the computer reboots.
    // Temporary,
    /// Stores the entries in a specified location for complete permanence.
    Persistent {
        /// Location to store the state. If not specified, $XDG_DATA_HOME/dragevac/state.json will be
        /// used.
        #[arg(short, long)]
        state: Option<PathBuf>,
    },
    // /// User cannot drag items into the list. User must specify the state file and/or a preload
    // /// directory from which the list of entries will be pre-filled.
    // ReadOnly,
}

fn main() {
    let args = Args::parse();
    logging::set_verbose(args.verbose);

    let config_path = args.config;

    match args.command.unwrap_or_default() {
        Command::Config => {
            let config = load_config(config_path.as_deref());
            config::write_config(config_path.as_deref(), config);
        }
        Command::NoSave => {
            let app = Application::builder()
                .application_id("ch.skew.dragevac")
                .build();

            app.connect_activate(move |app| build_ui(app, config_path.as_deref(), vec![]));

            app.run_with_args::<String>(&[]);
        }
        Command::Persistent { state } => {
            let app = Application::builder()
                .application_id("ch.skew.dragevac")
                .build();

            app.connect_activate(move |app| {
                build_ui(app, config_path.as_deref(), load_state(&state))
            });

            app.run_with_args::<String>(&[]);
        }
    }
}
