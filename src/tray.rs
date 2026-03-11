// NovaDream — system tray via StatusNotifierItem
// SPDX-License-Identifier: GPL-3.0-or-later

use ksni::{self, MenuItem, Tray as KsniTray, TrayMethods, OfflineReason};
use std::sync::mpsc::{self, Sender};

pub enum TrayEvent { Show, Quit }

struct NovaDreamTray { tx: Sender<TrayEvent> }

impl KsniTray for NovaDreamTray {
    fn id(&self) -> String { env!("CARGO_PKG_NAME").into() }
    fn title(&self) -> String { "NovaDream".into() }
    fn icon_name(&self) -> String { "input-gaming".into() }

    fn watcher_online(&self) {
        eprintln!("[tray] SNI watcher online — icon registered");
    }

    fn watcher_offline(&self, reason: OfflineReason) -> bool {
        eprintln!("[tray] SNI watcher offline: {:?}", reason);
        true // keep retrying
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send(TrayEvent::Show);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let tx_show = self.tx.clone();
        let tx_quit = self.tx.clone();
        vec![
            MenuItem::Standard(ksni::menu::StandardItem {
                label: "Show NovaDream".into(),
                activate: Box::new(move |_| { let _ = tx_show.send(TrayEvent::Show); }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(ksni::menu::StandardItem {
                label: "Quit".into(),
                activate: Box::new(move |_| { let _ = tx_quit.send(TrayEvent::Quit); }),
                ..Default::default()
            }),
        ]
    }
}

pub fn spawn_tray() -> mpsc::Receiver<TrayEvent> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        eprintln!("[tray] starting tokio runtime...");
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        eprintln!("[tray] calling spawn()...");
        let result = rt.block_on(NovaDreamTray { tx }.spawn());
        match result {
            Ok(_handle) => {
                eprintln!("[tray] spawn() succeeded, keeping handle alive");
                // Keep the thread (and runtime) alive so D-Bus messages keep processing
                loop { std::thread::sleep(std::time::Duration::from_secs(60)); }
            }
            Err(e) => eprintln!("[tray] spawn() failed: {}", e),
        }
    });
    rx
}
