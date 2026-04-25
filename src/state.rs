use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{debug, error};

/// Single item dragged into the list
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DropItem {
    pub display_name: String,
    pub data: String,
    pub mime_type: String,
}

/// State Management
/// If the current execution mode is in Persistent mode:
///   If the file does not exist, this implies initialisation. Program runs normally, and state is
///   written normally.
///   If the file does exist, but is not valid, then throw error to notify the user that the file
///   they have explicitly specified is broken.
///   If the file exists, then obviously load that and run the program

fn get_default_path() -> Option<PathBuf> {
    Some(
        match env::var("XDG_DATA_HOME") {
            Ok(home) => PathBuf::from(home),
            Err(e) => {
                error!(
                    "Failed go get XDG_DATA_HOME ({}). Falling back to $HOME/.local/state.",
                    e.to_string()
                );
                if let Ok(p) = env::var("HOME") {
                    PathBuf::from(p).join(".local/state")
                } else {
                    error!("Failed go get HOME ({}).", e.to_string());
                    return None;
                }
            }
        }
        .join("dragevac/state.json"),
    )
}

pub fn load_state(path: &Option<PathBuf>) -> Vec<DropItem> {
    let p = match path {
        Some(p) => p,
        None => &get_default_path().unwrap(),
    };
    if fs::exists(p).expect(&format!(
        "Failed to check if file at {:?} exists.",
        p.to_str()
    )) {
        debug!("Loading state from file {:?}.", p);
        match fs::read_to_string(p) {
            Ok(s) => match serde_json::from_str::<Vec<DropItem>>(&s) {
                Ok(c) => c,
                Err(e) => panic!("Failed to parse state file: {}", e.to_string()),
            },
            Err(e) => panic!("Failed to read state file: {}", e.to_string()),
        }
    } else {
        debug!("State file does not exist. Initialising with empty state.");
        vec![]
    }
}

// pub fn write_config(config_path: Option<&Path>, config: Config) {
//     let path = match config_path {
//         Some(p) => p.to_path_buf(),
//         None => get_default_config_path().expect("Cannot find config storage location!"),
//     };
//
//     if let Some(parent) = path.parent() {
//         if let Err(e) = fs::create_dir_all(parent) {
//             error!("Failed to create config directory: {}", e);
//             return;
//         }
//     }
//
//     match serde_json::to_string_pretty(&config) {
//         Ok(json) => {
//             if let Err(e) = fs::write(&path, json) {
//                 error!("Failed to write config file: {}", e);
//             } else {
//                 println!("Config written to {:?}", path);
//             }
//         }
//         Err(e) => error!("Failed to serialize config: {}", e),
//     }
// }
