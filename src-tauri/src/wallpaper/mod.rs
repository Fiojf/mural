//! macOS wallpaper application + per-Space + lock-screen mirror + video.

pub mod image;
pub mod lock_screen;
pub mod per_space;
pub mod video;

use anyhow::Result;
use std::path::Path;

use crate::scan::{classify, Kind};
use crate::state::AppState;

/// Apply `path` as the wallpaper.
///
/// `display_id`: when `Some`, only that display is updated. When `None`, applies
/// to all displays (subject to `per_screen` config).
pub fn apply(state: &AppState, path: &Path, display_id: Option<&str>) -> Result<()> {
    let kind = classify(path).unwrap_or(Kind::Image);
    let cfg = state.config.read().clone();
    match kind {
        Kind::Image => {
            video::stop_all();
            image::apply(path, display_id, &cfg)?;
            if cfg.lock_screen_mirror {
                lock_screen::mirror(path).ok();
            }
        }
        Kind::Video => {
            // Set a flat backdrop so any gaps look intentional, then layer video windows.
            video::stop_all();
            video::start(path, display_id, &cfg)?;
        }
    }
    Ok(())
}
