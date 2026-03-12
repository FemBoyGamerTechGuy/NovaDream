// NovaDream — a cosmic void-themed game launcher for Linux
// Copyright (C) 2026  FemBoyGamerTechGuy <https://github.com/FemBoyGamerTechGuy>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Arc, Mutex};
use std::time::Instant;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Button, Overlay, Orientation, Image};
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::gdk::Texture;
use glib;
use crate::game::Game;
use libc;

mod spawn_game_mod {
    use crate::game::{Game, LaunchMode};
    use std::process::{Child, Command};
    use std::path::Path;
    use std::os::unix::process::CommandExt;
    use dirs;

    /// Check if a binary exists on PATH
    #[allow(dead_code)]
    pub fn which_bin(name: &str) -> bool {
        Command::new("which").arg(name)
            .output().map(|o| o.status.success()).unwrap_or(false)
    }

    pub fn spawn_game(game: &Game) -> Option<Child> {
        let exe = game.exe_path.as_deref()?;

        let workdir = game.work_dir.as_deref()
            .map(Path::new)
            .unwrap_or_else(|| Path::new(exe).parent().unwrap_or(Path::new("/")));

        // Parse KEY=VALUE env vars
        let env_pairs: Vec<(String, String)> = game.env_vars.as_deref()
            .unwrap_or("")
            .lines()
            .filter_map(|l| {
                let mut parts = l.splitn(2, '=');
                Some((parts.next()?.trim().to_string(), parts.next()?.trim().to_string()))
            })
            .collect();

        // Extra CLI args
        let extra_args: Vec<String> = game.launch_args.as_deref()
            .unwrap_or("")
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        match game.launch_mode {
            LaunchMode::Linux => {
                let mut cmd = build_wrapped(exe, &extra_args, game);
                cmd.current_dir(workdir);
                for (k, v) in &env_pairs { cmd.env(k, v); }
                cmd.process_group(0);
                cmd.spawn().ok()
            }
            LaunchMode::Windows => {
                let runner = game.runner.as_deref().unwrap_or("wine");

                // Resolve prefix path
                let default_prefix = crate::config::default_prefix_for(&game.title);
                let prefix = game.wine_prefix.as_deref()
                    .filter(|s| !s.is_empty())
                    .unwrap_or(&default_prefix);

                // Classify runner by binary name
                let runner_bin = Path::new(runner)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| runner.to_string());

                // Any Proton build (proton, GE-Proton, etc) should go through umu-run —
                // same approach as Heroic. umu-run handles STEAM_COMPAT_* internally.
                // Priority: system umu-run → bundled umu-run in NovaDream data dir → call proton directly
                let is_proton_runner = runner_bin == "proton" || runner_bin == "umu-run";
                let is_wine = !is_proton_runner;

                // Resolve umu-run path: system first, then NovaDream's bundled copy
                let umu_path = if which_bin("umu-run") {
                    Some("umu-run".to_string())
                } else {
                    let bundled = dirs::data_local_dir()
                        .unwrap_or_default()
                        .join("NovaDream/runtimes/umu/umu-run");
                    if bundled.exists() { Some(bundled.to_string_lossy().to_string()) } else { None }
                };

                let use_umu = is_proton_runner && umu_path.is_some();

                // Ensure prefix dir exists
                let _ = std::fs::create_dir_all(prefix);

                // Wine only: init prefix with wineboot + disable virtual desktop
                if is_wine {
                    let prefix_empty = std::fs::read_dir(prefix)
                        .map(|mut d| d.next().is_none()).unwrap_or(true);
                    let rb = Path::new(runner);
                    let wine_bin = rb.parent()
                        .map(|p| p.join("wine")).filter(|p| p.exists())
                        .or_else(|| rb.parent().and_then(|p| p.parent())
                            .map(|p| p.join("bin/wine")).filter(|p| p.exists()))
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| runner.to_string());
                    if prefix_empty {
                        let wineboot = rb.parent()
                            .map(|p| p.join("wineboot")).filter(|p| p.exists())
                            .or_else(|| rb.parent().and_then(|p| p.parent())
                                .map(|p| p.join("bin/wineboot")).filter(|p| p.exists()))
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| "wineboot".to_string());
                        let _ = Command::new(&wineboot)
                            .arg("--init")
                            .env("WINEPREFIX", prefix)
                            .status();
                    }
                    let _ = Command::new(&wine_bin)
                        .args(["reg", "delete", "HKCU\\Software\\Wine\\Explorer",
                            "/v", "Desktop", "/f"])
                        .env("WINEPREFIX", prefix)
                        .output();
                }

