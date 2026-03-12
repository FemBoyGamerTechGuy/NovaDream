// NovaDream — Store tab (open stores in browser)
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation, ScrolledWindow};

struct StoreEntry {
    name:        &'static str,
    emoji:       &'static str,
    description: &'static str,
    url:         &'static str,
    css:         &'static str,
}

const STORES: &[StoreEntry] = &[
    StoreEntry {
        name:        "Steam",
        emoji:       "🎮",
        description: "The largest PC gaming platform. Browse thousands of games, sales, and your wishlist.",
        url:         "https://store.steampowered.com",
        css:         "store-btn-steam",
    },
    StoreEntry {
        name:        "Epic Games",
        emoji:       "⚡",
        description: "Free games every week, exclusives, and the Unreal Engine ecosystem.",
        url:         "https://store.epicgames.com",
        css:         "store-btn-epic",
    },
    StoreEntry {
        name:        "GOG",
        emoji:       "🔓",
        description: "DRM-free games. Buy once, own forever. Great classic game library.",
        url:         "https://www.gog.com",
        css:         "store-btn-gog",
    },
    StoreEntry {
        name:        "Itch.io",
        emoji:       "🎲",
        description: "Indie games, game jams, and pay-what-you-want titles from independent developers.",
        url:         "https://itch.io",
        css:         "store-btn-itch",
    },
];

pub struct StorePanel {
    pub widget: ScrolledWindow,
}

impl StorePanel {
    pub fn new() -> Self {
        let scroll = ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_hexpand(true);

        let root = GtkBox::new(Orientation::Vertical, 24);
        root.set_margin_start(32);
        root.set_margin_end(32);
        root.set_margin_top(32);
        root.set_margin_bottom(32);

        let heading = Label::new(Some("Browse Stores"));
        heading.add_css_class("store-heading");
        heading.set_halign(gtk4::Align::Start);
        root.append(&heading);

        let subheading = Label::new(Some("Open your favourite store in the browser to browse and purchase games."));
        subheading.add_css_class("store-subheading");
        subheading.set_halign(gtk4::Align::Start);
        root.append(&subheading);

        let grid = GtkBox::new(Orientation::Horizontal, 16);
        grid.set_homogeneous(false);
        grid.set_halign(gtk4::Align::Start);

        for store in STORES {
            let card = GtkBox::new(Orientation::Vertical, 12);
            card.add_css_class("store-card");
            card.set_width_request(220);

            let emoji_lbl = Label::new(Some(store.emoji));
            emoji_lbl.add_css_class("store-emoji");
            card.append(&emoji_lbl);

            let name_lbl = Label::new(Some(store.name));
            name_lbl.add_css_class("store-name");
            card.append(&name_lbl);

            let desc_lbl = Label::new(Some(store.description));
            desc_lbl.add_css_class("store-desc");
            desc_lbl.set_wrap(true);
            desc_lbl.set_max_width_chars(26);
            desc_lbl.set_halign(gtk4::Align::Center);
            card.append(&desc_lbl);

            let spacer = GtkBox::new(Orientation::Vertical, 0);
            spacer.set_vexpand(true);
            card.append(&spacer);

            let btn = Button::with_label(&format!("Open {}", store.name));
            btn.add_css_class("store-open-btn");
            btn.add_css_class(store.css);
            let url = store.url;
            btn.connect_clicked(move |_| {
                let _ = std::process::Command::new("xdg-open").arg(url).spawn();
            });
            card.append(&btn);

            grid.append(&card);
        }

        root.append(&grid);
        scroll.set_child(Some(&root));

        Self { widget: scroll }
    }
}
