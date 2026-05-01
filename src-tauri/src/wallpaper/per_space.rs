//! Per-Space wallpaper handling.
//!
//! macOS does not expose Space identifiers publicly, but applications can
//! observe `NSWorkspace.activeSpaceDidChangeNotification` and re-apply the
//! wallpaper when the active Space changes. The actual mapping of
//! "active Space → wallpaper" is held in [`crate::config::Config`] when
//! `per_space = true`.

#[cfg(target_os = "macos")]
pub fn install_observer(_app: &tauri::AppHandle) {
    // TODO: register an Objective-C observer on
    // NSWorkspace.activeSpaceDidChangeNotification and dispatch into the Tauri
    // event loop. Left as a follow-up to keep the initial scaffold compiling.
}

#[cfg(not(target_os = "macos"))]
pub fn install_observer(_app: &tauri::AppHandle) {}
