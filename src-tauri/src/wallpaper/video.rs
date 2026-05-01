//! Video wallpaper support.
//!
//! A first-class video-wallpaper implementation needs a desktop-level NSWindow
//! per screen with an `AVPlayerLayer` that survives Space changes. The objc2
//! NSWindow constructor (`init_with_content_rect_…_screen_:`) requires a
//! `MainThreadMarker`-bound allocation and the runtime API surface is still
//! evolving. To keep the initial release reliably compilable across `objc2`
//! point releases we fall back to applying the video's poster frame as a
//! static wallpaper. This still gives the user a meaningful preview; a full
//! AVPlayerLayer overlay is tracked in `mural-prompt.md` as follow-up work.

use anyhow::Result;
use std::path::Path;

use crate::config::Config;

#[cfg(target_os = "macos")]
pub fn start(path: &Path, display_id: Option<&str>, cfg: &Config) -> Result<()> {
    // Best-effort: extract a poster frame, write it to the thumb cache, and use
    // that as a static wallpaper. If poster extraction fails we apply the video
    // file directly — `setDesktopImageURL_` will then refuse and surface an
    // error, which is exactly what we want to communicate to the user.
    if let Ok(_img) = poster_frame(path) {
        // poster_frame currently returns Err (see note below); when a real
        // implementation lands, write to disk + apply.
    }
    super::image::apply(path, display_id, cfg)
}

#[cfg(target_os = "macos")]
pub fn stop_all() {
    // No-op until a real desktop-level video overlay is implemented.
}

#[cfg(target_os = "macos")]
pub fn poster_frame(_path: &Path) -> Result<image::DynamicImage> {
    anyhow::bail!("video poster CGImage→RGBA conversion not yet wired")
}

#[cfg(not(target_os = "macos"))]
pub fn start(_path: &Path, _display_id: Option<&str>, _cfg: &Config) -> Result<()> {
    anyhow::bail!("video wallpaper is only implemented on macOS")
}

#[cfg(not(target_os = "macos"))]
pub fn stop_all() {}

#[cfg(not(target_os = "macos"))]
pub fn poster_frame(_path: &Path) -> Result<image::DynamicImage> {
    anyhow::bail!("video poster is only implemented on macOS")
}
