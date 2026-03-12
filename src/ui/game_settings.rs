// NovaDream — per-game settings dialog
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, Entry, Label, Orientation, ScrolledWindow,
    Switch, TextView, Window, ApplicationWindow, StringList, DropDown,
    Separator,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::game::{Game, LaunchMode};
use crate::proton::Runner;
use crate::config::default_prefix_for;

/// Open the per-game settings dialog.
/// Returns Some(updated_game) if the user clicked Save, None if cancelled.
pub fn show_game_settings(
    parent:  &ApplicationWindow,
    game:    &Game,
    runners: &[Runner],
) -> Option<Game> {
    let result: Rc<RefCell<Option<Game>>> = Rc::new(RefCell::new(None));
    let done = Rc::new(RefCell::new(false));
    let game = game.clone();

    let dialog = Window::builder()
        .title(format!("Settings — {}", game.title))
        .transient_for(parent)
        .modal(true)
        .default_width(560)
        .default_height(620)
        .build();
    dialog.add_css_class("novadream");

    let scroll = ScrolledWindow::new();

    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_margin_top(24);
    root.set_margin_bottom(24);
    root.set_margin_start(24);
    root.set_margin_end(24);

    // ── Helper closures ───────────────────────────────────────────
    let make_section = |title: &str| -> GtkBox {
        let b = GtkBox::new(Orientation::Vertical, 8);
        b.set_margin_top(16);
        let lbl = Label::new(Some(title));
        lbl.add_css_class("game-settings-section");
        lbl.set_halign(gtk4::Align::Start);
        b.append(&lbl);
        let sep = Separator::new(Orientation::Horizontal);
        sep.set_margin_bottom(4);
        b.append(&sep);
        b
    };



    // ── Runner section (Windows games only) ──────────────────────
    let is_windows = game.launch_mode == LaunchMode::Windows;
    let runner_sec = make_section("Runner");

    let runner_names: Vec<String> = std::iter::once("(Use global default)".to_string())
        .chain(runners.iter().map(|r| r.label()))
        .collect();
    // Parallel vec of actual binary paths (index 0 = None for "use default")
    let runner_binaries: Vec<Option<String>> = std::iter::once(None)
        .chain(runners.iter().map(|r| Some(r.binary().to_string_lossy().to_string())))
        .collect();
    let runner_list = StringList::new(&runner_names.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let runner_drop = DropDown::new(Some(runner_list), gtk4::Expression::NONE);
    runner_drop.set_hexpand(true);
    // Select current runner if set — match on binary path
    if let Some(ref r) = game.runner {
        if let Some(idx) = runner_binaries.iter().position(|b| b.as_deref() == Some(r.as_str())) {
            runner_drop.set_selected(idx as u32);
        }
    }
    runner_sec.append(&make_row("Runner", &runner_drop));

    // Wine prefix
    let prefix_entry = Entry::builder()
        .placeholder_text("(Use global default prefix)")
        .hexpand(true)
        .build();
    // Use saved prefix, or fall back to the default per-game path
    let current_prefix = game.wine_prefix.clone()
        .unwrap_or_else(|| default_prefix_for(&game.title));
    prefix_entry.set_text(&current_prefix);
    let prefix_row = GtkBox::new(Orientation::Horizontal, 8);
    prefix_row.set_margin_top(4);
    {
        let lbl = Label::new(Some("Wine Prefix"));
        lbl.set_width_chars(18);
        lbl.set_xalign(0.0);
        lbl.add_css_class("game-settings-label");
        prefix_row.append(&lbl);
        prefix_row.append(&prefix_entry);

        // Browse button — opens at the NovaDream prefixes base directory
        let browse_btn = Button::with_label("Browse…");
        browse_btn.add_css_class("browse-btn");
        let pe = prefix_entry.clone();
        let par = parent.clone();
        browse_btn.connect_clicked(move |_| {
            let prefixes_base = dirs::data_local_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("~/.local/share"))
                .join("NovaDream")
                .join("prefixes");
            let _ = std::fs::create_dir_all(&prefixes_base);
            let fc = gtk4::FileDialog::builder()
                .title("Select Wine Prefix Folder")
                .initial_folder(&gtk4::gio::File::for_path(&prefixes_base))
                .build();
            let pe2 = pe.clone();
            fc.select_folder(Some(&par), gtk4::gio::Cancellable::NONE, move |res| {
                if let Ok(f) = res {
                    if let Some(path) = f.path() {
                        pe2.set_text(&path.to_string_lossy());
                    }
                }
            });
        });
        prefix_row.append(&browse_btn);
    }
    runner_sec.append(&prefix_row);

    if is_windows { root.append(&runner_sec); }

    // ── Launch section ────────────────────────────────────────────
    let launch_sec = make_section("Launch");

    let args_entry = Entry::builder()
        .placeholder_text("Extra launch arguments…")
        .hexpand(true)
        .build();
    if let Some(ref a) = game.launch_args {
        args_entry.set_text(a);
    }
    launch_sec.append(&make_row("Launch Args", &args_entry));

    let workdir_entry = Entry::builder()
        .placeholder_text("Default: executable directory")
        .hexpand(true)
        .build();
    if let Some(ref w) = game.work_dir {
        workdir_entry.set_text(w);
    }
    launch_sec.append(&make_row("Working Dir", &workdir_entry));

    let pre_entry = Entry::builder()
        .placeholder_text("Path to pre-launch script…")
        .hexpand(true)
        .build();
    if let Some(ref s) = game.pre_launch {
        pre_entry.set_text(s);
    }
    launch_sec.append(&make_row("Pre-launch Script", &pre_entry));

    let post_entry = Entry::builder()
        .placeholder_text("Path to post-exit script…")
        .hexpand(true)
        .build();
    if let Some(ref s) = game.post_exit {
        post_entry.set_text(s);
    }
    launch_sec.append(&make_row("Post-exit Script", &post_entry));
    root.append(&launch_sec);

    // ── Environment Variables ─────────────────────────────────────
    let env_sec = make_section("Environment Variables");
    let env_lbl = Label::new(Some("One KEY=VALUE per line"));
    env_lbl.add_css_class("game-settings-hint");
    env_lbl.set_halign(gtk4::Align::Start);
    env_sec.append(&env_lbl);

    let env_view = TextView::new();
    env_view.set_monospace(true);
    env_view.set_wrap_mode(gtk4::WrapMode::None);
    env_view.set_hexpand(true);
    env_view.set_size_request(-1, 80);
    if let Some(ref e) = game.env_vars {
        env_view.buffer().set_text(e);
    }
    let env_scroll = ScrolledWindow::new();
    env_scroll.add_css_class("env-scroll");
    env_scroll.set_child(Some(&env_view));
    env_scroll.set_hexpand(true);
    env_scroll.set_size_request(-1, 80);
    env_sec.append(&env_scroll);
    root.append(&env_sec);

    // ── Library section ───────────────────────────────────────────
    let lib_sec = make_section("Library");

    let fav_switch = Switch::new();
    fav_switch.set_active(game.favorite);
    fav_switch.set_valign(gtk4::Align::Center);
    lib_sec.append(&make_row("Favorite (pin to top)", &fav_switch));

    let hidden_switch = Switch::new();
    hidden_switch.set_active(game.hidden);
    hidden_switch.set_valign(gtk4::Align::Center);
    lib_sec.append(&make_row("Hide from Library", &hidden_switch));

    let mango_switch = Switch::new();
    mango_switch.set_active(game.use_mangohud);
    mango_switch.set_valign(gtk4::Align::Center);
    lib_sec.append(&make_row("MangoHud", &mango_switch));

    let gmode_switch = Switch::new();
    gmode_switch.set_active(game.use_gamemode);
    gmode_switch.set_valign(gtk4::Align::Center);
    lib_sec.append(&make_row("GameMode", &gmode_switch));

    let wayland_switch = Switch::new();
    wayland_switch.set_active(game.use_wine_wayland);
    wayland_switch.set_valign(gtk4::Align::Center);
    lib_sec.append(&make_row("Wine Wayland", &wayland_switch));

    root.append(&lib_sec);

    // ── Notes ─────────────────────────────────────────────────────
    let notes_sec = make_section("Notes");
    let notes_view = TextView::new();
    notes_view.set_wrap_mode(gtk4::WrapMode::Word);
    notes_view.set_hexpand(true);
    notes_view.set_size_request(-1, 72);
    if let Some(ref n) = game.notes {
        notes_view.buffer().set_text(n);
    }
    let notes_scroll = ScrolledWindow::new();
    notes_scroll.add_css_class("env-scroll");
    notes_scroll.set_child(Some(&notes_view));
    notes_scroll.set_hexpand(true);
    notes_scroll.set_size_request(-1, 72);
    notes_sec.append(&notes_scroll);
    root.append(&notes_sec);

    // ── Buttons ───────────────────────────────────────────────────
    let btn_row = GtkBox::new(Orientation::Horizontal, 8);
    btn_row.set_halign(gtk4::Align::End);
    btn_row.set_margin_top(24);

    let cancel_btn = Button::with_label("Cancel");
    let save_btn   = Button::with_label("Save");
    save_btn.add_css_class("suggested-action");
    btn_row.append(&cancel_btn);
    btn_row.append(&save_btn);
    root.append(&btn_row);

    scroll.set_child(Some(&root));
    dialog.set_child(Some(&scroll));

    // Cancel
    {
        let d = dialog.clone();
        let done = done.clone();
        cancel_btn.connect_clicked(move |_| {
            *done.borrow_mut() = true;
            d.close();
        });
    }

    // Save
    {
        let d = dialog.clone();
        let res = result.clone();
        let done = done.clone();
        let game = game.clone();
        let runner_binaries = runner_binaries.clone();
        save_btn.connect_clicked(move |_| {
            let selected = runner_drop.selected() as usize;
            let runner = runner_binaries.get(selected).cloned().flatten();

            let opt_str = |s: String| if s.is_empty() { None } else { Some(s) };

            let start = env_view.buffer().start_iter();
            let end   = env_view.buffer().end_iter();
            let env_text = env_view.buffer().text(&start, &end, false).to_string();

            let start = notes_view.buffer().start_iter();
            let end   = notes_view.buffer().end_iter();
            let notes_text = notes_view.buffer().text(&start, &end, false).to_string();

            let mut updated = game.clone();
            updated.runner      = runner;
            updated.wine_prefix = opt_str(prefix_entry.text().to_string());
            updated.launch_args = opt_str(args_entry.text().to_string());
            updated.work_dir    = opt_str(workdir_entry.text().to_string());
            updated.pre_launch  = opt_str(pre_entry.text().to_string());
            updated.post_exit   = opt_str(post_entry.text().to_string());
            updated.env_vars    = opt_str(env_text);
            updated.notes            = opt_str(notes_text);
            updated.favorite         = fav_switch.is_active();
            updated.hidden           = hidden_switch.is_active();
            updated.use_mangohud     = mango_switch.is_active();
            updated.use_gamemode     = gmode_switch.is_active();
            updated.use_wine_wayland = wayland_switch.is_active();

            *res.borrow_mut() = Some(updated);
            *done.borrow_mut() = true;
            d.close();
        });
    }

    {
        let done = done.clone();
        dialog.connect_close_request(move |_| {
            *done.borrow_mut() = true;
            glib::Propagation::Proceed
        });
    }

    dialog.present();
    let ctx = glib::MainContext::default();
    while !*done.borrow() {
        ctx.iteration(true);
    }

    let x = result.borrow().clone(); x
}

fn make_row(label: &str, widget: &impl IsA<gtk4::Widget>) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.set_margin_top(4);
    let lbl = Label::new(Some(label));
    lbl.set_width_chars(18);
    lbl.set_xalign(0.0);
    lbl.add_css_class("game-settings-label");
    row.append(&lbl);
    row.append(widget);
    row
}
