use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub css: Option<String>,
}

fn get_config_dir() -> PathBuf {
    PathBuf::from(match env::var("XDG_CONFIG_HOME") {
        Ok(home) => home,
        Err(e) => {
            println!(
                "Failed go get XDG_CONFIG_HOME ({}). Falling back to HOME.",
                e.to_string()
            );
            env::var("XDG_CONFIG_HOME").expect("You don't have HOME set too??")
        }
    })
    .join(".config")
}

pub fn load_config() -> Config {
    let config_path = get_config_dir().join("dragbox/config.json");

    fs::read_to_string(config_path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}
