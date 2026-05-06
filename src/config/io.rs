use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::{debug, error};

fn get_default_config_path() -> Option<PathBuf> {
    Some(
        match std::env::var("XDG_CONFIG_HOME") {
            Ok(home) => PathBuf::from(home),
            Err(e) => {
                error!(
                    "Failed go get XDG_CONFIG_HOME ({}). Falling back to $HOME/.config.",
                    e.to_string()
                );
                if let Ok(p) = std::env::var("HOME") {
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