                // The game's install directory — Proton uses this to locate game files
                let game_install_dir = Path::new(exe).parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Build the launch command.
                // Proton runners always go through umu-run (system or bundled) — same as Heroic.
                // Only fall back to calling proton directly if umu-run isn't available anywhere.
                let mut cmd = if use_umu {
                    let umu = umu_path.as_deref().unwrap();
                    let proton_dir = Path::new(runner).parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| runner.to_string());
                    let mut c = build_wrapped(umu, &extra_args_with_exe(exe, &extra_args), game);
                    c.env("WINEPREFIX", prefix);
                    c.env("GAMEID", "0");
                    c.env("PROTONPATH", &proton_dir);
                    c.env("STEAM_COMPAT_DATA_PATH", prefix);
                    c.env("STEAM_COMPAT_INSTALL_PATH", &game_install_dir);
                    c.env("SteamAppId", "0");
                    c.env("SteamGameId", format!("novadream-{}", &game.id));
                    c.env("WINEESYNC", "1");
                    c.env("WINEFSYNC", "1");
                    c.env("WINEDLLOVERRIDES", "winemenubuilder.exe=d");
                    c.env("LD_PRELOAD", "");
                    c
                } else if is_proton_runner {
                    // No umu-run — call proton directly as last resort
                    let mut proton_args = vec!["waitforexitandrun".to_string(), exe.to_string()];
                    proton_args.extend(extra_args.iter().cloned());
                    let steam_home = dirs::home_dir().unwrap_or_default().join(".steam/steam");
                    let _ = std::fs::create_dir_all(&steam_home);
                    let mut c = build_wrapped(runner, &proton_args, game);
                    c.env("STEAM_COMPAT_DATA_PATH", prefix);
                    c.env("STEAM_COMPAT_INSTALL_PATH", &game_install_dir);
                    c.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", steam_home.to_string_lossy().as_ref());
                    c.env("SteamAppId", "0");
                    c.env("SteamGameId", format!("novadream-{}", &game.id));
                    c.env("WINEPREFIX", format!("{}/pfx", prefix));
                    c.env("WINEESYNC", "1");
                    c.env("WINEFSYNC", "1");
                    c.env("WINEDLLOVERRIDES", "winemenubuilder.exe=d");
                    c.env("LD_PRELOAD", "");
                    c
                } else {
                    // Plain Wine
                    let mut c = build_wrapped(runner, &extra_args_with_exe(exe, &extra_args), game);
                    c.env("WINEPREFIX", prefix);
                    c.env("WINEESYNC", "1");
                    c.env("WINEFSYNC", "1");
                    c.env("WINEDLLOVERRIDES", "winemenubuilder.exe=d");
                    c
                };

                cmd.current_dir(workdir);

                // Apply wine-wayland settings before spawning.
                // - Plain Wine: unset DISPLAY (forces waylanddrv over x11drv/XWayland)
                //   and write Graphics=wayland into the prefix registry (prefix-wide enforcement).
                // - Proton/UMU: set PROTON_WAYLAND=1 + unset DISPLAY so Proton's own
                //   wayland support activates (same as Heroic's implementation).
                let wayland_display = std::env::var("WAYLAND_DISPLAY").ok();
                if game.use_wine_wayland {
                    cmd.env_remove("DISPLAY");
                    if let Some(ref wd) = wayland_display {
                        cmd.env("WAYLAND_DISPLAY", wd);
                    }
                    if use_umu || is_proton_runner {
                        cmd.env("PROTON_WAYLAND", "1");
                    }
                }

                // Write the real screen resolution into the prefix registry every launch.
                // Games that query desktop size at startup get the real answer, not 640x480.
                let (w, h) = get_screen_resolution();
                let res_str = format!("{}x{}", w, h);

                if is_wine {
                    let rb = Path::new(runner);
                    let wine_bin = rb.parent()
                        .map(|p| p.join("wine")).filter(|p| p.exists())
                        .or_else(|| rb.parent().and_then(|p| p.parent())
                            .map(|p| p.join("bin/wine")).filter(|p| p.exists()))
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| runner.to_string());

                    // Helper: run a wine reg command, inheriting wayland env state
                    let wine_bin_ref = wine_bin.clone();
                    let prefix_ref = prefix.to_string();
                    let wayland_display2 = wayland_display.clone();
                    let use_wayland = game.use_wine_wayland;
                    let wine_reg = move |args: &[&str]| {
                        let mut c = Command::new(&wine_bin_ref);
                        c.args(args).env("WINEPREFIX", &prefix_ref);
                        if use_wayland {
                            c.env_remove("DISPLAY");
                            if let Some(ref wd) = wayland_display2 {
                                c.env("WAYLAND_DISPLAY", wd);
                            }
                        }
                        let _ = c.output();
                    };

                    if game.use_wine_wayland {
                        // Force waylanddrv prefix-wide — the definitive way per Arch Wiki / Wine docs.
                        // Without this key Wine still prefers x11drv if DISPLAY ever reappears.
                        wine_reg(&["reg", "add",
                            "HKCU\\Software\\Wine\\Drivers",
                            "/v", "Graphics",
                            "/t", "REG_SZ", "/d", "wayland", "/f"]);
                        // Wayland driver resolution key
                        wine_reg(&["reg", "add",
                            "HKCU\\Software\\Wine\\Wayland Driver",
                            "/v", "ScreenResolution",
                            "/t", "REG_SZ", "/d", &res_str, "/f"]);
                        // Keep X11 key in sync so toggling back to X11 works immediately
                        wine_reg(&["reg", "add",
                            "HKCU\\Software\\Wine\\X11 Driver",
                            "/v", "ScreenResolution",
                            "/t", "REG_SZ", "/d", &res_str, "/f"]);
                        // Virtual desktop conflicts with wine-wayland — ensure it's off
                        wine_reg(&["reg", "delete",
                            "HKCU\\Software\\Wine\\Explorer",
                            "/v", "Desktop", "/f"]);
                    } else {
                        // Switching back to X11: remove the wayland-only Graphics override
                        // so Wine reverts to its default x11,wayland priority order.
                        wine_reg(&["reg", "delete",
                            "HKCU\\Software\\Wine\\Drivers",
                            "/v", "Graphics", "/f"]);
                        wine_reg(&["reg", "add",
                            "HKCU\\Software\\Wine\\X11 Driver",
                            "/v", "ScreenResolution",
                            "/t", "REG_SZ", "/d", &res_str, "/f"]);
                        wine_reg(&["reg", "delete",
                            "HKCU\\Software\\Wine\\Explorer",
                            "/v", "Desktop", "/f"]);
                    }
                } else if use_umu {
                    // Proton/UMU — just set a useful compat hint
                    cmd.env("PROTON_FORCE_LARGE_ADDRESS_AWARE", "1");
                }

                for (k, v) in &env_pairs { cmd.env(k, v); }
                cmd.process_group(0);
                cmd.spawn().ok()
            }
            LaunchMode::Browser => {
                Command::new("xdg-open").arg(exe).spawn().ok()
            }
        }
    }

    fn extra_args_with_exe(exe: &str, extra: &[String]) -> Vec<String> {
        std::iter::once(exe.to_string()).chain(extra.iter().cloned()).collect()
    }

    /// Get the primary display resolution. Tries xrandr, wlr-randr, then /sys framebuffer.
    /// Falls back to 1920x1080 if nothing works.
    fn get_screen_resolution() -> (u32, u32) {
        // Try xrandr (X11 and XWayland)
        if let Ok(out) = Command::new("xrandr").output() {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                // Look for "   1920x1080+0+0  ..." (connected primary output)
                if line.contains(" connected") || line.starts_with("   ") {
                    if let Some(res) = line.split_whitespace().find(|s| s.contains('x') && s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)) {
                        let res = res.split('+').next().unwrap_or(res);
                        let parts: Vec<&str> = res.splitn(2, 'x').collect();
                        if parts.len() == 2 {
                            if let (Ok(w), Ok(h)) = (parts[0].parse::<u32>(), parts[1].trim_end_matches(|c: char| !c.is_ascii_digit()).parse::<u32>()) {
                                if w > 320 && h > 240 {
                                    return (w, h);
                                }
                            }
                        }
                    }
                }
            }
        }
        // Try wlr-randr (wlroots Wayland compositors)
        if let Ok(out) = Command::new("wlr-randr").output() {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if line.trim_start().starts_with("\"") { continue; }
                if let Some(pos) = line.find(|c: char| c.is_ascii_digit()) {
                    let part = &line[pos..];
                    if part.contains('x') {
                        let nums: Vec<&str> = part.splitn(2, 'x').collect();
                        if nums.len() == 2 {
                            let w = nums[0].parse::<u32>().ok();
                            let h = nums[1].split_whitespace().next()
                                .and_then(|s| s.parse::<u32>().ok());
                            if let (Some(w), Some(h)) = (w, h) {
                                if w > 320 && h > 240 {
                                    return (w, h);
                                }
                            }
                        }
                    }
                }
            }
        }
        // Default
        (1920, 1080)
    }

    /// Build a Command with optional gamemoderun/mangohud wrapping
    fn build_wrapped(program: &str, args: &[String], game: &Game) -> Command {
        // Determine actual argv[0] and arg list after applying wrappers
        // Order: mangohud gamemoderun <program> [args]
        let mut full: Vec<String> = Vec::new();

        if game.use_gamemode { full.push("gamemoderun".into()); }
        full.push(program.to_string());
        full.extend_from_slice(args);

        if game.use_mangohud {
            let mut c = Command::new("mangohud");
            c.args(&full);
            c
        } else {
            let mut c = Command::new(&full[0]);
            c.args(&full[1..]);
            c
        }
    }

}
use spawn_game_mod::spawn_game;

