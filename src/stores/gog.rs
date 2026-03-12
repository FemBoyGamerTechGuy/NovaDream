// NovaDream — GOG store backend (via gogdl CLI)
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::game::{Game, Store};
use anyhow::{Result, anyhow};
use std::process::Command;
use super::StoreBackend;

pub struct GogStore {
    authenticated: bool,
}

impl GogStore {
    pub fn new() -> Self {
        let authenticated = Command::new("gogdl")
            .args(["auth", "--check"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        Self { authenticated }
    }
}

impl StoreBackend for GogStore {
    fn is_authenticated(&self) -> bool { self.authenticated }

    fn auth_url(&self) -> Option<String> {
        Some("https://login.gog.com/auth?client_id=46899977096215655&redirect_uri=https://embed.gog.com/on_login_success?origin=client&response_type=code&layout=client2".into())
    }

    fn handle_oauth_callback(&mut self, url: &str) -> Result<()> {
        let code = url.split("code=").nth(1)
            .ok_or_else(|| anyhow!("No auth code in URL"))?
            .split('&').next()
            .unwrap_or("");

        let status = Command::new("gogdl")
            .args(["auth", "--code", code])
            .status()?;

        if status.success() {
            self.authenticated = true;
            Ok(())
        } else {
            Err(anyhow!("gogdl auth failed"))
        }
    }

    fn fetch_library(&self) -> Result<Vec<Game>> {
        let output = Command::new("gogdl")
            .args(["games", "--json"])
            .output()?;

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let mut games = vec![];

        if let Some(arr) = json.as_array() {
            for item in arr {
                games.push(Game {
                    id:           item["id"].as_str().unwrap_or("").to_string(),
                    title:        item["title"].as_str().unwrap_or("Unknown").to_string(),
                    store:        Store::Gog,
                    cover_url:    item["image"].as_str().map(|s| format!("https:{}", s)),
                    cover_path:   None,
                    install_path: item["path"].as_str().map(|s| s.to_string()),
                    exe_path:     None,
                    launch_mode:  crate::game::LaunchMode::Linux,
                    runner:       None,
                    installed:    item["installed"].as_bool().unwrap_or(false),
                    play_time:    0,
                    last_played:  None,
                    ..Default::default()
                });
            }
        }
        Ok(games)
    }

    fn launch_game(&self, game: &Game) -> Result<()> {
        Command::new("gogdl")
            .args(["launch", &game.id])
            .spawn()?;
        Ok(())
    }

    fn install_game(&self, game: &Game) -> Result<()> {
        Command::new("gogdl")
            .args(["download", &game.id])
            .spawn()?;
        Ok(())
    }
}
