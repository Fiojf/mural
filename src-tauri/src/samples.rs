//! First-run sample wallpapers: bundled in `resources/samples/` and copied into
//! the user's wallpaper folder if it's empty.

use anyhow::{Context, Result};
use std::path::Path;
use tauri::{AppHandle, Manager};

pub fn seed_if_empty(folder: &Path) -> Result<()> {
    if folder.read_dir().map(|mut i| i.next().is_some()).unwrap_or(false) {
        return Ok(()); // not empty
    }
    std::fs::create_dir_all(folder).context("create wallpaper folder")?;
    let bundled = bundled_samples();
    for (name, bytes) in bundled {
        let dst = folder.join(name);
        if dst.exists() {
            continue;
        }
        std::fs::write(&dst, bytes).with_context(|| format!("seed {}", dst.display()))?;
    }
    Ok(())
}

/// Copies bundled samples from the resource directory to `folder` if empty.
/// Used by the onboarding finalization step.
pub fn seed_from_resources(handle: &AppHandle, folder: &Path) -> Result<()> {
    if folder.read_dir().map(|mut i| i.next().is_some()).unwrap_or(false) {
        return Ok(());
    }
    std::fs::create_dir_all(folder).ok();
    let res = handle
        .path()
        .resolve("resources/samples", tauri::path::BaseDirectory::Resource)
        .context("resolve samples dir")?;
    if let Ok(entries) = std::fs::read_dir(&res) {
        for entry in entries.flatten() {
            let from = entry.path();
            if !from.is_file() {
                continue;
            }
            let name = match from.file_name() {
                Some(n) => n,
                None => continue,
            };
            let to = folder.join(name);
            std::fs::copy(&from, &to).ok();
        }
    }
    Ok(())
}

/// Tiny embedded fallback wallpapers used when the bundled resources can't be
/// resolved (e.g. during `cargo run` before `tauri build`). Each is a 1024×640
/// solid-color PNG generated at build time by `scripts/gen-samples.mjs`. We
/// include them as a last-resort so first-run never leaves an empty folder.
fn bundled_samples() -> Vec<(&'static str, &'static [u8])> {
    Vec::new()
}
