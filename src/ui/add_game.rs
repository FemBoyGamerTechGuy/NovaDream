// NovaDream — Add Game dialog
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, DropDown, Entry, Label, Orientation,
    StringList, Window, ApplicationWindow, FileDialog, FileFilter,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::game::{Game, LaunchMode, Store};
use crate::proton::Runner;

pub fn show_add_game_dialog(
    parent: &ApplicationWindow,
    runners: &[Runner],
) -> Option<Game> {
    let result: Rc<RefCell<Option<Game>>> = Rc::new(RefCell::new(None));
    let done = Rc::new(RefCell::new(false));

    let dialog = Window::builder()
        .title("Add Game")
        .transient_for(parent)
        .modal(true)
        .default_width(480)
        .build();
    dialog.add_css_class("add-game-dialog");
    dialog.add_css_class("novadream");

    let vbox = GtkBox::new(Orientation::Vertical, 14);
    vbox.set_margin_top(24);
    vbox.set_margin_bottom(24);
    vbox.set_margin_start(24);
    vbox.set_margin_end(24);

    // ── Title ──────────────────────────────────────────────────────────────
    let title_lbl = Label::new(Some("Game Title"));
    title_lbl.set_halign(gtk4::Align::Start);
    title_lbl.add_css_class("field-label");
    vbox.append(&title_lbl);

    let title_entry = Entry::builder()
        .placeholder_text("e.g. Friday Night Funkin'")
        .build();
    vbox.append(&title_entry);

    // ── Launch mode ────────────────────────────────────────────────────────
    let mode_lbl = Label::new(Some("Run as"));
    mode_lbl.set_halign(gtk4::Align::Start);
    mode_lbl.add_css_class("field-label");
    vbox.append(&mode_lbl);

    let modes = StringList::new(&["Linux", "Windows (Wine/Proton)", "Web Browser"]);
    let mode_drop = DropDown::new(Some(modes), gtk4::Expression::NONE);
    mode_drop.set_selected(0);
    vbox.append(&mode_drop);

    // ── Runner selector (shown only for Windows mode) ──────────────────────
    let runner_lbl = Label::new(Some("Wine / Proton Runner"));
    runner_lbl.set_halign(gtk4::Align::Start);
    runner_lbl.add_css_class("field-label");
    runner_lbl.set_visible(false);
    vbox.append(&runner_lbl);

    // Build owned runner name strings for StringList
    let runner_name_owned: Vec<String> = runners.iter().map(|r| r.label()).collect();
    let runner_strs: Vec<&str> = runner_name_owned.iter().map(|s| s.as_str()).collect();

    let runner_list = StringList::new(if runner_strs.is_empty() {
        &["No runners found — add one to NovaDream/proton/"]
    } else {
        &runner_strs
    });
    let runner_drop = DropDown::new(Some(runner_list), gtk4::Expression::NONE);
    runner_drop.set_visible(false);
    vbox.append(&runner_drop);

    // Show/hide runner selector based on mode
    {
        let rl = runner_lbl.clone();
        let rd = runner_drop.clone();
        mode_drop.connect_selected_notify(move |d| {
            let show = d.selected() == 1; // Windows mode
            rl.set_visible(show);
            rd.set_visible(show);
        });
    }

    // ── Executable / URL ───────────────────────────────────────────────────
    let exe_lbl = Label::new(Some("Executable Path"));
    exe_lbl.set_halign(gtk4::Align::Start);
    exe_lbl.add_css_class("field-label");
    vbox.append(&exe_lbl);

    let exe_row = GtkBox::new(Orientation::Horizontal, 8);
    let exe_entry = Entry::builder()
        .placeholder_text("/path/to/game or https://...")
        .hexpand(true)
        .build();
    exe_row.append(&exe_entry);

    let browse_btn = Button::with_label("Browse…");
    browse_btn.add_css_class("browse-btn");
    exe_row.append(&browse_btn);
    vbox.append(&exe_row);

    // Browse button — open file picker
    {
        let entry = exe_entry.clone();
        let par = parent.clone();
        let mode_d = mode_drop.clone();
        browse_btn.connect_clicked(move |_| {
            if mode_d.selected() == 2 { return; } // no browse for browser mode
            let filter = FileFilter::new();
            filter.add_mime_type("application/x-executable");
            filter.add_pattern("*.exe");
            filter.add_pattern("*.sh");
            filter.add_pattern("*.AppImage");
            filter.set_name(Some("Executables"));

            let dialog = FileDialog::builder()
                .title("Select Executable")
                .default_filter(&filter)
                .initial_folder(&gtk4::gio::File::for_path(
                    dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"))
                ))
                .build();
            let e = entry.clone();
            dialog.open(Some(&par), gtk4::gio::Cancellable::NONE, move |res| {
                if let Ok(file) = res {
                    if let Some(path) = file.path() {
                        e.set_text(&path.to_string_lossy());
                    }
                }
            });
        });
    }

    // Update exe label based on mode
    {
        let lbl = exe_lbl.clone();
        let bb  = browse_btn.clone();
        mode_drop.connect_selected_notify(move |d| {
            match d.selected() {
                2 => { lbl.set_text("URL"); bb.set_sensitive(false); }
                _ => { lbl.set_text("Executable Path"); bb.set_sensitive(true); }
            }
        });
    }

    // ── Buttons row ─────────────────────────────────────────────────────────
    let btn_row = GtkBox::new(Orientation::Horizontal, 8);
    btn_row.set_halign(gtk4::Align::End);
    btn_row.set_margin_top(8);

    let cancel_btn = Button::with_label("Cancel");
    cancel_btn.add_css_class("flat");
    let add_btn = Button::with_label("Add Game");
    add_btn.add_css_class("suggested-action");
    btn_row.append(&cancel_btn);
    btn_row.append(&add_btn);
    vbox.append(&btn_row);

    dialog.set_child(Some(&vbox));

    // Cancel
    {
        let d    = dialog.clone();
        let done = done.clone();
        cancel_btn.connect_clicked(move |_| {
            *done.borrow_mut() = true;
            d.close();
        });
    }

    // Add
    {
        let d       = dialog.clone();
        let res     = result.clone();
        let done    = done.clone();
        let t_entry = title_entry.clone();
        let e_entry = exe_entry.clone();
        let m_drop  = mode_drop.clone();
        let r_drop  = runner_drop.clone();
        let runners = runners.to_vec();

        add_btn.connect_clicked(move |_| {
            let title = t_entry.text().to_string();
            let exe   = e_entry.text().to_string();
            if title.is_empty() || exe.is_empty() { return; }

            let launch_mode = match m_drop.selected() {
                1 => LaunchMode::Windows,
                2 => LaunchMode::Browser,
                _ => LaunchMode::Linux,
            };

            let runner = if launch_mode == LaunchMode::Windows && !runners.is_empty() {
                runners.get(r_drop.selected() as usize)
                    .map(|r| r.binary().to_string_lossy().to_string())
            } else {
                None
            };

            let game = Game {
                id:           uuid_simple(),
                title,
                store:        Store::Local,
                cover_url:    None,
                cover_path:   None,
                install_path: Some(exe.clone()),
                exe_path:     Some(exe),
                launch_mode,
                runner,
                installed:    true,
                play_time:    0,
                last_played:  None,
                ..Default::default()
            };

            *res.borrow_mut() = Some(game);
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

/// Generate a simple unique id
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("local-{}", t)
}