/// Returns true once the given pid has at least one child process in /proc.
/// Used to detect when umu-run has finished setting up and spawned the actual game.
fn has_grandchild(pid: u32) -> bool {
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if let Ok(p) = name.to_string_lossy().parse::<u32>() {
                let status_path = format!("/proc/{}/status", p);
                if let Ok(status) = std::fs::read_to_string(&status_path) {
                    let ppid = status.lines()
                        .find(|l| l.starts_with("PPid:"))
                        .and_then(|l| l.split_whitespace().nth(1))
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(0);
                    if ppid == pid && p != pid {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Recursively kill an entire process tree by walking /proc.
/// Wine, Proton and UMU all fork children that escape the process group,
/// so we must walk every descendant and SIGKILL them bottom-up.
fn kill_tree(root_pid: u32) {
    // Collect all pids and their parent pids from /proc
    let mut children: std::collections::HashMap<u32, Vec<u32>> = std::collections::HashMap::new();

    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if let Ok(pid) = name_str.parse::<u32>() {
                let status_path = format!("/proc/{}/status", pid);
                if let Ok(status) = std::fs::read_to_string(&status_path) {
                    let ppid = status.lines()
                        .find(|l| l.starts_with("PPid:"))
                        .and_then(|l| l.split_whitespace().nth(1))
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(0);
                    children.entry(ppid).or_default().push(pid);
                }
            }
        }
    }

    // Walk tree depth-first and collect pids to kill
    let mut to_kill = vec![];
    let mut stack = vec![root_pid];
    while let Some(pid) = stack.pop() {
        to_kill.push(pid);
        if let Some(kids) = children.get(&pid) {
            stack.extend_from_slice(kids);
        }
    }

    // Kill bottom-up (leaves first) so parents can't re-spawn
    for pid in to_kill.iter().rev() {
        unsafe { libc::kill(*pid as i32, libc::SIGKILL); }
    }
}

fn wire_play_button(
    btn:        &Button,
    game:       &Game,
    on_stopped: impl Fn(String, u64) + 'static,
) {
    let game_id  = game.id.clone();
    let game_arc = Arc::new(Mutex::new(game.clone()));
    // (child_arc, start_time, pid) — pid stored separately so Stop never has to lock the mutex
    let running: std::rc::Rc<std::cell::RefCell<Option<(Arc<Mutex<std::process::Child>>, Instant, u32)>>>
        = std::rc::Rc::new(std::cell::RefCell::new(None));

    let on_stopped = std::rc::Rc::new(on_stopped);
    let b = btn.clone();

    btn.connect_clicked(move |_| {
        // Check if already running — scope the borrow so it's dropped before do_launch
        let is_running = running.borrow().is_some();
        if is_running {
            if let Some((arc, _, pid)) = running.borrow_mut().take() {
                std::thread::spawn(move || {
                    // Kill by pid directly — no mutex needed, no blocking
                    kill_tree(pid);
                    unsafe { libc::kill(-(pid as i32), libc::SIGKILL); }
                    // Lock only to reap the zombie after it's already dead
                    if let Ok(mut c) = arc.lock() { let _ = c.wait(); }
                });
            }
            b.set_label("▶  Play");
            b.remove_css_class("btn-stop");
            b.add_css_class("btn-play");
        } else {
            let game = game_arc.lock().unwrap().clone();

            // For Windows games using a Proton runner, ensure umu-run is available.
            // If not, download it first (async) then launch.
            let needs_umu = matches!(game.launch_mode, crate::game::LaunchMode::Windows)
                && game.runner.as_deref()
                    .map(|r| std::path::Path::new(r).file_name()
                        .map(|n| n == "proton").unwrap_or(false))
                    .unwrap_or(false)
                && crate::umu::resolve_umu().is_none();

            if needs_umu {
                b.set_label("⬇  Downloading UMU…");
                b.remove_css_class("btn-play");
                b.add_css_class("btn-stop");
                b.set_sensitive(false);

                let b2 = b.clone();
                let running2 = running.clone();
                let on_stopped2 = on_stopped.clone();
                let game_id2 = game_id.clone();
                let game2 = game.clone();

                // Channel to send progress updates from the download thread to the GTK main loop
                let (prog_tx, prog_rx) = std::sync::mpsc::channel::<u8>();
                let b3 = b2.clone();
                glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
                    match prog_rx.try_recv() {
                        Ok(pct) => {
                            b3.set_label(&format!("⬇  UMU {}%", pct));
                            glib::ControlFlow::Continue
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                        Err(_) => glib::ControlFlow::Break,
                    }
                });

                // Result channel: Ok(path) or Err(msg)
                let (res_tx, res_rx) = std::sync::mpsc::channel::<Result<String, String>>();
                std::thread::spawn(move || {
                    let result = crate::umu::ensure_umu_blocking(move |dl, total| {
                        if let Some(t) = total {
                            let _ = prog_tx.send((dl * 100 / t) as u8);
                        }
                    });
                    let _ = res_tx.send(result.map_err(|e| e.to_string()));
                });

                // Poll result channel on main loop
                glib::timeout_add_local(std::time::Duration::from_millis(300), move || {
                    match res_rx.try_recv() {
                        Ok(Ok(_)) => {
                            b2.set_label("⏳  Launching…");
                            do_launch(&b2, &game2, &running2, &on_stopped2, &game_id2);
                            glib::ControlFlow::Break
                        }
                        Ok(Err(e)) => {
                            eprintln!("Failed to download umu-run: {e}");
                            b2.set_label("▶  Play");
                            b2.remove_css_class("btn-stop");
                            b2.add_css_class("btn-play");
                            b2.set_sensitive(true);
                            glib::ControlFlow::Break
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                        Err(_) => glib::ControlFlow::Break,
                    }
                });
            } else {
                b.set_sensitive(false);
                b.remove_css_class("btn-play");
                b.add_css_class("btn-stop");
                do_launch(&b, &game, &running, &on_stopped, &game_id);
            }
        }
    });
}

/// Shared launch logic — called after UMU is confirmed available.
fn do_launch<F: Fn(String, u64) + 'static>(
    b:          &Button,
    game:       &Game,
    running:    &std::rc::Rc<std::cell::RefCell<Option<(Arc<Mutex<std::process::Child>>, Instant, u32)>>>,
    on_stopped: &std::rc::Rc<F>,
    game_id:    &str,
) {
    if let Some(child) = spawn_game(game) {
        let pid       = child.id();
        let child_arc = Arc::new(Mutex::new(child));
        let started   = Instant::now();
        *running.borrow_mut() = Some((child_arc.clone(), started, pid));

        b.set_label("⏳  Launching…");
        b.set_sensitive(false);

        let b2          = b.clone();
        let running_ref = running.clone();
        let on_stopped2 = on_stopped.clone();
        let gid2        = game_id.to_string();

        // Channel for game-exited signal
        let (tx_exit, rx_exit) = std::sync::mpsc::channel::<u64>();
        let arc2 = child_arc.clone();
        std::thread::spawn(move || {
            let _ = { arc2.lock().ok().and_then(|mut c| c.wait().ok()) };
            let _ = tx_exit.send(started.elapsed().as_secs());
        });

        // Channel for "umu is done, game is actually running" signal.
        // We watch /proc for a grandchild of pid — once one appears,
        // umu has finished setting up and the game process is alive.
        let (tx_ready, rx_ready) = std::sync::mpsc::channel::<()>();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(300));
                if has_grandchild(pid) {
                    let _ = tx_ready.send(());
                    break;
                }
                // Also bail if the process is already gone
                let alive = std::path::Path::new(&format!("/proc/{}", pid)).exists();
                if !alive { break; }
            }
        });

        // Poll for ready signal → flip Launching → Stop
        let b3 = b.clone();
        let mut ready = false;
        glib::timeout_add_local(std::time::Duration::from_millis(300), move || {
            if !ready {
                if rx_ready.try_recv().is_ok() {
                    ready = true;
                    b3.set_label("■  Stop");
                    b3.set_sensitive(true);
                }
            }
            // Keep this timer alive until exit is detected by the other poller
            glib::ControlFlow::Continue
        });

        glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
            match rx_exit.try_recv() {
                Ok(secs) => {
                    running_ref.borrow_mut().take();
                    on_stopped2(gid2.clone(), secs);
                    b2.set_label("▶  Play");
                    b2.remove_css_class("btn-stop");
                    b2.add_css_class("btn-play");
                    b2.set_sensitive(true);
                    glib::ControlFlow::Break
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(_) => glib::ControlFlow::Break,
            }
        });
    } else {
        // Failed to launch
        b.set_label("▶  Play");
        b.remove_css_class("btn-stop");
        b.add_css_class("btn-play");
        b.set_sensitive(true);
    }
}

