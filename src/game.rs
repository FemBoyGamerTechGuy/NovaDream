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
    pub exe_path:     Option<String>,     // for local games
    pub launch_mode:  LaunchMode,
    pub runner:       Option<String>,     // Wine/Proton runner name override
    pub installed:    bool,
    pub play_time:    u64,                // seconds
    pub last_played:  Option<i64>,        // unix timestamp
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
