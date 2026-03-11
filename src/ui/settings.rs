// NovaDream — Settings panel
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, DropDown, Entry, Label, ListBox, Orientation,
    ScrolledWindow, Separator, StringList, Switch,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::config::Config;
use crate::proton::Runner;

#[allow(dead_code)]
pub struct SettingsPanel {
    pub widget: ScrolledWindow,
    cfg:        Rc<RefCell<Config>>,
}

impl SettingsPanel {
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

        // ── Appearance ───────────────────────────────────────────────────────
        root.append(&section_title("Appearance"));

        let (theme_lbl_box, _) = setting_row("Theme", "Choose the colour scheme");
        root.append(&theme_lbl_box);

        // Grouped theme picker — scrollable ListBox with category headers
        let theme_scroll = ScrolledWindow::new();
        theme_scroll.set_height_request(220);
        theme_scroll.set_hexpand(true);
        theme_scroll.add_css_class("theme-scroll");

        let theme_list = ListBox::new();
        theme_list.set_selection_mode(gtk4::SelectionMode::None);
        theme_list.add_css_class("theme-list");

        let theme_groups: &[(&str, &[&str])] = &[
            ("System", &["system"]),
            ("Catppuccin", &["catppuccin-macchiato","catppuccin-mocha","catppuccin-latte","catppuccin-frappe"]),
            ("Popular Dark", &["dracula","tokyo-night","tokyo-night-storm","nord","one-dark","monokai","material-ocean","kanagawa","everforest"]),
            ("Rose Pine", &["rose-pine","rose-pine-moon","rose-pine-dawn"]),
            ("Gruvbox", &["gruvbox","gruvbox-light"]),
            ("Solarized", &["solarized-dark","solarized-light"]),
            ("Ayu", &["ayu-dark","ayu-mirage","ayu-light"]),
        ];

        let current = cfg.borrow().theme.clone();

        for (group_name, themes) in theme_groups {
            // Group header row
            let header = Label::new(Some(group_name));
            header.add_css_class("theme-group-header");
            header.set_halign(gtk4::Align::Start);
            header.set_margin_top(6);
            header.set_margin_bottom(2);
            header.set_margin_start(10);
            theme_list.append(&header);

            for theme_id in *themes {
                let row_box = GtkBox::new(Orientation::Horizontal, 8);
                row_box.set_margin_start(20);
                row_box.set_margin_top(3);
                row_box.set_margin_bottom(3);
                row_box.add_css_class("theme-row");

                // Colour swatch
                let swatch = gtk4::DrawingArea::new();
                swatch.set_size_request(16, 16);
                swatch.add_css_class("theme-swatch");
                let tid = theme_id.to_string();
                swatch.set_draw_func(move |_, cr, _, _| {
                    let color = swatch_color(&tid);
                    cr.set_source_rgb(color.0, color.1, color.2);
                    cr.arc(8.0, 8.0, 7.0, 0.0, std::f64::consts::TAU);
                    let _ = cr.fill();
                });

                let name_lbl = Label::new(Some(&theme_display_name(theme_id)));
                name_lbl.add_css_class("theme-name");
                name_lbl.set_halign(gtk4::Align::Start);
                name_lbl.set_hexpand(true);

                // Checkmark if active
                let check = Label::new(Some(if *theme_id == current.as_str() { "✓" } else { "" }));
                check.add_css_class("theme-check");
                check.set_margin_end(10);

                row_box.append(&swatch);
                row_box.append(&name_lbl);
                row_box.append(&check);

                let btn = Button::new();
                btn.set_child(Some(&row_box));
                btn.add_css_class("theme-btn");

                {
                    let c   = cfg.clone();
                    let tid = theme_id.to_string();
                    let tl  = theme_list.clone();
                    let chk = check.clone();
                    btn.connect_clicked(move |_| {
                        // Clear all checks
                        clear_theme_checks(&tl);
                        chk.set_label("✓");
                        c.borrow_mut().theme = tid.clone();
                        c.borrow().save();
                        crate::app::apply_theme(&tid);
                    });
                }

                theme_list.append(&btn);
            }
        }