/// Build a fixed-size cover widget — always exactly w×h, clipped
fn build_cover(path: Option<&String>, w: i32, h: i32, placeholder_size: i32) -> GtkBox {
    let frame = GtkBox::new(Orientation::Vertical, 0);
    frame.set_width_request(w);
    frame.set_height_request(h);
    frame.set_vexpand(false);
    frame.set_hexpand(false);
    frame.set_valign(gtk4::Align::Center);
    frame.set_halign(gtk4::Align::Start);
    frame.set_overflow(gtk4::Overflow::Hidden);

    if let Some(p) = path {
        let img = if let Ok(pb) = Pixbuf::from_file_at_scale(p, w, h, false) {
            let texture = Texture::for_pixbuf(&pb);
            let picture = gtk4::Picture::for_paintable(&texture);
            picture.set_can_shrink(false);
            picture.set_width_request(w);
            picture.set_height_request(h);
            picture.set_vexpand(false);
            picture.set_hexpand(false);
            picture.set_valign(gtk4::Align::Fill);
            picture.set_halign(gtk4::Align::Fill);
            picture.upcast::<gtk4::Widget>()
        } else {
            let i = Image::from_icon_name("applications-games");
            i.set_pixel_size(placeholder_size);
            i.upcast::<gtk4::Widget>()
        };
        img.add_css_class("cover-image");
        frame.append(&img);
    } else {
        frame.add_css_class("cover-placeholder");
        let icon = Image::from_icon_name("applications-games");
        icon.set_pixel_size(placeholder_size);
        icon.set_valign(gtk4::Align::Center);
        icon.set_vexpand(true);
        frame.append(&icon);
    }
    frame
}

