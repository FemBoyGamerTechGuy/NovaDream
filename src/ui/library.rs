// NovaDream — library view with grid/list toggle
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, FlowBox, ListBox, ScrolledWindow,
    ToggleButton, Orientation, Label, Stack, SearchEntry,
    SelectionMode,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::game::Game;
use super::game_card::{build_grid_card, build_list_row};

#[derive(Clone, Copy, PartialEq)]
pub enum ViewMode { Grid, List }

#[allow(dead_code)]
pub struct LibraryView {
    pub widget:   GtkBox,
    games:        Rc<RefCell<Vec<Game>>>,
    view_mode:    Rc<RefCell<ViewMode>>,
    active_store: Rc<RefCell<Option<String>>>,
    grid_box:     FlowBox,
    list_box:     ListBox,
    stack:        Stack,
}

#[allow(dead_code)]
impl LibraryView {
    pub fn new() -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);

        // Toolbar
        let toolbar = GtkBox::new(Orientation::Horizontal, 8);
        toolbar.add_css_class("library-toolbar");
        toolbar.set_margin_start(16);
        toolbar.set_margin_end(16);
        toolbar.set_margin_top(12);
        toolbar.set_margin_bottom(12);

        let search = SearchEntry::new();
        search.set_placeholder_text(Some("Search games..."));
        search.set_hexpand(true);
        toolbar.append(&search);

        let grid_btn = ToggleButton::with_label("⊞ Grid");
        let list_btn = ToggleButton::with_label("☰ List");
        grid_btn.set_active(true);
        list_btn.set_group(Some(&grid_btn));
        grid_btn.add_css_class("view-toggle");
        list_btn.add_css_class("view-toggle");
        toolbar.append(&grid_btn);
        toolbar.append(&list_btn);
        root.append(&toolbar);

        // Stack
        let stack = Stack::new();

        let grid_scroll = ScrolledWindow::new();
        grid_scroll.set_vexpand(true);
        let grid_box = FlowBox::new();
        grid_box.set_valign(gtk4::Align::Start);
        grid_box.set_halign(gtk4::Align::Start);
        grid_box.set_homogeneous(false);
        grid_box.set_max_children_per_line(12);
        grid_box.set_min_children_per_line(1);
        grid_box.set_column_spacing(8);
        grid_box.set_row_spacing(8);
        grid_box.set_margin_start(16);
        grid_box.set_margin_end(16);
        grid_box.set_margin_top(12);
        grid_box.set_margin_bottom(12);
        grid_box.set_selection_mode(SelectionMode::None);
        grid_box.add_css_class("game-grid");
        grid_scroll.set_child(Some(&grid_box));
        stack.add_named(&grid_scroll, Some("grid"));

        let list_scroll = ScrolledWindow::new();
        list_scroll.set_vexpand(true);
        let list_box = ListBox::new();
        list_box.set_selection_mode(SelectionMode::None);
        list_box.add_css_class("game-list");
        list_box.set_margin_start(16);
        list_box.set_margin_end(16);
        list_box.set_margin_top(8);
        list_scroll.set_child(Some(&list_box));
        stack.add_named(&list_scroll, Some("list"));
        root.append(&stack);

        let view_mode    = Rc::new(RefCell::new(ViewMode::Grid));
        let active_store = Rc::new(RefCell::new(None::<String>));
        let games        = Rc::new(RefCell::new(vec![]));

        { let s = stack.clone(); let vm = view_mode.clone();
          grid_btn.connect_toggled(move |b| { if b.is_active() { *vm.borrow_mut() = ViewMode::Grid; s.set_visible_child_name("grid"); } }); }
        { let s = stack.clone(); let vm = view_mode.clone();
          list_btn.connect_toggled(move |b| { if b.is_active() { *vm.borrow_mut() = ViewMode::List; s.set_visible_child_name("list"); } }); }

        Self { widget: root, games, view_mode, active_store, grid_box, list_box, stack }
    }

    pub fn set_games(&self, games: Vec<Game>) {
        *self.games.borrow_mut() = games;
        self.refresh();
    }

    pub fn add_game(&self, game: Game) {
        self.games.borrow_mut().push(game);
        self.refresh();
    }

    /// Update cover path for a game by id and re-render
    pub fn update_cover(&self, game_id: &str, cover_path: std::path::PathBuf) {
        {
            let mut games = self.games.borrow_mut();
            if let Some(g) = games.iter_mut().find(|g| g.id == game_id) {
                g.cover_path = Some(cover_path.to_string_lossy().to_string());
            }
        }
        // Persist so banner survives restart
        use crate::game::Store;
        let local: Vec<_> = self.games.borrow().iter()
            .filter(|g| g.store == Store::Local)
            .cloned()
            .collect();
        crate::local_library::save_local_games(&local);
        self.refresh();
    }

    /// Return only local (manually added) games — for persistence
    pub fn local_games(&self) -> Vec<Game> {
        use crate::game::Store;
        self.games.borrow().iter()
            .filter(|g| g.store == Store::Local)
            .cloned()
            .collect()
    }

    pub fn set_store_filter(&self, store: Option<String>) {
        *self.active_store.borrow_mut() = store;
        self.refresh();
    }

    fn refresh(&self) {
        render_boxes(&self.games, &self.active_store, &self.grid_box, &self.list_box);
    }
}

