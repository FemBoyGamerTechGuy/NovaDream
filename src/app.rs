// NovaDream — GTK4 application and main window
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Label,
    Notebook, Orientation, HeaderBar, CssProvider, gdk, Separator,
};
use glib::Propagation;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{Config, novadream_data_dir};
use crate::proton::detect_runners;
use crate::stores::{EpicStore, GogStore, SteamStore, ItchStore, StoreBackend};
use crate::ui::library::LibraryView;
use crate::ui::settings::SettingsPanel;
use crate::ui::game_defaults::GameDefaultsPanel;
use crate::ui::store::StorePanel;

pub fn build_ui(app: &Application, cfg: Config) {
    let cfg = Rc::new(RefCell::new(cfg));

    // ── Detect runners ───────────────────────────────────────────────────────
    let data_dir = novadream_data_dir();
    let runners  = detect_runners(&data_dir);

    // Ensure expected data directories exist
    for subdir in &["proton", "wine", "covers", "prefixes"] {
        let _ = std::fs::create_dir_all(data_dir.join(subdir));
    }

    // ── Store backends ───────────────────────────────────────────────────────
    let epic  = Rc::new(RefCell::new(EpicStore::new()));
    let gog   = Rc::new(RefCell::new(GogStore::new()));
    let steam = Rc::new(RefCell::new(SteamStore::new()));
    let itch  = Rc::new(RefCell::new(ItchStore::new()));

    // ── Main window ─────────────────────────────────────────────────────────
    let window = ApplicationWindow::builder()
        .application(app)
        .title("NovaDream")
        .default_width(1340)
        .default_height(820)
        .build();
    window.add_css_class("novadream");

    // ── Header bar ──────────────────────────────────────────────────────────
    let header = HeaderBar::new();
    let title_lbl = Label::new(Some("🌌 NovaDream"));
    title_lbl.add_css_class("header-title");
    header.set_title_widget(Some(&title_lbl));
    window.set_titlebar(Some(&header));

    // ── Root: sidebar + content ──────────────────────────────────────────────
    let root = GtkBox::new(Orientation::Horizontal, 0);

    // ── Notebook (Library / Settings tabs) ───────────────────────────────────
    let notebook = Notebook::new();
    notebook.set_hexpand(true);
    notebook.set_vexpand(true);
    notebook.add_css_class("main-notebook");

    // Library tab — create BEFORE sidebar so sidebar can reference it
    let library = Rc::new(LibraryView::new());
    library.widget.set_hexpand(true);
    library.widget.set_vexpand(true);
    let lib_tab_lbl = Label::new(Some("🎮  Library"));
    notebook.append_page(&library.widget, Some(&lib_tab_lbl));

    // Settings tab
    let settings = SettingsPanel::new(cfg.clone());
    let set_tab_lbl = Label::new(Some("⚙  Settings"));
    notebook.append_page(&settings.widget, Some(&set_tab_lbl));

    // Game Defaults tab
    let game_defaults = GameDefaultsPanel::new(cfg.clone(), runners.clone());
    let gd_tab_lbl = Label::new(Some("🎮  Game Defaults"));
    notebook.append_page(&game_defaults.widget, Some(&gd_tab_lbl));

    // Store tab
    let store = StorePanel::new();
    let store_tab_lbl = Label::new(Some("🛒  Store"));
    notebook.append_page(&store.widget, Some(&store_tab_lbl));

    // Sidebar — gets library + notebook so Add Game can push directly in
    let sidebar = build_sidebar(
        &window, cfg.clone(), runners.clone(),
        library.clone(), notebook.clone(),
        epic.clone(), gog.clone(), steam.clone(), itch.clone(),
    );
    root.append(&sidebar);
    root.append(&Separator::new(Orientation::Vertical));

    root.append(&notebook);
    window.set_child(Some(&root));

    // ── Apply CSS ────────────────────────────────────────────────────────────
    apply_theme(&cfg.borrow().theme);

    // ── Load libraries on startup ─────────────────────────────────────────
    {
        let lib   = library.clone();
        let steam = steam.clone();
        glib::idle_add_local_once(move || {
            let mut all_games = vec![];

            // Local persisted games
            let local = crate::local_library::load_local_games();

            // Re-fetch cover if the file is missing (deleted/moved)
            // Also checks the title-keyed path in case the game was re-added
            let games_needing_covers: Vec<(String, String)> = local.iter()
                .filter(|g| {
                    let title_key = crate::config::sanitise_title(&g.title);
                    let title_path = crate::local_library::cover_path_for_title(&title_key);
                    // Already have a title-keyed cover — no fetch needed
                    if title_path.exists() { return false; }
                    // Fall back to checking whatever path is stored in the game entry
                    match &g.cover_path {
                        Some(p) => !std::path::Path::new(p).exists(),
                        None    => true,
                    }
                })
                .map(|g| (g.id.clone(), g.title.clone()))
                .collect();

            all_games.extend(local);

            // Steam library
            if let Ok(steam_games) = steam.borrow().fetch_library() {
                all_games.extend(steam_games);
            }

            lib.set_games(all_games);

            // Re-fetch missing covers in background
            if !games_needing_covers.is_empty() {
                let lib2 = lib.clone();
                let (tx, rx) = std::sync::mpsc::channel::<(String, std::path::PathBuf)>();
                std::thread::spawn(move || {
                    for (id, title) in games_needing_covers {
                        if let Some(path) = crate::local_library::fetch_cover(&title, &id) {
                            let _ = tx.send((id, path));
                        }
                    }
                });
                glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                    match rx.try_recv() {
                        Ok((id, path)) => { lib2.update_cover(&id, path); glib::ControlFlow::Continue }
                        Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                        Err(_) => glib::ControlFlow::Break,
                    }
                });
            }
        });
    }

    // ── System tray ──────────────────────────────────────────────────────────
    if cfg.borrow().show_tray {
        let rx = crate::tray::spawn_tray();
        let win_t = window.clone();
        let app_t = app.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            use crate::tray::TrayEvent;
            match rx.try_recv() {
                Ok(TrayEvent::Show) => { win_t.present(); }
                Ok(TrayEvent::Quit) => { win_t.close(); app_t.quit(); }
                Err(_) => {}
            }
            glib::ControlFlow::Continue
        });

        if cfg.borrow().close_to_tray {
            window.connect_close_request(|win| {
                win.set_visible(false);
                Propagation::Stop
            });
        }
    }

    // Give library access to window + runners so the per-game settings dialog can open
    library.set_window(window.clone());
    library.set_runners(runners.clone());

    window.present();
}

