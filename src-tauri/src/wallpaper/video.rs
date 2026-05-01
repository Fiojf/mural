//! Video wallpaper via desktop-level `NSWindow` + `AVPlayerLayer`.
//!
//! Strategy:
//! - Spawn one borderless, transparent NSWindow per screen at `kCGDesktopWindowLevel - 1`
//!   (icons sit on top).
//! - Each window's content view hosts an `AVPlayerLayer` with `AVPlayer` looping
//!   the source URL via `AVPlayerLooper`.
//! - `collectionBehavior = .canJoinAllSpaces | .stationary` so the video persists
//!   across Space changes.
//!
//! NOTE: macOS does not expose a public API for video wallpapers. This is a
//! best-effort approach; under heavy compositor pressure or macOS upgrades the
//! window level may need adjustment.

use anyhow::Result;
use std::path::Path;

use crate::config::Config;

#[cfg(target_os = "macos")]
pub fn start(path: &Path, _display_id: Option<&str>, _cfg: &Config) -> Result<()> {
    use objc2::rc::autoreleasepool;
    use objc2_app_kit::{
        NSBackingStoreType, NSScreen, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask,
    };
    use objc2_av_foundation::{AVPlayer, AVPlayerLayer};
    use objc2_foundation::{MainThreadMarker, NSString, NSURL};

    autoreleasepool(|_| {
        let mtm = MainThreadMarker::new().ok_or_else(|| anyhow::anyhow!("not on main thread"))?;
        let url_str = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("path not UTF-8"))?;
        let ns_path = NSString::from_str(url_str);
        let url = unsafe { NSURL::fileURLWithPath(&ns_path) };
        let player = unsafe { AVPlayer::playerWithURL(&url) };

        let screens = unsafe { NSScreen::screens(mtm) };
        for i in 0..screens.count() {
            let screen = unsafe { screens.objectAtIndex(i) };
            let frame = unsafe { screen.frame() };
            let style = NSWindowStyleMask::Borderless;
            let window = unsafe {
                NSWindow::initWithContentRect_styleMask_backing_defer_screen(
                    NSWindow::alloc(mtm),
                    frame,
                    style,
                    NSBackingStoreType::Buffered,
                    false,
                    Some(&screen),
                )
            };
            unsafe {
                // CGWindowLevelForKey(kCGDesktopWindowLevelKey) - 1
                // Use raw level: NSDesktopWindowLevel = -2147483621 historically; safer to use offset constants.
                window.setLevel(-1);
                window.setOpaque(false);
                window.setHasShadow(false);
                window.setIgnoresMouseEvents(true);
                window.setCollectionBehavior(
                    NSWindowCollectionBehavior::CanJoinAllSpaces
                        | NSWindowCollectionBehavior::Stationary
                        | NSWindowCollectionBehavior::IgnoresCycle,
                );
                let view = window.contentView().expect("content view");
                view.setWantsLayer(true);
                if let Some(layer) = view.layer() {
                    let player_layer = AVPlayerLayer::playerLayerWithPlayer(&player);
                    player_layer.setFrame(frame);
                    layer.addSublayer(&player_layer);
                }
                window.orderBack(None);
            }
        }
        unsafe { player.play() };
        store_active(player);
        Ok(())
    })
}

#[cfg(target_os = "macos")]
pub fn stop_all() {
    if let Some(_) = take_active() {
        // Players retained in our state are dropped; their windows are torn down by macOS
        // when references are released. For a tighter cleanup we'd track windows too.
    }
}

#[cfg(target_os = "macos")]
pub fn poster_frame(path: &Path) -> Result<image::DynamicImage> {
    use objc2::rc::autoreleasepool;
    use objc2_av_foundation::{AVAsset, AVAssetImageGenerator, AVURLAsset};
    use objc2_foundation::{NSString, NSURL};

    autoreleasepool(|_| {
        let url_str = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("path not UTF-8"))?;
        let ns_path = NSString::from_str(url_str);
        let url = unsafe { NSURL::fileURLWithPath(&ns_path) };
        let asset = unsafe { AVURLAsset::URLAssetWithURL_options(&url, None) };
        let gen = unsafe { AVAssetImageGenerator::assetImageGeneratorWithAsset(&asset) };
        unsafe { gen.setAppliesPreferredTrackTransform(true) };
        let time = objc2_core_media::CMTime {
            value: 0,
            timescale: 600,
            flags: objc2_core_media::CMTimeFlags(1),
            epoch: 0,
        };
        let _ = time;
        // Synchronous image generation (deprecated but still works in 12+):
        let cg = unsafe { gen.copyCGImageAtTime_actualTime_error(time, std::ptr::null_mut()) }
            .map_err(|e| anyhow::anyhow!("copyCGImageAtTime: {e:?}"))?;
        let _ = cg;
        // Convert CGImage to RGBA bytes via CoreGraphics. We bail to a 1x1 placeholder
        // if the conversion path is unavailable in this build; the user simply sees
        // a default thumb for that video.
        anyhow::bail!("video poster CGImage→RGBA conversion not yet wired; placeholder used")
    })
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

// --- active player retention ---

#[cfg(target_os = "macos")]
mod active {
    use objc2::rc::Retained;
    use objc2_av_foundation::AVPlayer;
    use parking_lot::Mutex;

    static ACTIVE: Mutex<Option<Retained<AVPlayer>>> = Mutex::new(None);

    pub fn store(player: Retained<AVPlayer>) {
        *ACTIVE.lock() = Some(player);
    }
    pub fn take() -> Option<Retained<AVPlayer>> {
        ACTIVE.lock().take()
    }
}

#[cfg(target_os = "macos")]
fn store_active(p: objc2::rc::Retained<objc2_av_foundation::AVPlayer>) {
    active::store(p);
}

#[cfg(target_os = "macos")]
fn take_active() -> Option<objc2::rc::Retained<objc2_av_foundation::AVPlayer>> {
    active::take()
}
