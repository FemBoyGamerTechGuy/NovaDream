// NovaDream — a dreamy void-themed game launcher
// Copyright (C) 2026  FemBoyGamerTechGuy
// SPDX-License-Identifier: GPL-3.0-or-later

mod app;
mod config;
mod game;
mod local_library;
mod proton;
mod stores;
mod tray;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;

const APP_ID: &str = "com.fadeddream.NovaDream";

fn main() {
    let application = Application::builder()
        .application_id(APP_ID)
        .build();

    application.connect_activate(|app| {
        let cfg = config::Config::load();
        app::build_ui(app, cfg);
    });

    application.run();
}
