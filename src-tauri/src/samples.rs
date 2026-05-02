//! First-run sample wallpapers: bundled in `resources/samples/` and copied into
//! the user's wallpaper folder if it's empty.

use anyhow::{Context, Result};
use std::path::Path;
use tauri::{AppHandle, Manager};

/// Copies bundled samples from the resource directory to `folder` if empty.
/// Used by the onboarding finalization step.
pub fn seed_from_resources(handle: &AppHandle, folder: &Path) -> Result<()> {
    if folder
        .read_dir()
        .map(|mut i| i.next().is_some())
        .unwrap_or(false)
    {
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
