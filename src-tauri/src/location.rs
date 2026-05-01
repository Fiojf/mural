//! CoreLocation-backed location lookup for sunrise/sunset rotation.
//!
//! macOS apps need a location-usage description in `Info.plist` and must
//! request authorization at runtime. We do a one-shot lookup; failures are
//! reported back to the UI so the user can re-prompt or pick a different
//! rotate mode.

use anyhow::Result;

use crate::commands::Loc;

#[cfg(target_os = "macos")]
pub async fn request() -> Result<Option<Loc>> {
    // Full async CoreLocation integration requires running a delegate against
    // the main run loop. To keep the initial scaffold compiling without
    // pulling in additional FFI plumbing we surface a clear "not yet wired"
    // error; UI can fall back to manual coordinates or a fixed interval.
    //
    // TODO: implement via objc2_core_location::CLLocationManager with a
    // small delegate that pushes the first fix into a oneshot channel.
    anyhow::bail!("location lookup is not yet wired; pick a fixed interval for now")
}

#[cfg(not(target_os = "macos"))]
pub async fn request() -> Result<Option<Loc>> {
    anyhow::bail!("location lookup is only implemented on macOS")
}
