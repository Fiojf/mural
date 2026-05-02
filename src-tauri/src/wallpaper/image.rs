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
    let url_str = path.to_str().context("path is not valid UTF-8")?.to_string();

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
            unsafe {
                ws.setDesktopImageURL_forScreen_options_error(&url, &screen, &opts)
            }
            .map_err(|e| anyhow::anyhow!("setDesktopImageURL failed: {e:?}"))?;
        }
        Ok(())
    })
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
