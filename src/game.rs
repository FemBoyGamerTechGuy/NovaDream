// NovaDream — game data model
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Store {
    Epic,
    Gog,
    Steam,
    Itch,
    Local,
}

#[allow(dead_code)]
impl Store {
    pub fn label(&self) -> &str {
        match self {
            Store::Epic  => "Epic",
            Store::Gog   => "GOG",
            Store::Steam => "Steam",
            Store::Itch  => "Itch.io",
            Store::Local => "Local",
        }
    }

    pub fn badge_color(&self) -> &str {
        match self {
            Store::Epic  => "#2563eb",
            Store::Gog   => "#9333ea",
            Store::Steam => "#1e40af",
            Store::Itch  => "#e11d48",
            Store::Local => "#16a34a",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaunchMode {
    Linux,           // native Linux binary
    Windows,         // run via Wine/Proton
    Browser,         // open in web browser
}

#[allow(dead_code)]
impl LaunchMode {
    pub fn label(&self) -> &str {
        match self {
            LaunchMode::Linux   => "Linux",
            LaunchMode::Windows => "Windows (Wine/Proton)",
            LaunchMode::Browser => "Web Browser",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id:           String,
    pub title:        String,
    pub store:        Store,
    pub cover_url:    Option<String>,
    pub cover_path:   Option<String>,
    pub install_path: Option<String>,
    pub exe_path:     Option<String>,
    pub launch_mode:  LaunchMode,
    pub runner:       Option<String>,     // per-game runner override
    pub installed:    bool,
    pub play_time:    u64,
    pub last_played:  Option<i64>,

    // ── Per-game overrides ────────────────────────────────────────
    #[serde(default)]
    pub wine_prefix:  Option<String>,     // custom WINEPREFIX for this game

    #[serde(default)]
    pub launch_args:  Option<String>,     // extra launch arguments

    #[serde(default)]
    pub env_vars:     Option<String>,     // KEY=VAL pairs, newline separated

    #[serde(default)]
    pub work_dir:     Option<String>,     // working directory override

    #[serde(default)]
    pub pre_launch:   Option<String>,     // script to run before launch

    #[serde(default)]
    pub post_exit:    Option<String>,     // script to run after exit

    #[serde(default)]
    pub notes:        Option<String>,     // user notes

    #[serde(default)]
    pub hidden:       bool,               // hide from library

    #[serde(default)]
    pub favorite:     bool,               // pin to top

    #[serde(default)]
    pub use_mangohud: bool,               // wrap launch with mangohud

    #[serde(default)]
    pub use_gamemode: bool,               // wrap launch with gamemoderun

    #[serde(default)]
    pub use_wine_wayland: bool,           // use wine-wayland driver for this game
}

impl Game {
    pub fn play_time_str(&self) -> String {
        let h = self.play_time / 3600;
        let m = (self.play_time % 3600) / 60;
        if h > 0      { format!("{}h {}m", h, m) }
        else if m > 0 { format!("{}m", m) }
        else          { "Never played".into() }
    }

    pub fn last_played_str(&self) -> String {
        match self.last_played {
            None => "Never".into(),
            Some(ts) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                let diff = now - ts;
                if diff < 3600        { "Just now".into() }
                else if diff < 86400  { format!("{}h ago", diff / 3600) }
                else if diff < 604800 { format!("{}d ago", diff / 86400) }
                else                  { format!("{}w ago", diff / 604800) }
            }
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self {
            id:           String::new(),
            title:        String::new(),
            store:        Store::Local,
            cover_url:    None,
            cover_path:   None,
            install_path: None,
            exe_path:     None,
            launch_mode:  LaunchMode::Linux,
            runner:       None,
            installed:    false,
            play_time:    0,
            last_played:  None,
            wine_prefix:  None,
            launch_args:  None,
            env_vars:     None,
            work_dir:     None,
            pre_launch:   None,
            post_exit:    None,
            notes:        None,
            hidden:       false,
            favorite:     false,
            use_mangohud: false,
            use_gamemode: false,
            use_wine_wayland: false,
        }
    }
}
