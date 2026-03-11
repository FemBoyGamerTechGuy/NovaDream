// NovaDream — Epic Games store backend (via legendary CLI)
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::game::{Game, Store};
use anyhow::{Result, anyhow};
use std::process::Command;
use super::StoreBackend;

pub struct EpicStore {
    authenticated: bool,
}

impl EpicStore {
    pub fn new() -> Self {
        // Check if legendary is logged in
        let authenticated = Command::new("legendary")
            .args(["status", "--json"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        Self { authenticated }
    }
}

impl StoreBackend for EpicStore {
    fn is_authenticated(&self) -> bool { self.authenticated }

    fn auth_url(&self) -> Option<String> {
        Some("https://www.epicgames.com/id/login?redirectUrl=https://www.epicgames.com/id/api/redirect?clientId=34a02cf8f4414e29b15921876da36f9a&responseType=code".into())
    }

    fn handle_oauth_callback(&mut self, url: &str) -> Result<()> {
        // Extract auth code from redirect URL and pass to legendary
        let code = url.split("code=").nth(1)
            .ok_or_else(|| anyhow!("No auth code in URL"))?
            .split('&').next()
            .unwrap_or("");

        let status = Command::new("legendary")
            .args(["auth", "--code", code])
            .status()?;

        if status.success() {
            self.authenticated = true;
            Ok(())
        } else {
            Err(anyhow!("legendary auth failed"))
        }
    }

    fn fetch_library(&self) -> Result<Vec<Game>> {
        let output = Command::new("legendary")
            .args(["list", "--json"])
            .output()?;

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let mut games = vec![];

        if let Some(arr) = json.as_array() {
            for item in arr {
                let id    = item["app_name"].as_str().unwrap_or("").to_string();
                let title = item["title"].as_str().unwrap_or("Unknown").to_string();
                let cover = item["metadata"]["keyImages"]
                    .as_array()
                    .and_then(|imgs| imgs.iter().find(|i| i["type"] == "DieselGameBoxTall"))
                    .and_then(|i| i["url"].as_str())
                    .map(|s| s.to_string());

                games.push(Game {
                    id,
                    title,
                    store: Store::Epic,
                    cover_url: cover,
                    cover_path: None,
                    install_path: item["install_path"].as_str().map(|s| s.to_string()),
                    exe_path: None,
                    launch_mode: crate::game::LaunchMode::Linux,
                    runner: None,
                    installed: item["is_installed"].as_bool().unwrap_or(false),
                    play_time: 0,
                    last_played: None,
                });
            }
        }
        Ok(games)
    }

    fn launch_game(&self, game: &Game) -> Result<()> {
        Command::new("legendary")
            .args(["launch", &game.id])
            .spawn()?;
        Ok(())
    }

    fn install_game(&self, game: &Game) -> Result<()> {
        Command::new("legendary")
            .args(["install", &game.id, "--yes"])
            .spawn()?;
        Ok(())
    }
}
