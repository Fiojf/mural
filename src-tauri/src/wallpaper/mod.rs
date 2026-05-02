//! macOS wallpaper application + per-Space + lock-screen mirror + video.

pub mod image;
pub mod lock_screen;
pub mod per_space;
pub mod video;

use anyhow::Result;
use std::path::Path;

use crate::commands::DisplayInfo;
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

#[cfg(target_os = "macos")]
pub fn list_displays() -> Result<Vec<DisplayInfo>> {
    use objc2_app_kit::NSScreen;
    use objc2_foundation::{MainThreadMarker, NSString};

    let mtm = MainThreadMarker::new()
        .ok_or_else(|| anyhow::anyhow!("list_displays requires main thread"))?;
    let screens = NSScreen::screens(mtm);
    let mut out = Vec::with_capacity(screens.count());
    for i in 0..screens.count() {
        let screen = screens.objectAtIndex(i);
        let dict = screen.deviceDescription();
        let key = NSString::from_str("NSScreenNumber");
        let id = dict
            .objectForKey(&key)
            .and_then(|obj| {
                obj.downcast_ref::<objc2_foundation::NSNumber>()
                    .map(|n| n.unsignedIntegerValue().to_string())
            })
            .unwrap_or_else(|| format!("screen-{i}"));
        let name = screen.localizedName().to_string();
        out.push(DisplayInfo { id, name });
    }
    Ok(out)
}

#[cfg(not(target_os = "macos"))]
pub fn list_displays() -> Result<Vec<DisplayInfo>> {
    Ok(Vec::new())
}
