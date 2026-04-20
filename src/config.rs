use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub css: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            css: include_str!("./default.css").to_string(),
        }
    }
}

fn get_config_dir() -> Option<PathBuf> {
    Some(
        PathBuf::from(match env::var("XDG_CONFIG_HOME") {
            Ok(home) => home,
            Err(e) => {
                println!(
                    "Failed go get XDG_CONFIG_HOME ({}). Falling back to HOME.",
                    e.to_string()
                );
                if let Ok(p) = env::var("XDG_CONFIG_HOME") {
                    p
                } else {
                    println!(
                        "Failed go get HOME ({}). Using default config.",
                        e.to_string()
                    );
                    return None;
                }
            }
        })
        .join(".config"),
    )
}

pub fn load_config() -> Config {
    let Some(config_path) = get_config_dir() else {
        return Config::default();
    };

    fs::read_to_string(config_path.join("dragbox/config.json"))
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}
