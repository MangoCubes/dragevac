use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error;

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum Anchor {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    #[default]
    Center,
}

#[derive(Serialize, Deserialize, Debug)]
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            css: include_str!("./default.css").to_string(),
            empty_text: "Drop items here".to_string(),
            keep_text: true,
            anchor: Anchor::default(),
            expand: false,
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

fn get_state_path(state_path: Option<&Path>) -> Option<PathBuf> {
    match state_path {
        Some(p) => Some(p.to_path_buf()),
        None => Some(
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
        ),
    }
}

fn get_config_path(config_path: Option<&Path>) -> Option<PathBuf> {
    match config_path {
        Some(p) => Some(p.to_path_buf()),
        None => Some(
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
            .join("dragbox/config.json"),
        ),
    }
}

pub fn load_config(config_path: Option<&Path>) -> Config {
    let Some(path) = get_config_path(config_path) else {
        return Config::default();
    };

    fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_else(|| {
            error!(
                "Failed to find config file at '{:?}'. Falling back to default config.",
                path.to_str()
            );
            Config::default()
        })
}

pub fn write_config(config_path: Option<&Path>) {
    let Some(path) = get_config_path(config_path) else {
        error!("Could not determine config path.");
        return;
    };

    let config = load_config(config_path);

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
