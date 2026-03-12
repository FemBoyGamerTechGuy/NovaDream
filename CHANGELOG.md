# Changelog

All notable changes to NovaDream will be documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---

## [0.1.1] - 2026-03-11

### Added
- **Windows game launching** via Wine or Proton-GE/UMU
- **UMU auto-download** — fetches `umu-run` from GitHub releases if not found on system, with live progress on the Play button (`⬇ UMU 47%`)
- **Runner detection** — classifies runners as UMU, Proton, or plain Wine and launches with the correct environment
- **Wine prefix auto-creation** — prefix initialised with wineboot on first launch, named after the game title (e.g. `prefixes/MiSide`)
- **Per-game settings dialog** — runner, prefix, launch mode, extra args, env vars, MangoHud, GameMode, wine-wayland per game
- **Wine-Wayland support** — plain Wine writes `HKCU\Software\Wine\Drivers → Graphics=wayland` into the prefix; Proton/UMU sets `PROTON_WAYLAND=1` + unsets `DISPLAY`
- **Screen resolution written to Wine registry** on every launch (fixes 640×480 on fresh prefixes)
- **MangoHud and GameMode** global defaults and per-game overrides
- **Minimize on launch** option
- **Playtime tracking** with optional badge on game cards
- **Game Defaults tab** — dedicated tab for runner, prefix base, overlays, behaviour and global env vars
- **NovaDream icon** — cosmic planet with Saturn-style ring, nebula and comet streak, at 16/32/48/64/128/256px
- **Desktop entry** and **AppStream metainfo**
- **Debian packaging** — `.deb` via `cargo-deb`
- **Fedora/RHEL packaging** — `.rpm` via `cargo-generate-rpm`
- **`packaging/` directory** — `build-deb.sh`, `build-rpm.sh`, `build-packages.sh`, `README.md`

### Changed
- **UI redesign** — cosmic/premium aesthetic: gradient backgrounds, glow effects, depth shadows, better typography (`Outfit`/`Rubik`)
- **Cover art keyed by game title** — re-adding the same game reuses the existing cover file instead of downloading duplicates
- **Prefix Browse button** opens at `~/.local/share/NovaDream/prefixes/` by default
- **Settings tab** slimmed down — Appearance, System Tray, Library Paths and Store Accounts only
- **Performance and runner settings** moved to the new Game Defaults tab

### Removed
- **"Create New Prefix Here" button** — prefixes are created automatically on launch
- **"Install DXVK into this prefix" button** — Proton-GE bundles DXVK already

### Fixed
- **Sidebar filter buttons were completely non-functional** *(broken since 0.1.0)* — All Games, Epic, GOG, Steam, Itch.io and Local now actually filter the library
- **wine-wayland was silently broken on Proton/UMU** *(broken since 0.1.0)* — `PROTON_WAYLAND=1` was never being set
- **All dialogs ignored the active theme** *(broken since 0.1.0)* — Add Game, Per-game Settings and Login popups now follow the active theme
- **DropDown widgets went black inside dialogs** on dark themes
- **Stop button did nothing** *(broken since 0.1.0)* — now sends SIGKILL to the entire process tree so Wine/Proton children are fully terminated
- **Duplicate cover downloads** — re-adding the same game no longer downloads the cover again
- **`serde_core` supply chain crate** removed from dependency tree (was pulled in transitively by `cargo-generate-rpm`)
- **Button states** — Play button now correctly shows `⏳ Launching…` while UMU sets up, then `■ Stop` once the game process is alive

---

## [0.1.0] - 2026-03-10

### Added
- Initial project setup
- GTK4 + libadwaita UI foundation
- Working system tray via libayatana-appindicator (Hyprland compatible)
- Multi-store support: Epic Games, GOG, Steam, Itch.io
- Unified game library view with artwork and covers
- Download, install and launch games from any supported store *(download/install not yet functional)*
- Built-in theme engine with Catppuccin, Dracula, Tokyo Night, Nord, Gruvbox and more
- Community theme support — load custom themes from `~/.local/share/NovaDream/themes/`
- Auto-generated config at `~/.config/NovaDream/config.json` on first launch
- Close to tray support
- GPL-3.0-or-later license
