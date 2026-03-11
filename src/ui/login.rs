// NovaDream — login dialogs
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Label, Button, Entry, Orientation, Window,
    ApplicationWindow,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Show an OAuth login dialog for Epic/GOG.
/// Returns the redirect URL or auth code when the user confirms.
pub fn show_oauth_dialog(parent: &ApplicationWindow, store_name: &str, auth_url: &str) -> Option<String> {
    let result: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let done = Rc::new(RefCell::new(false));

    let dialog = Window::builder()
        .title(format!("Log in to {}", store_name))
        .transient_for(parent)
        .modal(true)
        .default_width(520)
        .default_height(260)
        .build();

    let vbox = GtkBox::new(Orientation::Vertical, 12);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);

    let step1 = Label::new(Some(&format!(
        "1. Open this URL in your browser and log in to {}:",
        store_name
    )));
    step1.set_wrap(true);
    step1.set_halign(gtk4::Align::Start);
    vbox.append(&step1);

    let url_entry = Entry::new();
    url_entry.set_text(auth_url);
    url_entry.set_editable(false);
    vbox.append(&url_entry);

    let copy_btn = Button::with_label("Copy URL");
    let url_for_copy = auth_url.to_string();
    copy_btn.connect_clicked(move |btn| {
        if let Some(display) = gtk4::gdk::Display::default() {
            display.clipboard().set_text(&url_for_copy);
        }
        btn.set_label("✓ Copied!");
    });
    vbox.append(&copy_btn);

    let step2 = Label::new(Some(
        "2. After logging in, paste the redirect URL or auth code below:",
    ));
    step2.set_wrap(true);
    step2.set_halign(gtk4::Align::Start);
    vbox.append(&step2);

    let callback_entry = Entry::builder()
        .placeholder_text("Paste redirect URL or auth code here...")
        .build();
    vbox.append(&callback_entry);

    // Buttons row
    let btn_row = GtkBox::new(Orientation::Horizontal, 8);
    btn_row.set_halign(gtk4::Align::End);

    let cancel_btn = Button::with_label("Cancel");
    let ok_btn     = Button::with_label("Log In");
    ok_btn.add_css_class("suggested-action");
    btn_row.append(&cancel_btn);
    btn_row.append(&ok_btn);
    vbox.append(&btn_row);

    dialog.set_child(Some(&vbox));

    // Cancel
    {
        let d = dialog.clone();
        let done = done.clone();
        cancel_btn.connect_clicked(move |_| {
            *done.borrow_mut() = true;
            d.close();
        });
    }

    // OK
    {
        let d = dialog.clone();
        let res = result.clone();
        let done = done.clone();
        let entry = callback_entry.clone();
        ok_btn.connect_clicked(move |_| {
            let text = entry.text().to_string();
            if !text.is_empty() {
                *res.borrow_mut() = Some(text);
            }
            *done.borrow_mut() = true;
            d.close();
        });
    }

    // Run a local main loop until the dialog is closed
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

pub fn show_apikey_dialog(parent: &ApplicationWindow) -> Option<String> {
    let result: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let done = Rc::new(RefCell::new(false));

    let dialog = Window::builder()
        .title("Log in to Itch.io")
        .transient_for(parent)
        .modal(true)
        .default_width(480)
        .build();

    let vbox = GtkBox::new(Orientation::Vertical, 12);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);

    let msg = Label::new(Some(
        "Generate an API key at itch.io/user/settings/api-keys and paste it below:"
    ));
    msg.set_wrap(true);
    msg.set_halign(gtk4::Align::Start);
    vbox.append(&msg);

    let key_entry = Entry::builder()
        .placeholder_text("API key...")
        .visibility(false)
        .build();
    vbox.append(&key_entry);

    let btn_row = GtkBox::new(Orientation::Horizontal, 8);
    btn_row.set_halign(gtk4::Align::End);
    let cancel_btn = Button::with_label("Cancel");
    let ok_btn     = Button::with_label("Save");
    ok_btn.add_css_class("suggested-action");
    btn_row.append(&cancel_btn);
    btn_row.append(&ok_btn);
    vbox.append(&btn_row);

    dialog.set_child(Some(&vbox));

    {
        let d = dialog.clone();
        let done = done.clone();
        cancel_btn.connect_clicked(move |_| {
            *done.borrow_mut() = true;
            d.close();
        });
    }
    {
        let d = dialog.clone();
        let res = result.clone();
        let done = done.clone();
        let entry = key_entry.clone();
        ok_btn.connect_clicked(move |_| {
            let k = entry.text().to_string();
            if !k.is_empty() { *res.borrow_mut() = Some(k); }
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
