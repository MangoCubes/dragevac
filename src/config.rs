use serde::{Deserialize, Serialize};

use crate::config::action::{Action, OnDrop};

pub mod action;
pub mod io;

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
