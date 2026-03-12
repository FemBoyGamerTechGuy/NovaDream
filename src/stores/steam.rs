// NovaDream — Steam store backend (reads local Steam library)
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::game::{Game, Store};
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use super::StoreBackend;

pub struct SteamStore;

impl SteamStore {
    pub fn new() -> Self { Self }

    fn steam_root() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join(".steam/steam")
    }

    fn library_paths() -> Vec<PathBuf> {
        let mut paths = vec![Self::steam_root().join("steamapps")];

        // Parse libraryfolders.vdf for additional library locations
        let vdf = Self::steam_root()
            .join("steamapps/libraryfolders.vdf");

        if let Ok(contents) = std::fs::read_to_string(&vdf) {
            for line in contents.lines() {
                if line.trim_start().starts_with("\"path\"") {
                    if let Some(path) = line.split('"').nth(3) {
                        paths.push(PathBuf::from(path).join("steamapps"));
                    }
                }
            }
        }
        paths
    }
}

impl StoreBackend for SteamStore {
    // Steam doesn't need OAuth — we just read the local library
    fn is_authenticated(&self) -> bool { Self::steam_root().exists() }
    fn auth_url(&self) -> Option<String> { None }
    fn handle_oauth_callback(&mut self, _url: &str) -> Result<()> { Ok(()) }

    fn fetch_library(&self) -> Result<Vec<Game>> {
        let mut games = vec![];

        for lib_path in Self::library_paths() {
            if !lib_path.exists() { continue; }

            let entries = std::fs::read_dir(&lib_path)?;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("acf") { continue; }

                let contents = std::fs::read_to_string(&path).unwrap_or_default();
                let mut app_id    = String::new();
                let mut title     = String::new();
                let mut installed = false;

                for line in contents.lines() {
                    let line = line.trim();
                    if line.starts_with("\"appid\"")    { app_id    = parse_vdf_value(line); }
                    if line.starts_with("\"name\"")     { title     = parse_vdf_value(line); }
                    if line.starts_with("\"StateFlags\""){ installed = parse_vdf_value(line) == "4"; }
                }

                if app_id.is_empty() || title.is_empty() { continue; }

                let cover_url = Some(format!(
                    "https://cdn.cloudflare.steamstatic.com/steam/apps/{}/library_600x900.jpg",
                    app_id
                ));

                games.push(Game {
                    id: app_id,
                    title,
                    store: Store::Steam,
                    cover_url,
                    cover_path: None,
                    install_path: Some(lib_path.to_string_lossy().to_string()),
                    exe_path:     None,
                    launch_mode:  crate::game::LaunchMode::Linux,
                    runner:       None,
                    installed,
                    play_time: 0,
                    last_played: None,
                    ..Default::default()
                });
            }
        }
        Ok(games)
    }

    fn launch_game(&self, game: &Game) -> Result<()> {
        Command::new("steam")
            .args([&format!("steam://rungameid/{}", game.id)])
            .spawn()?;
        Ok(())
    }

    fn install_game(&self, game: &Game) -> Result<()> {
        Command::new("steam")
            .args([&format!("steam://install/{}", game.id)])
            .spawn()?;
        Ok(())
    }
}

fn parse_vdf_value(line: &str) -> String {
    line.splitn(3, '"')
        .nth(3)
        .unwrap_or("")
        .trim_matches('"')
        .to_string()
}
