<div align="center">

# NovaDream

**A dreamy void-themed game launcher with multi-store support.**

[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blueviolet?style=flat-square)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![GTK4](https://img.shields.io/badge/UI-GTK4-4a86cf?style=flat-square&logo=gnome)](https://gtk.org/)
[![Part of Faded Dream](https://img.shields.io/badge/part%20of-Faded%20Dream-purple?style=flat-square)](https://github.com/FemBoyGamerTechGuy/Faded-Dream-dotfiles)

</div>

---

## Overview

NovaDream is a fast, native game launcher built with Rust and GTK4. It brings together your Epic Games, GOG, Steam, and Itch.io libraries in one place, with a working system tray on Hyprland and a fully themeable void/dreamy aesthetic.

---

## Features

| | Feature |
|---|---|
| 🎮 | **Multi-store support** — Epic Games, GOG, Steam, Itch.io |
| 📚 | **Unified library** — all your games in one place |
| 🖼️ | **Game artwork & covers** fetched automatically |
| ⬇️ | **Download & install** games from any supported store |
| 🚀 | **Launch games** directly from the launcher |
| 🔧 | **Working system tray** on Hyprland and other Wayland compositors |
| 🎨 | **Many built-in themes** + community theme support |
| ⚡ | **Native performance** — no Electron, no web views |

---

## Store Support

| Store | Library | Download | Install | Launch |
|-------|---------|----------|---------|--------|
| Epic Games | ✅ | ✅ | ✅ | ✅ |
| GOG | ✅ | ✅ | ✅ | ✅ |
| Steam | ✅ | ✅ | ✅ | ✅ |
| Itch.io | ✅ | ✅ | ✅ | ✅ |

Store integration is handled via their official CLI tools (`legendary`, `gogdl`, `steam`, `butler`).

---

## Installation

### Arch / Artix

```bash
# Install dependencies
sudo pacman -S rust gtk4 libadwaita libayatana-appindicator

# Install store CLI tools
sudo pacman -S steam
yay -S legendary gogdl butler  # or paru

# Clone and build
git clone https://github.com/FemBoyGamerTechGuy/NovaDream
cd NovaDream
cargo build --release
sudo install -Dm755 target/release/NovaDream /usr/bin/NovaDream
```

### Fedora

```bash
sudo dnf install rust cargo gtk4-devel libadwaita-devel libayatana-appindicator-devel

git clone https://github.com/FemBoyGamerTechGuy/NovaDream
cd NovaDream
cargo build --release
sudo install -Dm755 target/release/NovaDream /usr/bin/NovaDream
```

---

## Configuration

Config is stored at `~/.config/NovaDream/config.json` and is created automatically on first launch.

| Key | Default | Description |
|-----|---------|-------------|
| `theme` | `catppuccin-macchiato` | Active theme |
| `show_tray` | `true` | Show system tray icon |
| `close_to_tray` | `true` | Minimize to tray on close |
| `epic_library` | `~/.local/share/NovaDream/epic` | Epic install path |
| `gog_library` | `~/.local/share/NovaDream/gog` | GOG install path |
| `itch_library` | `~/.local/share/NovaDream/itch` | Itch.io install path |

---

## Theming

Themes live in `~/.local/share/NovaDream/themes/` as JSON files and are loaded automatically on launch.

Many built-in themes are included: Catppuccin (all flavours), Dracula, Tokyo Night, Nord, Gruvbox, Rosé Pine, Everforest, Kanagawa and more.

---

## Project Structure

```
NovaDream/
├── src/
│   ├── main.rs           # Entry point
│   ├── app.rs            # GTK4 application setup
│   ├── ui/               # UI components
│   │   ├── library.rs    # Game library view
│   │   ├── sidebar.rs    # Store sidebar
│   │   └── tray.rs       # System tray
│   ├── stores/           # Store integrations
│   │   ├── epic.rs       # Epic Games via legendary
│   │   ├── gog.rs        # GOG via gogdl
│   │   ├── steam.rs      # Steam
│   │   └── itch.rs       # Itch.io via butler
│   ├── theme.rs          # Theme engine
│   └── config.rs         # Config management
├── themes/               # Built-in theme JSON files
├── assets/               # Icons and UI assets
├── CHANGELOG.md          # Version history
├── CONTRIBUTING.md       # Contributor License Agreement
├── Cargo.toml            # Rust package manifest
├── LICENSE               # GPL-3.0-or-later
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
