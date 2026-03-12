// NovaDream — local game library persistence + cover art fetching
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;
use crate::game::Game;
use crate::config::novadream_data_dir;

fn library_path() -> PathBuf {
    novadream_data_dir().join("local_games.json")
}

fn covers_dir() -> PathBuf {
    novadream_data_dir().join("covers")
}

/// Returns the expected cover path for a given sanitised title key
pub fn cover_path_for_title(title_key: &str) -> PathBuf {
    covers_dir().join(format!("{}.jpg", title_key))
}

/// Load persisted local games from disk
pub fn load_local_games() -> Vec<Game> {
    let path = library_path();
    if !path.exists() { return vec![]; }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Save all local games to disk
pub fn save_local_games(games: &[Game]) {
    let path = library_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(games) {
        let _ = std::fs::write(&path, json);
    }
}

/// Try to fetch a cover image for a game by name.
/// Tries SteamGridDB (no key needed for basic search via their public grid)
/// and falls back to a Steam CDN search.
/// Returns the local path where the image was saved, or None on failure.
pub fn fetch_cover(game_title: &str, _game_id: &str) -> Option<PathBuf> {
    let covers = covers_dir();
    let _ = std::fs::create_dir_all(&covers);

    // Key covers by sanitised title so re-adding the same game reuses the existing file
    let safe_title = crate::config::sanitise_title(game_title);
    let key = if safe_title.is_empty() { _game_id.to_string() } else { safe_title };
    let dest = covers.join(format!("{}.jpg", key));
    if dest.exists() { return Some(dest); }

    // Try Steam search first — free, no API key
    let steam_url = steam_cover_url(game_title)?;
    download_image(&steam_url, &dest)
}

/// Search Steam's app list for a matching title and return its grid cover URL
fn steam_cover_url(title: &str) -> Option<String> {
    // Use Steam's search suggest endpoint to find an app ID
    let query = urlencoding(title);
    let url = format!(
        "https://store.steampowered.com/search/suggest?term={}&f=games&cc=US&l=en",
        query
    );

    let body = http_get(&url)?;

    // Parse the HTML response — Steam returns a list of <a> tags with data-ds-appid
    // We look for the first match
    let app_id = extract_steam_appid(&body)?;

    Some(format!(
        "https://cdn.cloudflare.steamstatic.com/steam/apps/{}/library_600x900.jpg",
        app_id
    ))
}

fn extract_steam_appid(html: &str) -> Option<String> {
    // Find data-ds-appid="12345"
    let marker = "data-ds-appid=\"";
    let start  = html.find(marker)? + marker.len();
    let end    = html[start..].find('"')? + start;
    let id     = html[start..end].to_string();
    if id.is_empty() { None } else { Some(id) }
}

fn download_image(url: &str, dest: &PathBuf) -> Option<PathBuf> {
    let bytes = http_get_bytes(url)?;
    if bytes.len() < 1024 { return None; } // too small = probably an error page
    std::fs::write(dest, &bytes).ok()?;
    Some(dest.clone())
}

/// Minimal synchronous HTTP GET returning body as String
fn http_get(url: &str) -> Option<String> {
    let bytes = http_get_bytes(url)?;
    String::from_utf8(bytes).ok()
}

/// Minimal synchronous HTTP GET using curl (always available on Linux)
fn http_get_bytes(url: &str) -> Option<Vec<u8>> {
    let out = std::process::Command::new("curl")
        .args([
            "-s",           // silent
            "-L",           // follow redirects
            "--max-time", "8",
            "-A", "NovaDream/0.1 (game launcher)",
            url,
        ])
        .output()
        .ok()?;
    if out.status.success() && !out.stdout.is_empty() {
        Some(out.stdout)
    } else {
        None
    }
}

fn urlencoding(s: &str) -> String {
    s.chars().map(|c| match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
        ' ' => "+".to_string(),
        c   => format!("%{:02X}", c as u32),
    }).collect()
}