/// Build a grid card — fixed 180×(220+60) total
pub fn build_grid_card(
    game:        &Game,
    on_remove:   impl Fn(String) + 'static,
    on_stopped:  impl Fn(String, u64) + 'static,
    on_settings: impl Fn(String) + 'static,
) -> GtkBox {
    let card = GtkBox::new(Orientation::Vertical, 0);
    card.set_width_request(180);
    card.set_height_request(280);
    card.set_vexpand(false);
    card.set_hexpand(false);
    card.set_valign(gtk4::Align::Start);
    card.set_halign(gtk4::Align::Start);
    card.set_overflow(gtk4::Overflow::Hidden);
    card.add_css_class("game-card");

    let cover_frame = build_cover(game.cover_path.as_ref(), 180, 220, 64);
    cover_frame.set_valign(gtk4::Align::Start);

    let overlay = Overlay::new();
    overlay.set_child(Some(&cover_frame));
    overlay.set_width_request(180);
    overlay.set_height_request(220);
    overlay.set_vexpand(false);
    overlay.set_hexpand(false);
    overlay.set_valign(gtk4::Align::Start);
    overlay.set_overflow(gtk4::Overflow::Hidden);

    // Info bar overlaid on cover
    let info = GtkBox::new(Orientation::Vertical, 4);
    info.add_css_class("card-info");
    info.set_valign(gtk4::Align::End);

    let badge_row = GtkBox::new(Orientation::Horizontal, 4);
    badge_row.set_hexpand(true);
    let store_badge = Label::new(Some(game.store.label()));
    store_badge.add_css_class("store-badge");
    store_badge.set_halign(gtk4::Align::Start);
    store_badge.set_hexpand(true);
    badge_row.append(&store_badge);
    let gear_btn = Button::with_label("⚙");
    gear_btn.add_css_class("card-remove-btn");
    let gid_s = game.id.clone();
    gear_btn.connect_clicked(move |_| on_settings(gid_s.clone()));
    badge_row.append(&gear_btn);
    let remove_btn = Button::with_label("×");
    remove_btn.add_css_class("card-remove-btn");
    let gid = game.id.clone();
    remove_btn.connect_clicked(move |_| on_remove(gid.clone()));
    badge_row.append(&remove_btn);
    info.append(&badge_row);

    let title = Label::new(Some(&game.title));
    title.add_css_class("card-title");
    title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    title.set_max_width_chars(18);
    title.set_halign(gtk4::Align::Start);
    info.append(&title);

    let meta = GtkBox::new(Orientation::Horizontal, 8);
    let playtime_lbl = Label::new(Some(&game.play_time_str()));
    playtime_lbl.add_css_class("card-meta");
    let last_lbl = Label::new(Some(&game.last_played_str()));
    last_lbl.add_css_class("card-meta");
    meta.append(&playtime_lbl);
    meta.append(&last_lbl);
    info.append(&meta);

    overlay.add_overlay(&info);
    card.append(&overlay);

    if game.installed {
        let play_btn = Button::with_label("▶  Play");
        play_btn.add_css_class("btn-play");
        let game_id = game.id.clone();
        wire_play_button(&play_btn, game, move |id, secs| on_stopped(id, secs));
        let _ = game_id;
        card.append(&play_btn);
    } else {
        let install_btn = Button::with_label("⬇  Install");
        install_btn.add_css_class("btn-install");
        card.append(&install_btn);
    }



    card
}