        theme_scroll.set_child(Some(&theme_list));
        root.append(&theme_scroll);
        root.append(&Separator::new(Orientation::Horizontal));

        // ── System Tray ──────────────────────────────────────────────────────
        root.append(&section_title("System Tray"));

        let tray_switch = Switch::new();
        tray_switch.set_active(cfg.borrow().show_tray);
        tray_switch.set_valign(gtk4::Align::Center);
        {
            let c = cfg.clone();
            tray_switch.connect_active_notify(move |s| {
                c.borrow_mut().show_tray = s.is_active();
                c.borrow().save();
            });
        }
        let (tray_lbl, _) = setting_row("Show tray icon", "Show NovaDream in the system tray");
        root.append(&hrow(tray_lbl, tray_switch.upcast()));

        let close_switch = Switch::new();
        close_switch.set_active(cfg.borrow().close_to_tray);
        close_switch.set_valign(gtk4::Align::Center);
        {
            let c = cfg.clone();
            close_switch.connect_active_notify(move |s| {
                c.borrow_mut().close_to_tray = s.is_active();
                c.borrow().save();
            });
        }
        let (close_lbl, _) = setting_row("Close to tray", "Minimise to tray instead of quitting");
        root.append(&hrow(close_lbl, close_switch.upcast()));
        root.append(&Separator::new(Orientation::Horizontal));

        // ── Wine / Proton ────────────────────────────────────────────────────
        root.append(&section_title("Wine / Proton"));

        if runners.is_empty() {
            let no_runners = Label::new(Some(
                "No runners found.\nAdd Proton or Wine builds to ~/.local/share/NovaDream/proton/"
            ));
            no_runners.set_wrap(true);
            no_runners.add_css_class("settings-hint");
            root.append(&no_runners);
        } else {
            let runner_names: Vec<String> = runners.iter().map(|r| r.label()).collect();
            let runner_strs: Vec<&str>    = runner_names.iter().map(|s| s.as_str()).collect();
            let runner_list = StringList::new(&runner_strs);
            let runner_drop = DropDown::new(Some(runner_list), gtk4::Expression::NONE);

            let current_runner = cfg.borrow().default_runner.clone();
            if let Some(idx) = runner_names.iter().position(|r| *r == current_runner) {
                runner_drop.set_selected(idx as u32);
            }
            runner_drop.add_css_class("settings-drop");

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

            let (runner_lbl, _) = setting_row("Default runner", "Used for Windows games unless overridden per game");
            root.append(&hrow(runner_lbl, runner_drop.upcast()));
        }

        root.append(&Separator::new(Orientation::Horizontal));

        // ── Library Paths ────────────────────────────────────────────────────
        root.append(&section_title("Library Paths"));

        for (label, hint, getter, setter) in library_path_rows() {
            let current_val = getter(&cfg.borrow());
            let entry = Entry::new();
            entry.set_text(&current_val);
            entry.set_hexpand(true);
            entry.add_css_class("settings-entry");
            {
                let c = cfg.clone();
                let s = setter;
                entry.connect_changed(move |e| {
                    s(&mut c.borrow_mut(), e.text().to_string());
                    c.borrow().save();
                });
            }
            let (lbl_box, _) = setting_row(label, hint);
            let col = GtkBox::new(Orientation::Vertical, 4);
            col.append(&lbl_box);
            col.append(&entry);
            col.set_margin_bottom(8);
            root.append(&col);
        }

        root.append(&Separator::new(Orientation::Horizontal));

        // ── Accounts ─────────────────────────────────────────────────────────
        root.append(&section_title("Store Accounts"));

        let accounts_hint = Label::new(Some(
            "Log in to stores from the sidebar. Account status is shown there."
        ));
        accounts_hint.set_wrap(true);
        accounts_hint.add_css_class("settings-hint");
        root.append(&accounts_hint);

