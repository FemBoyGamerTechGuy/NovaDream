// NovaDream — game card widgets (grid + list)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Arc, Mutex};
use std::time::Instant;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Button, Overlay, Orientation, Image};
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::gdk::Texture;
use glib;
use crate::game::Game;

mod spawn_game_mod {
    use crate::game::{Game, LaunchMode};
    use std::process::{Child, Command};
    use std::path::Path;

    pub fn spawn_game(game: &Game) -> Option<Child> {
        let exe = game.exe_path.as_deref()?;
        let workdir = Path::new(exe).parent().unwrap_or(Path::new("/"));
        match game.launch_mode {
            LaunchMode::Linux => Command::new(exe)
                .current_dir(workdir)
                .spawn().ok(),
            LaunchMode::Windows => {
                let runner = game.runner.as_deref().unwrap_or("wine");
                Command::new(runner)
                    .arg(exe)
                    .current_dir(workdir)
                    .spawn().ok()
            }
            LaunchMode::Browser => {
                Command::new("xdg-open").arg(exe).spawn().ok()
            }
        }
    }
}
use spawn_game_mod::spawn_game;

fn wire_play_button(
    btn:        &Button,
    game:       &Game,
    on_stopped: impl Fn(String, u64) + 'static,
) {
    let game_id  = game.id.clone();
    let game_arc = Arc::new(Mutex::new(game.clone()));
    let running: std::rc::Rc<std::cell::RefCell<Option<(Arc<Mutex<std::process::Child>>, Instant)>>>
        = std::rc::Rc::new(std::cell::RefCell::new(None));

    let on_stopped = std::rc::Rc::new(on_stopped);
    let b = btn.clone();

    btn.connect_clicked(move |_| {
        let mut state = running.borrow_mut();
        if state.is_some() {
            // Stop
            if let Some((arc, _)) = state.take() {
                if let Ok(mut c) = arc.lock() { let _ = c.kill(); }
            }
            b.set_label("▶  Play");
            b.remove_css_class("btn-stop");
            b.add_css_class("btn-play");
        } else {
            let game = game_arc.lock().unwrap().clone();
            if let Some(child) = spawn_game(&game) {
                let child_arc = Arc::new(Mutex::new(child));
                let started   = Instant::now();
                *state = Some((child_arc.clone(), started));

                b.set_label("■  Stop");
                b.remove_css_class("btn-play");
                b.add_css_class("btn-stop");

                let b2           = b.clone();
                let running_ref  = running.clone();
                let on_stopped2  = on_stopped.clone();
                let gid2         = game_id.clone();

                let (tx, rx) = std::sync::mpsc::channel::<u64>();
                let arc2     = child_arc.clone();
                std::thread::spawn(move || {
                    if let Ok(mut c) = arc2.lock() { let _ = c.wait(); }
                    let _ = tx.send(started.elapsed().as_secs());
                });

                glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                    match rx.try_recv() {
                        Ok(secs) => {
                            running_ref.borrow_mut().take();
                            on_stopped2(gid2.clone(), secs);
                            b2.set_label("▶  Play");
                            b2.remove_css_class("btn-stop");
                            b2.add_css_class("btn-play");
                            glib::ControlFlow::Break
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                        Err(_) => glib::ControlFlow::Break,
                    }
                });
            }
        }
    });
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
    game:       &Game,
    on_remove:  impl Fn(String) + 'static,
    on_stopped: impl Fn(String, u64) + 'static,
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

/// Build a list row — thumbnail 48×64, fixed height 72
pub fn build_list_row(
    game:       &Game,
    on_remove:  impl Fn(String) + 'static,
    on_stopped: impl Fn(String, u64) + 'static,
) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("list-row");
    row.set_height_request(64);
    row.set_vexpand(false);
    row.set_overflow(gtk4::Overflow::Hidden);
    row.set_margin_start(8);
    row.set_margin_end(8);

    let thumb = build_cover(game.cover_path.as_ref(), 40, 56, 24);
    thumb.set_margin_top(4);
    thumb.set_margin_bottom(4);
    thumb.set_valign(gtk4::Align::Center);
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

    let remove_btn = Button::with_label("×");
    remove_btn.add_css_class("card-remove-btn");
    let gid = game.id.clone();
    remove_btn.connect_clicked(move |_| on_remove(gid.clone()));
    btn_box.append(&remove_btn);

    row.append(&btn_box);
    row
}