/// Build a list row — compact, 40x40 thumbnail
pub fn build_list_row(
    game:        &Game,
    on_remove:   impl Fn(String) + 'static,
    on_stopped:  impl Fn(String, u64) + 'static,
    on_settings: impl Fn(String) + 'static,
) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 10);
    row.add_css_class("list-row");
    row.set_vexpand(false);
    row.set_hexpand(true);
    row.set_margin_start(8);
    row.set_margin_end(8);
    row.set_margin_top(4);
    row.set_margin_bottom(4);

    // Use Image with pixbuf — pixel_size is a hard cap unlike Picture
    let thumb = if let Some(path) = &game.cover_path {
        if let Ok(pb) = gtk4::gdk_pixbuf::Pixbuf::from_file_at_scale(path, 40, 40, false) {
#[allow(deprecated)]
            let img = Image::from_pixbuf(Some(&pb));
            img.set_pixel_size(40);
            img
        } else {
            let img = Image::from_icon_name("applications-games");
            img.set_pixel_size(40);
            img
        }
    } else {
        let img = Image::from_icon_name("applications-games");
        img.set_pixel_size(40);
        img
    };
    thumb.set_valign(gtk4::Align::Center);
    thumb.set_vexpand(false);
    row.append(&thumb);

    let info = GtkBox::new(Orientation::Vertical, 4);
    info.set_hexpand(true);
    info.set_valign(gtk4::Align::Center);

    let title = Label::new(Some(&game.title));
    title.add_css_class("list-title");
    title.set_halign(gtk4::Align::Start);
    title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    info.append(&title);

    let meta = GtkBox::new(Orientation::Horizontal, 8);
    let store_badge = Label::new(Some(game.store.label()));
    store_badge.add_css_class("store-badge");
    let playtime_lbl = Label::new(Some(&game.play_time_str()));
    playtime_lbl.add_css_class("card-meta");
    let last_lbl = Label::new(Some(&game.last_played_str()));
    last_lbl.add_css_class("card-meta");
    meta.append(&store_badge);
    meta.append(&playtime_lbl);
    meta.append(&last_lbl);
    info.append(&meta);

    row.append(&info);

    let btn_box = GtkBox::new(Orientation::Horizontal, 6);
    btn_box.set_valign(gtk4::Align::Center);

    if game.installed {
        let play_btn = Button::with_label("▶  Play");
        play_btn.add_css_class("btn-play-small");
        wire_play_button(&play_btn, game, move |id, secs| on_stopped(id, secs));
        btn_box.append(&play_btn);
    } else {
        let install_btn = Button::with_label("⬇  Install");
        install_btn.add_css_class("btn-install-small");
        btn_box.append(&install_btn);
    }

    let gear_btn = Button::with_label("⚙");
    gear_btn.add_css_class("card-remove-btn");
    let gid_s = game.id.clone();
    gear_btn.connect_clicked(move |_| on_settings(gid_s.clone()));
    btn_box.append(&gear_btn);

    let remove_btn = Button::with_label("×");
    remove_btn.add_css_class("card-remove-btn");
    let gid = game.id.clone();
    remove_btn.connect_clicked(move |_| on_remove(gid.clone()));
    btn_box.append(&remove_btn);

    row.append(&btn_box);



    row
}