        scroll.set_child(Some(&root));
        Self { widget: scroll, cfg }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

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

type PathGetter = fn(&Config) -> String;
type PathSetter = fn(&mut Config, String);

fn library_path_rows() -> Vec<(&'static str, &'static str, PathGetter, PathSetter)> {
    vec![
        ("Epic Games library",  "Where Epic games are installed",  |c: &Config| c.epic_library.clone(),  |c: &mut Config, v| c.epic_library = v),
        ("GOG library",         "Where GOG games are installed",   |c: &Config| c.gog_library.clone(),   |c: &mut Config, v| c.gog_library = v),
        ("Steam library",       "Where Steam games are installed",  |c: &Config| c.steam_library.clone(), |c: &mut Config, v| c.steam_library = v),
        ("Itch.io library",     "Where Itch games are installed",  |c: &Config| c.itch_library.clone(),  |c: &mut Config, v| c.itch_library = v),
        ("Local library",       "Where manually added games live", |c: &Config| c.local_library.clone(), |c: &mut Config, v| c.local_library = v),
    ]
}

fn theme_display_name(id: &str) -> String {
    match id {
        "system"                => "System Theme".into(),
        "catppuccin-macchiato"  => "Macchiato".into(),
        "catppuccin-mocha"      => "Mocha".into(),
        "catppuccin-latte"      => "Latte (Light)".into(),
        "catppuccin-frappe"     => "Frappé".into(),
        "tokyo-night"           => "Tokyo Night".into(),
        "tokyo-night-storm"     => "Tokyo Storm".into(),
        "rose-pine"             => "Rosé Pine".into(),
        "rose-pine-moon"        => "Rosé Pine Moon".into(),
        "rose-pine-dawn"        => "Rosé Pine Dawn (Light)".into(),
        "gruvbox-light"         => "Gruvbox Light".into(),
        "material-ocean"        => "Material Ocean".into(),
        "one-dark"              => "One Dark".into(),
        "solarized-dark"        => "Solarized Dark".into(),
        "solarized-light"       => "Solarized Light".into(),
        "ayu-dark"              => "Ayu Dark".into(),
        "ayu-mirage"            => "Ayu Mirage".into(),
        "ayu-light"             => "Ayu Light".into(),
        other => {
            let mut s = other.replace('-', " ");
            s[..1].make_ascii_uppercase(); s
        }
    }
}

/// Returns (r,g,b) 0–1 accent colour for the swatch
fn swatch_color(id: &str) -> (f64, f64, f64) {
    match id {
        "system"               => (0.5,  0.5,  0.5),
        "catppuccin-macchiato" => (0.78, 0.63, 0.96),
        "catppuccin-mocha"     => (0.80, 0.65, 0.97),
        "catppuccin-latte"     => (0.53, 0.22, 0.94),
        "catppuccin-frappe"    => (0.79, 0.62, 0.90),
        "dracula"              => (0.74, 0.58, 0.98),
        "tokyo-night" | "tokyo-night-storm" => (0.48, 0.64, 0.97),
        "nord"                 => (0.53, 0.75, 0.82),
        "gruvbox" | "gruvbox-light" => (0.85, 0.53, 0.60),
        "rose-pine" | "rose-pine-moon" => (0.77, 0.66, 0.91),
        "rose-pine-dawn"       => (0.43, 0.42, 0.64),
        "everforest"           => (0.65, 0.75, 0.50),
        "kanagawa"             => (0.58, 0.50, 0.72),
        "material-ocean"       => (0.51, 0.67, 1.00),
        "one-dark"             => (0.78, 0.47, 0.87),
        "solarized-dark" | "solarized-light" => (0.42, 0.44, 0.77),
        "monokai"              => (0.68, 0.51, 1.00),
        "ayu-dark" | "ayu-mirage" => (0.82, 0.65, 1.00),
        "ayu-light"            => (0.64, 0.48, 0.80),
        _                      => (0.6,  0.6,  0.6),
    }
}

fn clear_theme_checks(list: &ListBox) {
    let mut row = list.first_child();
    while let Some(r) = row {
        // GTK wraps appended widgets in a ListBoxRow — get its child
        if let Some(inner) = r.first_child() {
            if let Some(btn) = inner.downcast_ref::<Button>() {
                if let Some(box_child) = btn.child().and_then(|w| w.downcast::<GtkBox>().ok()) {
                    if let Some(check) = box_child.last_child().and_then(|w| w.downcast::<Label>().ok()) {
                        check.set_label("");
                    }
                }
            }
        }
        row = r.next_sibling();
    }
}
