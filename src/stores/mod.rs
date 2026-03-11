// NovaDream — store backends
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod epic;
pub mod gog;
pub mod steam;
pub mod itch;

pub use epic::EpicStore;
pub use gog::GogStore;
pub use steam::SteamStore;
pub use itch::ItchStore;

use crate::game::Game;
use anyhow::Result;

#[allow(dead_code)]
pub trait StoreBackend {
    fn is_authenticated(&self) -> bool;
    fn auth_url(&self) -> Option<String>;
    fn handle_oauth_callback(&mut self, url: &str) -> Result<()>;
    fn fetch_library(&self) -> Result<Vec<Game>>;
    fn launch_game(&self, game: &Game) -> Result<()>;
    fn install_game(&self, game: &Game) -> Result<()>;
}