fn build_sidebar(
    window:   &ApplicationWindow,
    _cfg:     Rc<RefCell<Config>>,
    runners:  Vec<crate::proton::Runner>,
    library:  Rc<LibraryView>,
    notebook: Notebook,
    epic:     Rc<RefCell<EpicStore>>,
    gog:      Rc<RefCell<GogStore>>,
    _steam:   Rc<RefCell<SteamStore>>,
    itch:     Rc<RefCell<ItchStore>>,
) -> GtkBox {
    let sidebar = GtkBox::new(Orientation::Vertical, 2);
    sidebar.set_width_request(230);
    sidebar.add_css_class("sidebar");
    sidebar.set_margin_top(12);
    sidebar.set_margin_bottom(12);
    sidebar.set_margin_start(8);
    sidebar.set_margin_end(8);

    // Logo
    let logo = Label::new(Some("🌌 NovaDream"));
    logo.add_css_class("sidebar-title");
    logo.set_margin_bottom(8);
    sidebar.append(&logo);

    // ── Add Game button ──────────────────────────────────────────────────────
    let add_btn = Button::with_label("＋  Add Game");
    add_btn.add_css_class("add-game-btn");
    add_btn.set_margin_bottom(8);
    {
        let win  = window.clone();
        let run  = runners.clone();
        let lib  = library.clone();
        let nb   = notebook.clone();
        add_btn.connect_clicked(move |_| {
            if let Some(game) = crate::ui::add_game::show_add_game_dialog(&win, &run) {
                // Fetch cover in background — send result back to GTK thread via channel
                let (tx, rx) = std::sync::mpsc::channel::<(String, std::path::PathBuf)>();
                let game_id2    = game.id.clone();
                let game_title2 = game.title.clone();
                std::thread::spawn(move || {
                    if let Some(path) = crate::local_library::fetch_cover(&game_title2, &game_id2) {
                        let _ = tx.send((game_id2, path));
                    }
                });

                // Poll on GTK thread until cover arrives, then update
                {
                    let lib2 = lib.clone();
                    glib::timeout_add_local_once(
                        std::time::Duration::from_millis(200),
                        move || {
                            // Poll a few times until result arrives
                            let lib3 = lib2.clone();
                            glib::timeout_add_local(
                                std::time::Duration::from_millis(500),
                                move || match rx.try_recv() {
                                    Ok((id, path)) => {
                                        lib3.update_cover(&id, path);
                                        glib::ControlFlow::Break
                                    }
                                    Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                                    Err(_) => glib::ControlFlow::Break,
                                },
                            );
                        },
                    );
                }

                lib.add_game(game);

                // Save local library to disk
                let local_games = lib.local_games();
                crate::local_library::save_local_games(&local_games);

                nb.set_current_page(Some(0));
            }
        });
    }
    sidebar.append(&add_btn);
    sidebar.append(&Separator::new(Orientation::Horizontal));

    // ── Store filter buttons ─────────────────────────────────────────────────
    let filter_lbl = Label::new(Some("LIBRARY"));
    filter_lbl.add_css_class("sidebar-section");
    sidebar.append(&filter_lbl);

    for (label, store) in &[
        ("🎮  All Games",   None),
        ("🔵  Epic Games",  Some("Epic")),
        ("🟣  GOG",         Some("GOG")),
        ("🔷  Steam",       Some("Steam")),
        ("🔴  Itch.io",     Some("Itch.io")),
        ("📁  Local",       Some("Local")),
    ] {
        let btn = Button::with_label(label);
        btn.add_css_class("sidebar-btn");
        let lib2 = library.clone();
        let store_val = store.map(|s| s.to_string());
        btn.connect_clicked(move |_| {
            lib2.set_store_filter(store_val.clone());
        });
        sidebar.append(&btn);
    }

    // Spacer
    let spacer = GtkBox::new(Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    sidebar.append(&spacer);

    sidebar.append(&Separator::new(Orientation::Horizontal));

    // ── Accounts ─────────────────────────────────────────────────────────────
    let acc_lbl = Label::new(Some("ACCOUNTS"));
    acc_lbl.add_css_class("sidebar-section");
    sidebar.append(&acc_lbl);

    // Epic
    {
        let win  = window.clone();
        let eref = epic.clone();
        let auth = epic.borrow().is_authenticated();
        let btn  = sidebar_account_btn("Epic Games", auth);
        btn.connect_clicked(move |b| {
            if !eref.borrow().is_authenticated() {
                if let Some(url) = eref.borrow().auth_url() {
                    if let Some(cb) = crate::ui::login::show_oauth_dialog(&win, "Epic Games", &url) {
                        if eref.borrow_mut().handle_oauth_callback(&cb).is_ok() {
                            b.set_label("✓ Epic Games");
                            b.add_css_class("account-ok");
                        }
                    }
                }
            }
        });
        sidebar.append(&btn);
    }

    // GOG
    {
        let win  = window.clone();
        let gref = gog.clone();
        let auth = gog.borrow().is_authenticated();
        let btn  = sidebar_account_btn("GOG", auth);
        btn.connect_clicked(move |b| {
            if !gref.borrow().is_authenticated() {
                if let Some(url) = gref.borrow().auth_url() {
                    if let Some(cb) = crate::ui::login::show_oauth_dialog(&win, "GOG", &url) {
                        if gref.borrow_mut().handle_oauth_callback(&cb).is_ok() {
                            b.set_label("✓ GOG");
                            b.add_css_class("account-ok");
                        }
                    }
                }
            }
        });
        sidebar.append(&btn);
    }

    // Itch
    {
        let win  = window.clone();
        let iref = itch.clone();
        let auth = itch.borrow().is_authenticated();
        let btn  = sidebar_account_btn("Itch.io", auth);
        btn.connect_clicked(move |b| {
            if !iref.borrow().is_authenticated() {
                if let Some(cb) = crate::ui::login::show_apikey_dialog(&win) {
                    if iref.borrow_mut().handle_oauth_callback(&cb).is_ok() {
                        b.set_label("✓ Itch.io");
                        b.add_css_class("account-ok");
                    }
                }
            }
        });
        sidebar.append(&btn);
    }

    // Steam — no login needed
    {
        let btn = Button::with_label("✓ Steam (local)");
        btn.add_css_class("sidebar-btn");
        btn.add_css_class("account-ok");
        btn.set_sensitive(false);
        sidebar.append(&btn);
    }

    sidebar
}

fn sidebar_account_btn(store: &str, authenticated: bool) -> Button {
    let label = if authenticated {
        format!("✓ {}", store)
    } else {
        format!("Log in to {}", store)
    };
    let btn = Button::with_label(&label);
    btn.add_css_class("sidebar-btn");
    if authenticated { btn.add_css_class("account-ok"); }
    btn
}

// Global CSS provider so we can REMOVE the old one before adding the new one.
// This is what prevents system theme bleed — without removal the old provider
// stays active at the same priority and both fight each other.
thread_local! {
    static THEME_PROVIDER: RefCell<Option<CssProvider>> = RefCell::new(None);
}

pub fn apply_theme(theme_name: &str) {
    let display = gdk::Display::default().expect("No display");

    THEME_PROVIDER.with(|cell| {
        // Remove the previous custom provider if one exists
        if let Some(old) = cell.borrow().as_ref() {
            gtk4::style_context_remove_provider_for_display(&display, old);
        }

        if theme_name == "system" {
            // Let GTK use its own theme — no custom CSS at all
            *cell.borrow_mut() = None;
            return;
        }

        // Load our custom theme at priority 800 (> APPLICATION=600, < USER=900)
        // This ensures our CSS fully overrides Adwaita/system theme widgets
        let css = theme_css(theme_name);
        let provider = CssProvider::new();
        provider.load_from_string(&css);
        gtk4::style_context_add_provider_for_display(&display, &provider, 800);
        *cell.borrow_mut() = Some(provider);
    });
}

fn theme_css(name: &str) -> String {
    let (base, surface, surface2, text, accent, muted, green) = match name {
        "catppuccin-macchiato"  => ("#24273a","#1e2030","#2a2d3e","#cad3f5","#c6a0f6","#6e738d","#a6da95"),
        "catppuccin-mocha"      => ("#1e1e2e","#181825","#232634","#cdd6f4","#cba6f7","#6c7086","#a6e3a1"),
        "catppuccin-latte"      => ("#eff1f5","#e6e9ef","#dce0e8","#4c4f69","#8839ef","#8c8fa1","#40a02b"),
        "catppuccin-frappe"     => ("#303446","#292c3c","#414559","#c6d0f5","#ca9ee6","#626880","#a6d189"),
        "dracula"               => ("#282a36","#21222c","#343746","#f8f8f2","#bd93f9","#6272a4","#50fa7b"),
        "tokyo-night"           => ("#1a1b26","#16161e","#1f2335","#c0caf5","#7aa2f7","#565f89","#9ece6a"),
        "tokyo-night-storm"     => ("#24283b","#1f2335","#292e42","#c0caf5","#7aa2f7","#565f89","#9ece6a"),
        "nord"                  => ("#2e3440","#272c36","#3b4252","#eceff4","#88c0d0","#4c566a","#a3be8c"),
        "gruvbox"               => ("#282828","#1d2021","#3c3836","#ebdbb2","#d3869b","#928374","#b8bb26"),
        "gruvbox-light"         => ("#fbf1c7","#f2e5bc","#ebdbb2","#3c3836","#8f3f71","#a89984","#79740e"),
        "rose-pine"             => ("#191724","#1f1d2e","#26233a","#e0def4","#c4a7e7","#6e6a86","#31748f"),
        "rose-pine-moon"        => ("#232136","#2a273f","#393552","#e0def4","#c4a7e7","#6e6a86","#3e8fb0"),
        "rose-pine-dawn"        => ("#faf4ed","#fffaf3","#f2e9e1","#575279","#907aa9","#9893a5","#286983"),
        "everforest"            => ("#2d353b","#272e33","#3d484d","#d3c6aa","#a7c080","#859289","#a7c080"),
        "kanagawa"              => ("#1f1f28","#16161d","#2a2a37","#dcd7ba","#957fb8","#727169","#76946a"),
        "material-ocean"        => ("#0f111a","#090b10","#1a1c25","#8f93a2","#82aaff","#464b5d","#c3e88d"),
        "one-dark"              => ("#282c34","#21252b","#2c313c","#abb2bf","#c678dd","#5c6370","#98c379"),
        "solarized-dark"        => ("#002b36","#073642","#003847","#839496","#6c71c4","#657b83","#859900"),
        "solarized-light"       => ("#fdf6e3","#eee8d5","#e8e2cc","#657b83","#6c71c4","#93a1a1","#859900"),
        "monokai"               => ("#272822","#1e1f1c","#3e3d32","#f8f8f2","#ae81ff","#75715e","#a6e22e"),
        "ayu-dark"              => ("#0d1017","#0a0e14","#131721","#bfbdb6","#d2a6ff","#3d424d","#7fd962"),
        "ayu-mirage"            => ("#1f2430","#191e2a","#232834","#cccac2","#d2a6ff","#3d424d","#d5ff80"),
        "ayu-light"             => ("#fafafa","#f3f4f5","#e7e8e9","#5c6166","#a37acc","#8a9199","#4cbf99"),
        _                       => ("#24273a","#1e2030","#2a2d3e","#cad3f5","#c6a0f6","#6e738d","#a6da95"),
    };

    format!(r#"
/* ── NovaDream — Cosmic UI ───────────────────────────────────────────────── */
/* Scope to NovaDream windows only — prevents bleed into file chooser portal */
window.novadream label,
window.novadream entry,
window.novadream text {{ color: {text}; background-color: transparent; }}
window.novadream label * {{ background-color: transparent; }}
window.novadream flowbox,
window.novadream listbox {{ background-color: transparent; }}
window.novadream flowboxchild,
window.novadream .transparent-child {{
    background-color: transparent;
    border: none;
    padding: 0;
    margin: 0;
    outline: none;
    box-shadow: none;
}}
window.novadream flowboxchild:selected,
window.novadream flowboxchild:focus,
window.novadream flowboxchild:hover,
window.novadream flowboxchild:active,
window.novadream .transparent-child:selected,
window.novadream .transparent-child:focus,
window.novadream .transparent-child:hover,
window.novadream .transparent-child:active {{
    background-color: transparent;
    box-shadow: none;
    outline: none;
}}
/* Kill ALL backgrounds inside game cards so Adwaita can't bleed through */
.game-card * {{ background-color: transparent; }}
.game-card label {{ color: white; background-color: transparent; }}
.game-card box {{ background-color: transparent; }}
.game-card overlay {{ background-color: transparent; }}
/* Restore intentional backgrounds */
.game-card .cover-placeholder {{ background-color: alpha({accent}, 0.07); }}
.game-card .card-info {{ background: linear-gradient(transparent, alpha(black, 0.92)); }}
.game-card .store-badge {{ background: alpha({accent}, 0.18); }}
.game-card .btn-play {{ background: {accent}; }}
.game-card .btn-stop {{ background: #c0392b; }}
.game-card .btn-install {{ background: alpha({accent}, 0.12); }}
.game-card .card-remove-btn {{ background: transparent; }}

/* ── Base ────────────────────────────────────────────────────────── */
window, .background, widget, box, scrolledwindow, viewport {{
    background-color: {base};
    color: {text};
    font-family: "Outfit", "Rubik", "Noto Sans", sans-serif;
    font-size: 14px;
}}
window, .background {{
    background-color: {base};
    color: {text};
}}

/* ── Header bar ──────────────────────────────────────────────────── */
headerbar {{
    background: linear-gradient(180deg, alpha({accent}, 0.08) 0%, {surface} 100%);
    color: {text};
    border-bottom: 1px solid alpha({accent}, 0.25);
    box-shadow: 0 2px 20px alpha(black, 0.4);
    min-height: 54px;
    padding: 0 12px;
}}
.header-title {{
    font-size: 17px;
    font-weight: 800;
    color: {accent};
    letter-spacing: 2px;
    text-transform: uppercase;
}}

/* ── Sidebar ─────────────────────────────────────────────────────── */
.sidebar {{
    background: linear-gradient(180deg, alpha({accent}, 0.04) 0%, {surface} 60%);
    border-right: 1px solid alpha({accent}, 0.12);
}}
.sidebar-title {{
    color: {accent};
    font-size: 18px;
    font-weight: 900;
    letter-spacing: 3px;
    text-transform: uppercase;
    padding: 8px 12px 4px 12px;
}}
.sidebar-section {{
    color: {muted};
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 2.5px;
    text-transform: uppercase;
    padding: 14px 12px 5px 12px;
}}
.sidebar-btn {{
    background: transparent;
    color: alpha({text}, 0.75);
    border: none;
    border-radius: 10px;
    padding: 8px 14px;
    font-size: 13px;
    font-weight: 500;
    min-height: 36px;
    transition: all 150ms;
}}
.sidebar-btn:hover {{
    background: linear-gradient(90deg, alpha({accent}, 0.15), alpha({accent}, 0.05));
    color: {text};
    border-left: 3px solid {accent};
    padding-left: 11px;
}}
.sidebar-btn:active {{
    background: alpha({accent}, 0.22);
}}
.add-game-btn {{
    background: linear-gradient(135deg, alpha({accent}, 0.25), alpha({accent}, 0.12));
    color: {accent};
    border: 1px solid alpha({accent}, 0.35);
    border-radius: 10px;
    padding: 9px 14px;
    font-weight: 700;
    font-size: 13px;
    letter-spacing: 0.5px;
    box-shadow: 0 2px 12px alpha({accent}, 0.15);
}}
.add-game-btn:hover {{
    background: linear-gradient(135deg, alpha({accent}, 0.35), alpha({accent}, 0.2));
    border-color: {accent};
    box-shadow: 0 4px 20px alpha({accent}, 0.3);
}}
.account-ok {{
    color: {green};
}}

/* ── Notebook tabs ───────────────────────────────────────────────── */
.main-notebook > header {{
    background: {surface};
    border-bottom: 1px solid alpha({accent}, 0.18);
    padding: 0 10px;
}}
.main-notebook > header > tabs > tab {{
    color: {muted};
    padding: 11px 22px;
    font-size: 13px;
    font-weight: 500;
    letter-spacing: 0.3px;
    border-bottom: 2px solid transparent;
}}
.main-notebook > header > tabs > tab:checked {{
    color: {accent};
    border-bottom: 2px solid {accent};
    background: linear-gradient(180deg, transparent, alpha({accent}, 0.06));
}}

/* ── Library toolbar ─────────────────────────────────────────────── */
.library-toolbar {{
    background: linear-gradient(180deg, {surface2}, alpha({surface2}, 0.7));
    border-bottom: 1px solid alpha({accent}, 0.12);
    padding: 10px 18px;
}}
.view-toggle {{
    background: alpha({accent}, 0.06);
    color: {muted};
    border: 1px solid alpha({accent}, 0.18);
    border-radius: 8px;
    padding: 5px 16px;
    font-size: 12px;
    min-height: 32px;
    font-weight: 500;
}}
.view-toggle:checked {{
    background: alpha({accent}, 0.2);
    color: {accent};
    border-color: alpha({accent}, 0.45);
    box-shadow: 0 0 10px alpha({accent}, 0.15);
}}

/* ── Game cards ──────────────────────────────────────────────────── */
.game-card {{
    background: linear-gradient(160deg, alpha({accent}, 0.06) 0%, {surface} 60%);
    border-radius: 14px;
    border: 1px solid alpha({accent}, 0.14);
    box-shadow: 0 4px 24px alpha(black, 0.3);
    transition: all 180ms;
}}
.game-card:hover {{
    border-color: alpha({accent}, 0.55);
    box-shadow: 0 8px 32px alpha({accent}, 0.2), 0 0 0 1px alpha({accent}, 0.08);
}}
.cover-placeholder {{
    background: linear-gradient(160deg, alpha({accent}, 0.18) 0%, alpha({surface}, 0.9) 100%);
    min-height: 220px;
    border-radius: 14px 14px 0 0;
}}
.cover-image {{
    min-height: 220px;
    border-radius: 14px 14px 0 0;
}}
.card-info {{
    padding: 10px 10px 6px 10px;
    border-radius: 0 0 14px 14px;
    background: linear-gradient(transparent, alpha(black, 0.88));
}}
.card-title {{
    font-size: 13px;
    font-weight: 700;
    color: white;
    letter-spacing: 0.2px;
}}
.card-meta {{
    font-size: 11px;
    color: alpha(white, 0.6);
}}
.store-badge {{
    font-size: 10px;
    font-weight: 700;
    color: {accent};
    background: alpha({accent}, 0.2);
    border-radius: 5px;
    padding: 2px 7px;
    margin-bottom: 2px;
    letter-spacing: 0.5px;
}}
.btn-play {{
    background: linear-gradient(135deg, {accent}, alpha({accent}, 0.75));
    color: {base};
    border: none;
    border-radius: 0 0 14px 14px;
    font-weight: 700;
    padding: 8px;
    font-size: 13px;
    letter-spacing: 0.5px;
    box-shadow: 0 -2px 12px alpha({accent}, 0.2);
}}
.btn-play:hover {{
    background: linear-gradient(135deg, alpha({accent}, 0.9), alpha({accent}, 0.65));
    box-shadow: 0 -2px 20px alpha({accent}, 0.4);
}}
.btn-stop {{
    background: linear-gradient(135deg, #c0392b, #96281b);
    color: white;
    border: none;
    border-radius: 0 0 14px 14px;
    font-weight: 700;
    padding: 8px;
    font-size: 13px;
}}
.btn-stop:hover {{ background: linear-gradient(135deg, #e74c3c, #c0392b); }}
.btn-install {{
    background: alpha({accent}, 0.1);
    color: {accent};
    border: none;
    border-radius: 0 0 14px 14px;
    padding: 8px;
    font-size: 13px;
    font-weight: 600;
}}
.btn-install:hover {{ background: alpha({accent}, 0.2); }}
.btn-play-small {{
    background: linear-gradient(135deg, {accent}, alpha({accent}, 0.8));
    color: {base};
    border: none;
    border-radius: 8px;
    font-weight: 700;
    padding: 5px 16px;
    font-size: 12px;
}}
.btn-play-small:hover {{ background: linear-gradient(135deg, alpha({accent}, 0.9), alpha({accent}, 0.7)); }}
.btn-install-small {{
    background: alpha({accent}, 0.1);
    color: {accent};
    border: none;
    border-radius: 8px;
    padding: 5px 16px;
    font-size: 12px;
    font-weight: 600;
}}
.btn-install-small:hover {{ background: alpha({accent}, 0.2); }}

/* ── Remove button ───────────────────────────────────────────────── */
.card-remove-btn {{
    background: transparent;
    color: alpha(white, 0.45);
    border: none;
    border-radius: 5px;
    padding: 0px 5px;
    font-size: 14px;
    min-height: 0;
    min-width: 0;
}}
.card-remove-btn:hover {{
    background: alpha(red, 0.6);
    color: white;
}}

/* ── List rows ───────────────────────────────────────────────────── */
.game-list {{ background: transparent; }}
.game-list > row {{
    padding: 2px 0;
    background-color: transparent;
}}
.game-list > row:hover {{ background-color: transparent; }}
.game-list > row:selected {{ background-color: transparent; }}
.list-row {{
    min-height: 0;
    border-left: 3px solid transparent;
    border-bottom: 1px solid alpha({accent}, 0.07);
    padding-left: 12px;
    transition: border-color 120ms;
}}
.list-row:hover {{
    border-left-color: {accent};
    background: linear-gradient(90deg, alpha({accent}, 0.07), transparent);
}}
.list-title {{
    font-size: 14px;
    font-weight: 600;
    color: {text};
}}
.empty-label {{
    color: {muted};
    font-size: 16px;
    padding: 64px;
    letter-spacing: 0.3px;
}}

/* ── Theme picker ────────────────────────────────────────────────── */
.theme-scroll {{
    border: 1px solid alpha({accent}, 0.2);
    border-radius: 12px;
    background: {surface2};
}}
.theme-list {{ background: transparent; }}
.theme-group-header {{
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 2px;
    color: {muted};
    text-transform: uppercase;
}}
.theme-btn {{
    background: transparent;
    border: none;
    border-radius: 8px;
    padding: 2px 0;
}}
.theme-btn:hover {{ background: alpha({accent}, 0.1); }}
.theme-name {{
    font-size: 13px;
    color: {text};
    font-weight: 500;
}}
.theme-check {{
    font-size: 13px;
    color: {accent};
    font-weight: 700;
}}

/* ── Settings ────────────────────────────────────────────────────── */
.settings-section {{
    font-size: 11px;
    font-weight: 700;
    color: {accent};
    letter-spacing: 2px;
    text-transform: uppercase;
    margin-top: 10px;
}}
.setting-title {{
    font-size: 14px;
    font-weight: 600;
    color: {text};
}}
.setting-hint {{
    font-size: 12px;
    color: {muted};
    font-weight: 400;
}}
.settings-drop {{
    background: {surface2};
    color: {text};
    border: 1px solid alpha({accent}, 0.25);
    border-radius: 10px;
    padding: 5px 10px;
    min-width: 200px;
}}
.settings-entry {{
    background: {surface2};
    color: {text};
    border: 1px solid alpha({accent}, 0.2);
    border-radius: 10px;
    padding: 7px 12px;
}}
.settings-hint {{
    color: {muted};
    font-size: 13px;
}}

/* ── Add game dialog ─────────────────────────────────────────────── */
.add-game-dialog {{ background-color: {surface}; }}
.field-label {{
    font-size: 13px;
    font-weight: 600;
    color: {text};
    margin-bottom: 2px;
}}
.browse-btn {{
    background: alpha({accent}, 0.1);
    color: {accent};
    border: 1px solid alpha({accent}, 0.22);
    border-radius: 10px;
    padding: 6px 14px;
    font-size: 13px;
    font-weight: 500;
}}
.browse-btn:hover {{ background: alpha({accent}, 0.2); }}

/* ── Inputs ──────────────────────────────────────────────────────── */
entry {{
    background-color: {surface2};
    color: {text};
    border: 1px solid alpha({accent}, 0.22);
    border-radius: 10px;
    padding: 7px 12px;
    caret-color: {accent};
}}
entry:focus {{
    border-color: {accent};
    box-shadow: 0 0 0 2px alpha({accent}, 0.12);
}}
separator {{
    background: linear-gradient(90deg, transparent, alpha({accent}, 0.15), transparent);
    min-height: 1px;
    min-width: 1px;
}}
button.flat {{
    background: transparent;
    color: {muted};
    border: none;
}}
button.flat:hover {{ color: {text}; }}
button.suggested-action {{
    background: linear-gradient(135deg, {accent}, alpha({accent}, 0.8));
    color: {base};
    border: none;
    border-radius: 10px;
    padding: 7px 20px;
    font-weight: 700;
    letter-spacing: 0.5px;
    box-shadow: 0 3px 14px alpha({accent}, 0.3);
}}
button.suggested-action:hover {{
    box-shadow: 0 5px 20px alpha({accent}, 0.45);
}}
/* ── DropDown / ComboBox / Popover ───────────────────────────────── */
dropdown, combobox {{
    background: {surface2};
    color: {text};
    border: 1px solid alpha({accent}, 0.22);
    border-radius: 10px;
}}
dropdown > button, combobox > button {{
    background: {surface2};
    color: {text};
    border: 1px solid alpha({accent}, 0.22);
    border-radius: 10px;
    padding: 5px 10px;
}}
dropdown > button:hover, combobox > button:hover {{
    background: alpha({accent}, 0.1);
    border-color: alpha({accent}, 0.4);
}}
popover, .popover {{
    background-color: {surface};
    color: {text};
    border: 1px solid alpha({accent}, 0.2);
    border-radius: 12px;
    box-shadow: 0 8px 32px alpha(black, 0.5);
}}
popover > contents, .popover > contents {{
    background-color: {surface};
    border-radius: 12px;
    padding: 4px;
}}
listview, .popover listview {{
    background: transparent;
    color: {text};
}}
listview > row, .popover listview > row {{
    background: transparent;
    color: {text};
    padding: 6px 10px;
    border-radius: 8px;
}}
listview > row:hover, .popover listview > row:hover {{
    background: alpha({accent}, 0.12);
    color: {text};
}}
listview > row:selected, .popover listview > row:selected {{
    background: alpha({accent}, 0.22);
    color: {accent};
}}
button.suggested-action:hover {{ background: alpha({accent}, 0.82); }}
/* ── Store tab ───────────────────────────────────────────────────── */
.store-heading {{
    font-size: 22px;
    font-weight: bold;
    color: {text};
}}
.store-subheading {{
    font-size: 13px;
    color: {muted};
    margin-bottom: 8px;
}}
.store-card {{
    background: {surface};
    border: 1px solid alpha({accent}, 0.12);
    border-radius: 14px;
    padding: 20px 16px 16px 16px;
    min-height: 220px;
    transition: border-color 120ms;
}}
.store-card:hover {{
    border-color: alpha({accent}, 0.4);
}}
.store-emoji {{
    font-size: 36px;
    margin-bottom: 4px;
}}
.store-name {{
    font-size: 16px;
    font-weight: bold;
    color: {text};
}}
.store-desc {{
    font-size: 12px;
    color: {muted};
    margin-top: 4px;
}}
.store-open-btn {{
    border-radius: 8px;
    font-weight: bold;
    font-size: 13px;
    padding: 7px 12px;
    border: none;
    margin-top: 8px;
}}
.store-btn-steam  {{ background: #1b2838; color: #c7d5e0; }}
.store-btn-steam:hover  {{ background: #2a475e; }}
.store-btn-epic   {{ background: #2d2d2d; color: white; }}
.store-btn-epic:hover   {{ background: #444; }}
.store-btn-gog    {{ background: #7a2c8c; color: white; }}
.store-btn-gog:hover    {{ background: #9b38b0; }}
.store-btn-itch   {{ background: #fa5c5c; color: white; }}
.store-btn-itch:hover   {{ background: #ff7070; }}
/* ── Per-game settings dialog ────────────────────────────────────── */
.game-settings-section {{
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 2px;
    color: {accent};
    text-transform: uppercase;
    margin-top: 10px;
}}
.game-settings-label {{
    color: {muted};
    font-size: 13px;
    font-weight: 500;
}}
.game-settings-hint {{
    font-size: 11px;
    color: {muted};
}}
.env-scroll {{
    border: 1px solid alpha({accent}, 0.18);
    border-radius: 10px;
}}
.dxvk-btn {{
    background: alpha({accent}, 0.1);
    color: {text};
    border: 1px solid alpha({accent}, 0.28);
    border-radius: 10px;
    padding: 7px 16px;
    font-weight: 600;
    letter-spacing: 0.3px;
}}
.dxvk-btn:hover {{
    background: alpha({accent}, 0.22);
    border-color: alpha({accent}, 0.5);
}}
"#)
}
