use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::error;

/// Single item dragged into the list
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DropItem {
    pub display_name: String,
    pub data: String,
    pub mime_type: String,
}

fn get_default_path() -> Option<PathBuf> {
    Some(
        match env::var("XDG_DATA_HOME") {
            Ok(home) => PathBuf::from(home),
            Err(e) => {
                error!(
                    "Failed go get XDG_DATA_HOME ({}). Falling back to $HOME/.config.",
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
        .join("dragbox/state.json"),
    )
}

pub fn parse_state(path: &PathBuf) -> Option<Vec<DropItem>> {
    fs::read_to_string(path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_else(|| {
            error!("Failed to parse state file at '{:?}'.", path.to_str());
            None
        })
}

pub fn load_default_state() -> Option<Vec<DropItem>> {
    if let Some(path) = get_default_path() {
        if fs::exists(&path).expect("Failed to check if $XDG_DATA_HOME/dragbox/state.json exists.")
        {
            parse_state(&path)
        } else {
            Some(vec![])
        }
    } else {
        error!("Failed to get DragBox's data path!");
        None
    }
}

// pub fn write_state(state_path: Option<&Path>, state: Vec<DropItem>) {
//     if let None = state_path {
//         return;
//     }
//     let Some(path) = get_state_path(state_path) else {
//         error!("Could not determine state path.");
//         return;
//     };
//     if let Some(parent) = path.parent() {
//         if let Err(e) = fs::create_dir_all(parent) {
//             error!("Failed to create state directory: {}", e);
//             return;
//         }
//     }
//
//     match serde_json::to_string_pretty(&state) {
//         Ok(json) => {
//             if let Err(e) = fs::write(&path, json) {
//                 error!("Failed to write state file: {}", e);
//             } else {
//                 println!("State written to {:?}", path);
//             }
//         }
//         Err(e) => error!("Failed to serialize state: {}", e),
//     }
// }
