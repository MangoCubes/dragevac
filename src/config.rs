use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error;

#[derive(Deserialize, Debug, Default)]
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

#[derive(Deserialize, Debug)]
pub struct Config {
    /// CSS for the surface
    pub css: String,
    /// Text to show when there are no entries
    pub empty_text: String,
    /// Keep [`Config::empty_text`] even if there are entries in the list
    pub keep_text: bool,
    /// Where to anchor the surface on the screen
    pub anchor: Anchor,
    /// Expand along the anchored edge. Available only if user selected Top/Bottom/Left/Right for
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

fn get_config_dir() -> Option<PathBuf> {
    Some(PathBuf::from(match env::var("XDG_CONFIG_HOME") {
        Ok(home) => home,
        Err(e) => {
            error!(
                "Failed go get XDG_CONFIG_HOME ({}). Falling back to HOME.",
                e.to_string()
            );
            if let Ok(p) = env::var("XDG_CONFIG_HOME") {
                p
            } else {
                error!(
                    "Failed go get HOME ({}). Using default config.",
                    e.to_string()
                );
                return None;
            }
        }
    }))
}

pub fn load_config(config_path: Option<&Path>) -> Config {
    let path = match config_path {
        Some(p) => p.to_path_buf(),
        None => {
            let Some(dir) = get_config_dir() else {
                return Config::default();
            };
            dir.join("dragbox/config.json")
        }
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
