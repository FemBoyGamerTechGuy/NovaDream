// NovaDream — UMU launcher auto-download
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;
use std::process::Command;

pub fn bundled_umu_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("NovaDream/runtimes/umu/umu-run")
}

pub fn resolve_umu() -> Option<String> {
    if which_bin("umu-run") {
        return Some("umu-run".to_string());
    }
    let bundled = bundled_umu_path();
    if bundled.exists() {
        return Some(bundled.to_string_lossy().to_string());
    }
    None
}

pub fn ensure_umu_blocking(
    on_progress: impl Fn(u64, Option<u64>),
) -> anyhow::Result<String> {
    if let Some(path) = resolve_umu() {
        return Ok(path);
    }

    let dest = bundled_umu_path();
    let dest_dir = dest.parent().unwrap();
    std::fs::create_dir_all(dest_dir)?;

    let client = reqwest::blocking::Client::builder()
        .user_agent("NovaDream/0.1 (github.com/FemBoyGamerTechGuy/NovaDream)")
        .build()?;

    // Get latest release from GitHub API
    let api_url = "https://api.github.com/repos/Open-Wine-Components/umu-launcher/releases/latest";
    let release: serde_json::Value = client.get(api_url)
        .header("Accept", "application/vnd.github+json")
        .send()?
        .json()?;

    // Find the zipapp tar asset (e.g. "umu-launcher-1.3.0-zipapp.tar")
    let download_url = release["assets"]
        .as_array()
        .and_then(|assets| assets.iter().find(|a| {
            a["name"].as_str()
                .map(|n| n.contains("zipapp") && n.ends_with(".tar"))
                .unwrap_or(false)
        }))
        .and_then(|a| a["browser_download_url"].as_str().map(|s| s.to_string()))
        .ok_or_else(|| anyhow::anyhow!("No zipapp.tar asset found in latest umu-launcher release"))?;

    eprintln!("NovaDream: downloading {download_url}");

    // Download the tar
    let mut resp = client.get(&download_url).send()?;
    if !resp.status().is_success() {
        anyhow::bail!("HTTP {} downloading umu-launcher", resp.status());
    }

    let total = resp.content_length();
    let mut downloaded: u64 = 0;
    let mut tar_bytes: Vec<u8> = Vec::new();

    loop {
        use std::io::Read;
        let mut buf = vec![0u8; 65536];
        let n = resp.read(&mut buf)?;
        if n == 0 { break; }
        tar_bytes.extend_from_slice(&buf[..n]);
        downloaded += n as u64;
        on_progress(downloaded, total);
    }

    // Extract umu-run from the tar (it's a flat tar, file is named "umu-run")
    let mut archive = tar::Archive::new(std::io::Cursor::new(&tar_bytes));
    let mut found = false;
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();
        if path.file_name().map(|n| n == "umu-run").unwrap_or(false) {
            entry.unpack(&dest)?;
            found = true;
            break;
        }
    }

    if !found {
        anyhow::bail!("umu-run not found inside the downloaded tar");
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dest)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest, perms)?;
    }

    eprintln!("NovaDream: umu-run installed to {}", dest.display());
    Ok(dest.to_string_lossy().to_string())
}

fn which_bin(name: &str) -> bool {
    Command::new("which").arg(name)
        .output().map(|o| o.status.success()).unwrap_or(false)
}