/// Central render function — used by LibraryView::refresh and remove/stop callbacks
fn render_boxes(
    games:        &Rc<RefCell<Vec<Game>>>,
    active_store: &Rc<RefCell<Option<String>>>,
    grid_box:     &FlowBox,
    list_box:     &ListBox,
) {
    // Clear
    while let Some(c) = grid_box.first_child() { grid_box.remove(&c); }
    while let Some(c) = list_box.first_child()  { list_box.remove(&c); }

    let gs     = games.borrow();
    let filter = active_store.borrow();

    let filtered: Vec<&Game> = gs.iter().filter(|g| {
        match filter.as_deref() {
            None | Some("All Games") => true,
            Some(s) => g.store.label() == s,
        }
    }).collect();

    if filtered.is_empty() {
        let empty = Label::new(Some("No games found.\nClick '＋ Add Game' to add one."));
        empty.add_css_class("empty-label");
        empty.set_justify(gtk4::Justification::Center);
        grid_box.insert(&empty, -1);
        return;
    }

    for game in &filtered {
        let g2 = games.clone();
        let s2 = active_store.clone();
        let gb = grid_box.clone();
        let lb = list_box.clone();

        // ── on_remove ────────────────────────────────────────────────────────
        let on_remove_grid = {
            let g2 = g2.clone(); let s2 = s2.clone();
            let gb = gb.clone(); let lb = lb.clone();
            move |id: String| {
                g2.borrow_mut().retain(|g| g.id != id);
                // Persist the updated local list
                use crate::game::Store;
                let local: Vec<_> = g2.borrow().iter().filter(|g| g.store == Store::Local).cloned().collect();
                crate::local_library::save_local_games(&local);
                render_boxes(&g2, &s2, &gb, &lb);
            }
        };
        let on_remove_list = {
            let g2 = g2.clone(); let s2 = s2.clone();
            let gb = gb.clone(); let lb = lb.clone();
            move |id: String| {
                g2.borrow_mut().retain(|g| g.id != id);
                use crate::game::Store;
                let local: Vec<_> = g2.borrow().iter().filter(|g| g.store == Store::Local).cloned().collect();
                crate::local_library::save_local_games(&local);
                render_boxes(&g2, &s2, &gb, &lb);
            }
        };

        // ── on_stopped — update play_time + last_played, re-render ───────────
        let on_stopped_grid = {
            let g2 = g2.clone(); let s2 = s2.clone();
            let gb = gb.clone(); let lb = lb.clone();
            move |id: String, secs: u64| {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default().as_secs() as i64;
                if let Some(game) = g2.borrow_mut().iter_mut().find(|g| g.id == id) {
                    game.play_time  += secs;
                    game.last_played = Some(now);
                }
                use crate::game::Store;
                let local: Vec<_> = g2.borrow().iter().filter(|g| g.store == Store::Local).cloned().collect();
                crate::local_library::save_local_games(&local);
                render_boxes(&g2, &s2, &gb, &lb);
            }
        };
        let on_stopped_list = {
            let g2 = g2.clone(); let s2 = s2.clone();
            let gb = gb.clone(); let lb = lb.clone();
            move |id: String, secs: u64| {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default().as_secs() as i64;
                if let Some(game) = g2.borrow_mut().iter_mut().find(|g| g.id == id) {
                    game.play_time  += secs;
                    game.last_played = Some(now);
                }
                use crate::game::Store;
                let local: Vec<_> = g2.borrow().iter().filter(|g| g.store == Store::Local).cloned().collect();
                crate::local_library::save_local_games(&local);
                render_boxes(&g2, &s2, &gb, &lb);
            }
        };

        let card = build_grid_card(game, on_remove_grid, on_stopped_grid);
        grid_box.insert(&card, -1);
        // GTK wraps inserted widgets in a FlowBoxChild — clear its background
        if let Some(child) = card.parent() {
            child.add_css_class("transparent-child");
        }

        let row = build_list_row(game, on_remove_list, on_stopped_list);
        list_box.append(&row);
    }
}
