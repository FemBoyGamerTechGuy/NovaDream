// NovaDream — Game Defaults panel
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, DropDown, Entry, Label, Orientation,
    ScrolledWindow, Separator, StringList, Switch, TextView, WrapMode,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::config::Config;
use crate::proton::Runner;

#[allow(dead_code)]
pub struct GameDefaultsPanel {
    pub widget: ScrolledWindow,
    cfg:        Rc<RefCell<Config>>,
}

impl GameDefaultsPanel {
    pub fn new(cfg: Rc<RefCell<Config>>, runners: Vec<Runner>) -> Self {
        let scroll = ScrolledWindow::new();
        scroll.set_hexpand(true);
        scroll.set_vexpand(true);

        let root = GtkBox::new(Orientation::Vertical, 0);
        root.set_margin_top(32);
        root.set_margin_bottom(32);
        root.set_margin_start(48);
        root.set_margin_end(48);
        root.set_halign(gtk4::Align::Center);
        root.set_width_request(600);

        // ── Runner ───────────────────────────────────────────────────────────
        root.append(&section_title("Runner"));

        if runners.is_empty() {
            let hint = Label::new(Some(
                "No runners found.\nAdd Proton or Wine builds to ~/.local/share/NovaDream/proton/"
            ));
            hint.set_wrap(true);
            hint.add_css_class("settings-hint");
            root.append(&hint);
        } else {
            let runner_names: Vec<String> = runners.iter().map(|r| r.label()).collect();
            let runner_strs: Vec<&str>    = runner_names.iter().map(|s| s.as_str()).collect();
            let runner_list = StringList::new(&runner_strs);
            let runner_drop = DropDown::new(Some(runner_list), gtk4::Expression::NONE);
            runner_drop.add_css_class("settings-drop");

            let current_runner = cfg.borrow().default_runner.clone();
            if let Some(idx) = runner_names.iter().position(|r| *r == current_runner) {
                runner_drop.set_selected(idx as u32);
            }
            {
                let c = cfg.clone();
                let names = runner_names.clone();
                runner_drop.connect_selected_notify(move |d: &DropDown| {
                    if let Some(name) = names.get(d.selected() as usize) {
                        c.borrow_mut().default_runner = name.clone();
                        c.borrow().save();
                    }
                });
            }
            let (lbl, _) = setting_row("Default runner", "Used for Windows games unless overridden per game");
            root.append(&hrow(lbl, runner_drop.upcast()));
        }

        // Default wine prefix base
        {
            let entry = Entry::new();
            entry.set_text(&cfg.borrow().default_wine_prefix);
            entry.set_hexpand(true);
            entry.add_css_class("settings-entry");
            {
                let c = cfg.clone();
                entry.connect_changed(move |e| {
                    c.borrow_mut().default_wine_prefix = e.text().to_string();
                    c.borrow().save();
                });
            }
            let (lbl, _) = setting_row("Default prefix base", "Base directory for Wine prefixes — each game gets its own subfolder");
            let col = GtkBox::new(Orientation::Vertical, 4);
            col.append(&lbl);
            col.append(&entry);
            col.set_margin_bottom(8);
            root.append(&col);
        }

        root.append(&Separator::new(Orientation::Horizontal));

        // ── Overlays & Tools ─────────────────────────────────────────────────
        root.append(&section_title("Overlays & Tools"));

        // MangoHud
        {
            let sw = Switch::new();
            sw.set_active(cfg.borrow().use_mangohud);
            sw.set_valign(gtk4::Align::Center);
            let c = cfg.clone();
            sw.connect_active_notify(move |s| {
                c.borrow_mut().use_mangohud = s.is_active();
                c.borrow().save();
            });
            let (lbl, _) = setting_row("MangoHud", "Show FPS/frame-time overlay (per-game setting overrides this)");
            root.append(&hrow(lbl, sw.upcast()));
        }

        // GameMode
        {
            let sw = Switch::new();
            sw.set_active(cfg.borrow().use_gamemode);
            sw.set_valign(gtk4::Align::Center);
            let c = cfg.clone();
            sw.connect_active_notify(move |s| {
                c.borrow_mut().use_gamemode = s.is_active();
                c.borrow().save();
            });
            let (lbl, _) = setting_row("GameMode", "Run games with gamemoderun for CPU optimisations");
            root.append(&hrow(lbl, sw.upcast()));
        }

        // Wine Wayland
        {
            let sw = Switch::new();
            sw.set_active(cfg.borrow().use_wine_wayland);
            sw.set_valign(gtk4::Align::Center);
            let c = cfg.clone();
            sw.connect_active_notify(move |s| {
                c.borrow_mut().use_wine_wayland = s.is_active();
                c.borrow().save();
            });
            let (lbl, _) = setting_row("Wine Wayland", "Use the wine-wayland driver for native Wayland rendering (no XWayland)");
            root.append(&hrow(lbl, sw.upcast()));
        }

        root.append(&Separator::new(Orientation::Horizontal));

        // ── Behaviour ────────────────────────────────────────────────────────
        root.append(&section_title("Behaviour"));

        // Minimize on launch
        {
            let sw = Switch::new();
            sw.set_active(cfg.borrow().minimize_on_launch);
            sw.set_valign(gtk4::Align::Center);
            let c = cfg.clone();
            sw.connect_active_notify(move |s| {
                c.borrow_mut().minimize_on_launch = s.is_active();
                c.borrow().save();
            });
            let (lbl, _) = setting_row("Minimise on launch", "Hide the launcher window when a game starts");
            root.append(&hrow(lbl, sw.upcast()));
        }

        // Track playtime
        {
            let sw = Switch::new();
            sw.set_active(cfg.borrow().track_playtime);
            sw.set_valign(gtk4::Align::Center);
            let c = cfg.clone();
            sw.connect_active_notify(move |s| {
                c.borrow_mut().track_playtime = s.is_active();
                c.borrow().save();
            });
            let (lbl, _) = setting_row("Track playtime", "Record how long you play each game");
            root.append(&hrow(lbl, sw.upcast()));
        }

        // Show playtime on card
        {
            let sw = Switch::new();
            sw.set_active(cfg.borrow().show_playtime_on_card);
            sw.set_valign(gtk4::Align::Center);
            let c = cfg.clone();
            sw.connect_active_notify(move |s| {
                c.borrow_mut().show_playtime_on_card = s.is_active();
                c.borrow().save();
            });
            let (lbl, _) = setting_row("Show playtime on card", "Display playtime badge on game cards in the library");
            root.append(&hrow(lbl, sw.upcast()));
        }

        root.append(&Separator::new(Orientation::Horizontal));

        // ── Launch Flags ─────────────────────────────────────────────────────
        root.append(&section_title("Launch Flags"));

        {
            let entry = Entry::new();
            entry.set_text(&cfg.borrow().launch_flags);
            entry.set_hexpand(true);
            entry.add_css_class("settings-entry");
            entry.set_placeholder_text(Some("e.g. --windowed --no-intro"));
            {
                let c = cfg.clone();
                entry.connect_changed(move |e| {
                    c.borrow_mut().launch_flags = e.text().to_string();
                    c.borrow().save();
                });
            }
            let (lbl, _) = setting_row("Extra launch flags", "Appended to every game's command line (per-game overrides this)");
            let col = GtkBox::new(Orientation::Vertical, 4);
            col.append(&lbl);
            col.append(&entry);
            col.set_margin_bottom(8);
            root.append(&col);
        }

        root.append(&Separator::new(Orientation::Horizontal));

        // ── Environment Variables ────────────────────────────────────────────
        root.append(&section_title("Environment Variables"));

        {
            let (hint_lbl, _) = setting_row(
                "Global env vars",
                "One KEY=VALUE per line — applied to every game launch"
            );
            root.append(&hint_lbl);

            let tv = TextView::new();
            tv.set_hexpand(true);
            tv.set_height_request(100);
            tv.set_wrap_mode(WrapMode::None);
            tv.set_monospace(true);
            tv.add_css_class("settings-entry");
            tv.buffer().set_text(&cfg.borrow().env_vars);

            {
                let c = cfg.clone();
                tv.buffer().connect_changed(move |buf| {
                    let text = buf.text(&buf.start_iter(), &buf.end_iter(), false).to_string();
                    c.borrow_mut().env_vars = text;
                    c.borrow().save();
                });
            }

            let tv_wrap = GtkBox::new(Orientation::Vertical, 0);
            tv_wrap.append(&tv);
            tv_wrap.set_margin_bottom(8);
            root.append(&tv_wrap);
        }

        scroll.set_child(Some(&root));
        Self { widget: scroll, cfg }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn section_title(text: &str) -> Label {
    let lbl = Label::new(Some(text));
    lbl.add_css_class("settings-section");
    lbl.set_halign(gtk4::Align::Start);
    lbl.set_margin_top(16);
    lbl.set_margin_bottom(8);
    lbl
}

fn setting_row(title: &str, hint: &str) -> (GtkBox, Label) {
    let col = GtkBox::new(Orientation::Vertical, 2);
    col.set_hexpand(true);
    let t = Label::new(Some(title));
    t.add_css_class("setting-title");
    t.set_halign(gtk4::Align::Start);
    let h = Label::new(Some(hint));
    h.add_css_class("setting-hint");
    h.set_halign(gtk4::Align::Start);
    col.append(&t);
    col.append(&h);
    (col, h)
}

fn hrow(left: GtkBox, right: gtk4::Widget) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 16);
    row.set_margin_top(8);
    row.set_margin_bottom(8);
    row.append(&left);
    right.set_halign(gtk4::Align::End);
    right.set_valign(gtk4::Align::Center);
    row.append(&right);
    row
}
