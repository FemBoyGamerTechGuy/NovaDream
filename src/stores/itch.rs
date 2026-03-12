// NovaDream — Itch.io store backend (via butler CLI)
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::game::{Game, Store};
use anyhow::{Result, anyhow};
use std::process::Command;
use super::StoreBackend;

pub struct ItchStore {
    authenticated: bool,
    api_key: Option<String>,
}

impl ItchStore {
    pub fn new() -> Self {
        // Check for stored itch.io API key
        let key_path = dirs::config_dir()
            .unwrap_or_default()
            .join("NovaDream/itch_key");

        let api_key = std::fs::read_to_string(&key_path).ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let authenticated = api_key.is_some();
        Self { authenticated, api_key }
    }
}

impl StoreBackend for ItchStore {
    fn is_authenticated(&self) -> bool { self.authenticated }

    fn auth_url(&self) -> Option<String> {
        // Itch.io uses API keys — redirect to the API key generation page
        Some("https://itch.io/user/settings/api-keys".into())
    }

    fn handle_oauth_callback(&mut self, url: &str) -> Result<()> {
        // For itch.io the "callback" is the user pasting their API key
        let key = url.trim().to_string();
        if key.is_empty() { return Err(anyhow!("Empty API key")); }

        // Save key to config
        let key_path = dirs::config_dir()
            .unwrap_or_default()
            .join("NovaDream/itch_key");
        std::fs::write(&key_path, &key)?;

        self.api_key = Some(key);
        self.authenticated = true;
        Ok(())
    }

    fn fetch_library(&self) -> Result<Vec<Game>> {
        let key = self.api_key.as_deref()
            .ok_or_else(|| anyhow!("Not authenticated"))?;

        let output = Command::new("butler")
            .args(["status", "--json", key])
            .output()?;

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let mut games = vec![];

        if let Some(arr) = json["owned_keys"].as_array() {
            for item in arr {
                let game = &item["game"];
                games.push(Game {
                    id:           game["id"].to_string(),
                    title:        game["title"].as_str().unwrap_or("Unknown").to_string(),
                    store:        Store::Itch,
                    cover_url:    game["cover_url"].as_str().map(|s| s.to_string()),
                    cover_path:   None,
                    install_path: None,
                    exe_path:     None,
                    launch_mode:  crate::game::LaunchMode::Linux,
                    runner:       None,
                    installed:    false,
                    play_time:    0,
                    last_played:  None,
                    ..Default::default()
                });
            }
        }
        Ok(games)
    }

    fn launch_game(&self, game: &Game) -> Result<()> {
        if let Some(path) = &game.install_path {
            Command::new("butler")
                .args(["run", path])
                .spawn()?;
        }
        Ok(())
    }

    fn install_game(&self, game: &Game) -> Result<()> {
        let key = self.api_key.as_deref()
            .ok_or_else(|| anyhow!("Not authenticated"))?;
        let install_path = dirs::data_local_dir()
            .unwrap_or_default()
            .join("NovaDream/itch")
            .join(&game.id);
        Command::new("butler")
            .args(["install", &game.id, &install_path.to_string_lossy(), key])
            .spawn()?;
        Ok(())
    }
}
