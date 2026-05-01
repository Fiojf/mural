//! Set a static image as the desktop wallpaper via `NSWorkspace`.

use anyhow::{Context, Result};
use std::path::Path;

use crate::config::Config;

#[cfg(target_os = "macos")]
pub fn apply(path: &Path, display_id: Option<&str>, cfg: &Config) -> Result<()> {
    use objc2::rc::autoreleasepool;
    use objc2::ClassType;
    use objc2_app_kit::{NSScreen, NSWorkspace};
    use objc2_foundation::{NSDictionary, NSString, NSURL};

    autoreleasepool(|_| {
        let url_str = path
            .to_str()
            .context("path is not valid UTF-8")?
            .to_string();
        let ns_url_path = NSString::from_str(&url_str);
        let url = unsafe { NSURL::fileURLWithPath(&ns_url_path) };
        let ws = unsafe { NSWorkspace::sharedWorkspace() };
        let screens = unsafe { NSScreen::screens(objc2_foundation::MainThreadMarker::new().unwrap()) };
        let count = screens.count();
        let opts = unsafe { NSDictionary::dictionary() };
        for i in 0..count {
            let screen = unsafe { screens.objectAtIndex(i) };
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
                    .map_err(|e| anyhow::anyhow!("setDesktopImageURL failed: {e:?}"))?;
            }
        }
        Ok::<_, anyhow::Error>(())
    })
}

#[cfg(target_os = "macos")]
fn screen_uuid(screen: &objc2_app_kit::NSScreen) -> Option<String> {
    use objc2_foundation::NSString;
    unsafe {
        let dict = screen.deviceDescription();
        let key = NSString::from_str("NSScreenNumber");
        let num: Option<&objc2_foundation::NSNumber> = dict.objectForKey(&key).map(|o| o.downcast_ref().unwrap());
        num.map(|n| format!("{}", n.unsignedIntegerValue()))
    }
}

#[cfg(not(target_os = "macos"))]
pub fn apply(_path: &Path, _display_id: Option<&str>, _cfg: &Config) -> Result<()> {
    anyhow::bail!("wallpaper setting is only implemented on macOS")
}
