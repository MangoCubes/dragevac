mod action;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::action::{Action, OnDrop};
use crate::{debug, error};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum Anchor {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    /// CSS for the surface
    pub css: String,
    /// Text to show when there are no entries
    pub empty_text: String,
    /// Keep [`Config::empty_text`] even if there are entries in the list
    pub keep_text: bool,
    /// Where to anchor the surface on the screen
    pub anchor: Anchor,
    /// Expand along the anchored edge. Applicable only if user selected top/bottom/left/right for
    /// [`Config::anchor`].
    pub expand: bool,
    /// If you are using tiling window manager, other windows will resize themselves to give DragEvac
    /// its own space so that other windows do not overlap with it. Applicable only if user selected
    /// top/bottom/left. Setting [`Config::expand`] to [`true`] is also recommended.
    pub exclusive: bool,
    /// Create a set of cards in which user can drop items into to trigger a certain action
    pub actions: Vec<Action>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            css: include_str!("./default.css").to_string(),
            empty_text: "Drop items here".to_string(),
            keep_text: true,
            anchor: Anchor::default(),
            expand: true,
            exclusive: true,
            actions: vec![Action {
                title: "Move to Downloads".to_string(),
                class_name: None,
                accept: vec!["text/uri-list".to_string()],
                command: vec![
                    "mv".to_string(),
                    "%ITEMS".to_string(),
                    "~/Downloads/".to_string(),
                ],
                concat: " ".to_string(),
                on_drop: OnDrop::NoAction,
                block_self_drop: false,
            }],
        }
    }
}

impl Config {
    pub fn get_edges(&self) -> (bool, bool, bool, bool) {
        let expand = self.expand;
        match self.anchor {
            Anchor::Top => (true, false, expand, expand),
            Anchor::Bottom => (false, true, expand, expand),
            Anchor::Left => (expand, expand, true, false),
            Anchor::Right => (expand, expand, false, true),
            Anchor::TopLeft => (true, false, true, false),
            Anchor::TopRight => (true, false, false, true),
            Anchor::BottomLeft => (false, true, true, false),
            Anchor::BottomRight => (false, true, false, true),
            Anchor::Center => (false, false, false, false),
        }
    }
}

fn get_default_config_path() -> Option<PathBuf> {
    Some(
        match env::var("XDG_CONFIG_HOME") {
            Ok(home) => PathBuf::from(home),
            Err(e) => {
                error!(
                    "Failed go get XDG_CONFIG_HOME ({}). Falling back to $HOME/.config.",
                    e.to_string()
                );
                if let Ok(p) = env::var("HOME") {
                    PathBuf::from(p).join(".config")
                } else {
                    error!(
                        "Failed go get HOME ({}). Using default config.",
                        e.to_string()
                    );
                    return None;
                }
            }
        }
        .join("dragevac/config.json"),
    )
}

/// Reads config from a specified directory, or from the defautl path
/// (~/.config/dragevac/config.json)
/// Returns Config object in the following scenario:
/// 1. The [`config_path`] has been specified, and the config is valid
/// 2. The [`config_path`] has not been specified, and the config is either valid, or simply does
///    not exist (in this case, we assume the user wants to use default config)
/// In other scenarios, this function will panic to reduce confusion when the config appears to
/// behave not as user intended.
pub fn load_config(config_path: Option<&Path>) -> Config {
    fn read_file(path: &Path) -> Config {
        match fs::read_to_string(&path) {
            Ok(s) => {
                if s.trim().is_empty() {
                    return Config::default();
                }
                match serde_json::from_str::<Config>(&s) {
                    Ok(c) => c,
                    Err(e) => panic!("Failed to parse the config: {}", e.to_string()),
                }
            }
            Err(e) => panic!("Failed to read config file: {}", e.to_string()),
        }
    }
    match config_path {
        Some(p) => read_file(p),
        None => {
            debug!("Config path not specified. Using default config path.");
            let Some(path) = get_default_config_path() else {
                return Config::default();
            };
            debug!("Using path: {:?}", path);
            match fs::exists(&path).expect("Failed to check if the file exists.") {
                true => read_file(&path),
                false => {
                    println!("Config file does not exist. Using default config.");
                    println!("You can generate a new config file using the following command:");
                    println!("dragevac config");
                    Config::default()
                }
            }
        }
    }
}

/// Writes config at a specified location
pub fn write_config(config_path: Option<&Path>, config: Config) {
    let path = match config_path {
        Some(p) => p.to_path_buf(),
        None => get_default_config_path().expect("Cannot find config storage location!"),
    };

    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            error!("Failed to create config directory: {}", e);
            return;
        }
    }

    match serde_json::to_string_pretty(&config) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json) {
                error!("Failed to write config file: {}", e);
            } else {
                println!("Config written to {:?}", path);
            }
        }
        Err(e) => error!("Failed to serialize config: {}", e),
    }
}
