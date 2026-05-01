//! Floating popover window: borderless, centered on the screen containing the
//! cursor. Vibrancy/blur is handled in CSS via `backdrop-filter`; we only set
//! window-level traits here.

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
fn apply_panel_style(_win: &WebviewWindow) {
    // Window-level decoration (borderless, transparent, alwaysOnTop) is handled
    // by tauri.conf.json; vibrancy/blur is CSS `backdrop-filter`. Spaces /
    // collection-behavior tweaks would go here.
}

#[cfg(target_os = "macos")]
fn center_on_cursor_screen(win: &WebviewWindow) {
    use objc2::rc::autoreleasepool;
    use objc2_app_kit::{NSEvent, NSScreen};
    use objc2_foundation::MainThreadMarker;

    let mtm = match MainThreadMarker::new() {
        Some(m) => m,
        None => return,
    };
    autoreleasepool(|_| {
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
