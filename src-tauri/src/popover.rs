//! Floating popover window: borderless NSPanel-style, centered on the screen
//! containing the cursor, with `NSVisualEffectView` blur.

use anyhow::Result;
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

const LABEL: &str = "popover";

pub fn configure_window(app: &AppHandle) -> Result<()> {
    let Some(win) = app.get_webview_window(LABEL) else {
        return Ok(());
    };
    apply_panel_style(&win);
    Ok(())
}

pub fn toggle(app: &AppHandle) -> Result<()> {
    let Some(win) = app.get_webview_window(LABEL) else {
        return Ok(());
    };
    let visible = win.is_visible().unwrap_or(false);
    if visible {
        let _ = app.emit("mural:popover-dismiss", ());
        win.hide()?;
    } else {
        center_on_cursor_screen(&win);
        win.show()?;
        win.set_focus()?;
    }
    Ok(())
}

pub fn show(app: &AppHandle) -> Result<()> {
    let Some(win) = app.get_webview_window(LABEL) else {
        return Ok(());
    };
    center_on_cursor_screen(&win);
    win.show()?;
    win.set_focus()?;
    Ok(())
}

pub fn hide(app: &AppHandle) -> Result<()> {
    let Some(win) = app.get_webview_window(LABEL) else {
        return Ok(());
    };
    let _ = app.emit("mural:popover-dismiss", ());
    win.hide()?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn apply_panel_style(win: &WebviewWindow) {
    use objc2::msg_send;
    use objc2::rc::autoreleasepool;
    use objc2_app_kit::{
        NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState,
        NSVisualEffectView, NSWindowCollectionBehavior,
    };
    use objc2_foundation::{MainThreadMarker, NSRect};

    let Ok(ns_window) = win.ns_window() else { return };
    autoreleasepool(|_| unsafe {
        let mtm = MainThreadMarker::new().unwrap();
        let win_ptr = ns_window as *mut objc2_app_kit::NSWindow;
        let win_ref: &objc2_app_kit::NSWindow = &*win_ptr;
        win_ref.setOpaque(false);
        win_ref.setHasShadow(true);
        win_ref.setMovableByWindowBackground(false);
        win_ref.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary
                | NSWindowCollectionBehavior::Stationary,
        );
        // Above floating windows but below screensaver:
        win_ref.setLevel(8);

        // Insert NSVisualEffectView under the webview content.
        let frame: NSRect = win_ref.frame();
        let bounds = NSRect::new(
            objc2_foundation::NSPoint::new(0.0, 0.0),
            frame.size,
        );
        let effect = NSVisualEffectView::initWithFrame(NSVisualEffectView::alloc(mtm), bounds);
        effect.setMaterial(NSVisualEffectMaterial::HUDWindow);
        effect.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
        effect.setState(NSVisualEffectState::Active);
        let _: () = msg_send![&effect, setWantsLayer: true];
        if let Some(content) = win_ref.contentView() {
            // Insert as the first subview so the webview renders on top.
            content.addSubview_positioned_relativeTo(&effect, objc2_app_kit::NSWindowOrderingMode::Below, None);
        }
    });
}

#[cfg(target_os = "macos")]
fn center_on_cursor_screen(win: &WebviewWindow) {
    use objc2::rc::autoreleasepool;
    use objc2_app_kit::{NSEvent, NSScreen};
    use objc2_foundation::MainThreadMarker;

    autoreleasepool(|_| unsafe {
        let mtm = MainThreadMarker::new().unwrap();
        let mouse = NSEvent::mouseLocation();
        let screens = NSScreen::screens(mtm);
        for i in 0..screens.count() {
            let s = screens.objectAtIndex(i);
            let f = s.frame();
            if mouse.x >= f.origin.x
                && mouse.x <= f.origin.x + f.size.width
                && mouse.y >= f.origin.y
                && mouse.y <= f.origin.y + f.size.height
            {
                let size = win
                    .outer_size()
                    .ok()
                    .map(|p| (p.width as f64, p.height as f64))
                    .unwrap_or((720.0, 360.0));
                let x = f.origin.x + (f.size.width - size.0) / 2.0;
                let y = f.origin.y + (f.size.height - size.1) / 2.0;
                let _ = win.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));
                break;
            }
        }
    });
}

#[cfg(not(target_os = "macos"))]
fn apply_panel_style(_win: &WebviewWindow) {}

#[cfg(not(target_os = "macos"))]
fn center_on_cursor_screen(_win: &WebviewWindow) {}
