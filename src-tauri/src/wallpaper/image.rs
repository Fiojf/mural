//! Set a static image as the desktop wallpaper via `NSWorkspace`.

use anyhow::{Context, Result};
use std::path::Path;

use crate::config::Config;

#[cfg(target_os = "macos")]
pub fn apply(path: &Path, display_id: Option<&str>, cfg: &Config) -> Result<()> {
    use objc2::rc::autoreleasepool;
    use objc2_app_kit::{NSScreen, NSWorkspace};
    use objc2_foundation::{MainThreadMarker, NSDictionary, NSString, NSURL};

    let mtm = MainThreadMarker::new().context("setting wallpaper requires main thread")?;
    let url_str = path
        .to_str()
        .context("path is not valid UTF-8")?
        .to_string();

    autoreleasepool(|_| -> Result<()> {
        let ns_path = NSString::from_str(&url_str);
        let url = NSURL::fileURLWithPath(&ns_path);
        let ws = NSWorkspace::sharedWorkspace();
        let screens = NSScreen::screens(mtm);
        let opts: objc2::rc::Retained<NSDictionary<NSString>> = NSDictionary::new();
        for i in 0..screens.count() {
            let screen = screens.objectAtIndex(i);
            if cfg.per_screen {
                if let Some(id) = display_id {
                    let this_id = screen_uuid(&screen).unwrap_or_default();
                    if this_id != id {
                        continue;
                    }
                }
            }
            unsafe { ws.setDesktopImageURL_forScreen_options_error(&url, &screen, &opts) }
                .map_err(|e| anyhow::anyhow!("setDesktopImageURL failed: {e:?}"))?;
        }
        Ok(())
    })?;

    // NSWorkspace only sets the wallpaper for the active Space on each screen.
    // When per-Space rotation is OFF, the user expects the chosen image to
    // apply to every Space. Drive that via System Events / AppleScript which
    // iterates `every desktop` (one per Space, modern macOS).
    if !cfg.per_space {
        apply_all_spaces(&url_str);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn apply_all_spaces(path: &str) {
    let escaped = path.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        r#"tell application "System Events" to tell every desktop to set picture to "{escaped}""#
    );
    // Fire-and-forget so set_wallpaper returns instantly; AppleScript-via-
    // System-Events takes ~1s per Space and would otherwise block the IPC
    // handler / popover dismissal.
    if let Err(e) = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        tracing::warn!("osascript spawn failed: {e}");
    }
}

#[cfg(target_os = "macos")]
fn screen_uuid(screen: &objc2_app_kit::NSScreen) -> Option<String> {
    use objc2_foundation::NSString;
    let dict = screen.deviceDescription();
    let key = NSString::from_str("NSScreenNumber");
    let obj = dict.objectForKey(&key)?;
    let num: &objc2_foundation::NSNumber = obj.downcast_ref()?;
    Some(format!("{}", num.unsignedIntegerValue()))
}

#[cfg(not(target_os = "macos"))]
pub fn apply(_path: &Path, _display_id: Option<&str>, _cfg: &Config) -> Result<()> {
    anyhow::bail!("wallpaper setting is only implemented on macOS")
}
