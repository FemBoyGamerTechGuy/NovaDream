// NovaDream — config management
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn default_theme()        -> String { "catppuccin-macchiato".into() }
fn default_true()         -> bool   { true }
fn default_epic_path()    -> String { data_dir("epic") }
fn default_gog_path()     -> String { data_dir("gog") }
fn default_steam_path()   -> String { data_dir("steam") }
fn default_itch_path()    -> String { data_dir("itch") }
fn default_local_path()   -> String { data_dir("local") }

fn data_dir(store: &str) -> String {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("NovaDream")
        .join(store)
        .to_string_lossy()
        .to_string()
}

pub fn novadream_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("NovaDream")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_true")]
    pub show_tray: bool,

    #[serde(default = "default_true")]
    pub close_to_tray: bool,

    #[serde(default = "default_epic_path")]
    pub epic_library: String,

    #[serde(default = "default_gog_path")]
    pub gog_library: String,

    #[serde(default = "default_steam_path")]
    pub steam_library: String,

    #[serde(default = "default_itch_path")]
    pub itch_library: String,

    #[serde(default = "default_local_path")]
    pub local_library: String,

    #[serde(default)]
    pub default_runner: String, // name of default Wine/Proton runner

    #[serde(default)]
    pub launch_flags: String,   // extra flags passed to all game launches
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme:           default_theme(),
            show_tray:       true,
            close_to_tray:   true,
            epic_library:    default_epic_path(),
            gog_library:     default_gog_path(),
            steam_library:   default_steam_path(),
            itch_library:    default_itch_path(),
            local_library:   default_local_path(),
            default_runner:  String::new(),
            launch_flags:    String::new(),
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("NovaDream")
            .join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(text) = std::fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<Config>(&text) {
                    return cfg;
                }
            }
        }
        let cfg = Config::default();
        cfg.save();
        cfg
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, text);
        }
    }
}
