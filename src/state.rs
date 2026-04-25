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

#[derive(Clone)]
pub enum StateLocation {
    NoSave,
    Temporary,
    Persistent(PathBuf),
    PersistentDefault,
}

impl StateLocation {
    fn get_save_location(&self) -> Option<PathBuf> {
        match &self {
            StateLocation::NoSave => None,
            StateLocation::Temporary => Some(PathBuf::from("/tmp/dragevac/state.json")),
            StateLocation::Persistent(path_buf) => Some(path_buf.to_path_buf()),
            StateLocation::PersistentDefault => Some(
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
            ),
        }
    }
    pub fn load_state(&self) -> Vec<DropItem> {
        let Some(p) = self.get_save_location() else {
            return vec![];
        };
        if fs::exists(&p).expect(&format!(
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
    pub fn write_state(&self, state: &[DropItem]) {
        let Some(p) = self.get_save_location() else {
            return;
        };

        if let Some(parent) = p.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                error!("Failed to create state directory: {}", e);
                return;
            }
        }

        match serde_json::to_string_pretty(state) {
            Ok(json) => {
                if let Err(e) = fs::write(&p, json) {
                    error!("Failed to write state file: {}", e);
                } else {
                    debug!("State written to {:?}", p);
                }
            }
            Err(e) => error!("Failed to serialize state: {}", e),
        }
    }
}
