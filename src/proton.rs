// NovaDream — Wine/Proton auto-detection
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Runner {
    pub name:    String,
    pub kind:    RunnerKind,
    pub path:    PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunnerKind {
    Proton,
    Wine,
    System, // system wine from PATH
}

#[allow(dead_code)]
impl Runner {
    /// Returns the path to the wine/proton binary
    pub fn binary(&self) -> PathBuf {
        match self.kind {
            RunnerKind::Proton => self.path.join("proton"),
            RunnerKind::Wine   => self.path.join("bin/wine"),
            RunnerKind::System => PathBuf::from("wine"),
        }
    }

    pub fn label(&self) -> String {
        match self.kind {
            RunnerKind::System => format!("System Wine ({})", self.name),
            _                  => self.name.clone(),
        }
    }
}

/// Scan NovaDream's own proton/ folder for installed runners
pub fn detect_runners(novadream_data_dir: &PathBuf) -> Vec<Runner> {
    let mut runners = vec![];

    // Always add system wine if available
    if which_wine() {
        runners.push(Runner {
            name:  system_wine_version(),
            kind:  RunnerKind::System,
            path:  PathBuf::from("/usr"),
        });
    }

    // Scan NovaDream/proton/
    let proton_dir = novadream_data_dir.join("proton");
    if proton_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&proton_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() { continue; }

                let name = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                // Detect if it's Proton (has a 'proton' binary) or Wine (has bin/wine)
                if path.join("proton").exists() {
                    runners.push(Runner { name, kind: RunnerKind::Proton, path });
                } else if path.join("bin/wine").exists() {
                    runners.push(Runner { name, kind: RunnerKind::Wine, path });
                }
            }
        }
    }

    runners
}

fn which_wine() -> bool {
    std::process::Command::new("which")
        .arg("wine")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn system_wine_version() -> String {
    std::process::Command::new("wine")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Wine".to_string())
}
