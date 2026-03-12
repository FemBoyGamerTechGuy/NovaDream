<div align="center">

# NovaDream

**A cosmic void-themed game launcher for Linux with multi-store support.**

[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blueviolet?style=flat-square)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![GTK4](https://img.shields.io/badge/UI-GTK4-4a86cf?style=flat-square&logo=gnome)](https://gtk.org/)
[![Part of Faded Dream](https://img.shields.io/badge/part%20of-Faded%20Dream-purple?style=flat-square)](https://github.com/FemBoyGamerTechGuy/Faded-Dream-dotfiles)

</div>

---

## Overview

NovaDream is a fast, native Linux game launcher built with Rust and GTK4. It brings together your Epic Games, GOG, Steam, Itch.io and local game libraries in one place, with full Windows game support via Wine and Proton-GE/UMU, a working system tray on Hyprland, and a fully themeable cosmic aesthetic.

---

## Features

| | Feature |
|---|---|
| 🎮 | **Multi-store support** — Epic Games, GOG, Steam, Itch.io, Local |
| 📚 | **Unified library** — all your games in one place with cover art |
| 🪟 | **Windows games** — launch via Wine or Proton-GE/UMU with auto prefix setup |
| 🌊 | **Wine-Wayland** — native Wayland rendering for Wine and Proton games |
| ⬇️ | **UMU auto-download** — fetches `umu-run` automatically if not found |
| 🔧 | **Working system tray** on Hyprland and other Wayland compositors |
| 🎨 | **Many built-in themes** + community theme support |
| ⚡ | **Native performance** — no Electron, no web views |
| 📦 | **Per-game settings** — runner, prefix, MangoHud, GameMode, env vars and more |

---

## Store Support

| Store | Library | Launch |
|-------|---------|--------|
| Epic Games | ✅ | ⚠️ in progress |
| GOG | ✅ | ⚠️ in progress |
| Steam | ✅ | ⚠️ in progress |
| Itch.io | ✅ | ⚠️ in progress |
| Local | ✅ | ✅ |

> Native and Windows local games launch fully. Store-specific install/download buttons are in active development.

---

## Installation

### Arch / Artix

```bash
# Dependencies
sudo pacman -S rust gtk4 libayatana-appindicator

# Clone and build
git clone https://github.com/FemBoyGamerTechGuy/NovaDream
cd NovaDream
cargo build --release
sudo install -Dm755 target/release/NovaDream /usr/bin/NovaDream
```

### Fedora

```bash
sudo dnf install rust cargo gtk4-devel libayatana-appindicator-gtk3-devel

git clone https://github.com/FemBoyGamerTechGuy/NovaDream
cd NovaDream
cargo build --release
sudo install -Dm755 target/release/NovaDream /usr/bin/NovaDream
```

### .deb / .rpm packages

Pre-built packages can be built locally:

```bash
# Install packaging tools
cargo install cargo-deb cargo-generate-rpm

# Build both
chmod +x packaging/build-packages.sh
./packaging/build-packages.sh
```

Output: `target/debian/novadream_*.deb` and `target/generate-rpm/NovaDream-*.rpm`

---

## Windows Games

NovaDream supports launching Windows `.exe` games via Wine or Proton-GE/UMU.

- Wine prefixes are created automatically and named after the game title
- UMU is downloaded automatically if not found on your system
- Wine-Wayland is supported — enable it per-game or globally in Game Defaults
- MangoHud and GameMode can be toggled per-game or globally

Per-game settings are accessible by clicking the ⚙ icon on any game card.

---

## Configuration

Config is stored at `~/.config/NovaDream/config.json` and created automatically on first launch.

| Key | Default | Description |
|-----|---------|-------------|
| `theme` | `catppuccin-macchiato` | Active theme |
| `show_tray` | `true` | Show system tray icon |
| `close_to_tray` | `true` | Minimise to tray on close |
| `epic_library` | `~/.local/share/NovaDream/epic` | Epic install path |
| `gog_library` | `~/.local/share/NovaDream/gog` | GOG install path |
| `itch_library` | `~/.local/share/NovaDream/itch` | Itch.io install path |

Wine prefixes are stored at `~/.local/share/NovaDream/prefixes/<GameTitle>`.

---

## Theming

Themes live in `~/.local/share/NovaDream/themes/` as JSON files and are loaded automatically on launch.

Built-in themes include: Catppuccin (all flavours), Dracula, Tokyo Night, Tokyo Storm, Nord, Gruvbox, Gruvbox Light, Rosé Pine (all variants), Everforest, Kanagawa, Material Ocean, One Dark, Solarized Dark/Light, Monokai, Ayu Dark/Mirage/Light.

---

## Project Structure

```
NovaDream/
├── src/
│   ├── main.rs              # Entry point
│   ├── app.rs               # GTK4 application + CSS
│   ├── config.rs            # Config management
│   ├── game.rs              # Game data model
│   ├── local_library.rs     # Local game persistence + cover fetching
│   ├── proton.rs            # Proton runner detection
│   ├── umu.rs               # UMU download + resolution
│   ├── tray.rs              # System tray
│   ├── stores/              # Store integrations
│   │   ├── epic.rs
│   │   ├── gog.rs
│   │   ├── steam.rs
│   │   └── itch.rs
│   └── ui/
│       ├── library.rs       # Game library view + filters
│       ├── game_card.rs     # Grid + list cards, launch logic
│       ├── game_settings.rs # Per-game settings dialog
│       ├── game_defaults.rs # Global game defaults tab
│       ├── settings.rs      # App settings tab
│       ├── add_game.rs      # Add local game dialog
│       ├── store.rs         # Store browser
│       └── login.rs         # Store login dialogs
├── assets/
│   ├── icons/hicolor/       # App icons (16–256px)
│   ├── desktop/             # .desktop entry
│   └── *.metainfo.xml       # AppStream metadata
├── packaging/               # .deb and .rpm build scripts
├── CHANGELOG.md
├── Cargo.toml
├── LICENSE
└── README.md
```

---

## Part of Faded Dream

NovaDream is part of the [Faded Dream dotfiles](https://github.com/FemBoyGamerTechGuy/Faded-Dream-dotfiles) ecosystem.

---

## License

Copyright (C) 2026 FemBoyGamerTechGuy

This project is licensed under the **GNU General Public License v3.0**.

You are free to use, modify, and distribute this project, but any derivative work must also be open source under the same license.

See the [LICENSE](LICENSE) file for the full license text.
