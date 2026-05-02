//! Floating popover window: borderless, centered on the screen containing the
//! cursor. Vibrancy/blur is handled in CSS via `backdrop-filter`; we only set
//! window-level traits here.

use anyhow::Result;
use tauri::{AppHandle, LogicalSize, Manager, WebviewWindow};

use crate::config::Layout;

const LABEL: &str = "popover";

pub fn size_for(layout: &Layout) -> (f64, f64) {
    match layout {
        Layout::Horizontal => (820.0, 240.0),
        Layout::Grid => (720.0, 480.0),
        Layout::Vertical => (380.0, 560.0),
    }
}

pub fn resize_for_layout(app: &AppHandle, layout: &Layout) -> Result<()> {
    let Some(win) = app.get_webview_window(LABEL) else {
        return Ok(());
    };
    let (w, h) = size_for(layout);
    win.set_size(LogicalSize::new(w, h))?;
    let _ = win.center();
    Ok(())
}

pub fn configure_window(app: &AppHandle) -> Result<()> {
    let Some(win) = app.get_webview_window(LABEL) else {
        return Ok(());
    };
    apply_panel_style(&win);
    if let Some(state) = app.try_state::<std::sync::Arc<crate::state::AppState>>() {
        let layout = state.config.read().layout.clone();
        let (w, h) = size_for(&layout);
        let _ = win.set_size(LogicalSize::new(w, h));
    }
    let win_clone = win.clone();
    let had_focus = std::sync::atomic::AtomicBool::new(false);
    let had_focus = std::sync::Arc::new(had_focus);
    win.on_window_event(move |event| {
        use std::sync::atomic::Ordering;
        if let tauri::WindowEvent::Focused(focused) = event {
            if *focused {
                had_focus.store(true, Ordering::SeqCst);
            } else if had_focus.swap(false, Ordering::SeqCst) {
                let _ = win_clone.hide();
            }
        }
    });
    Ok(())
}

pub fn toggle(app: &AppHandle) -> Result<()> {
    let Some(win) = app.get_webview_window(LABEL) else {
        return Ok(());
    };
    let visible = win.is_visible().unwrap_or(false);
    tracing::debug!("popover toggle: visible={visible}");
    if visible {
        win.hide()?;
    } else {
        win.show()?;
        let _ = win.center();
        let _ = win.set_focus();
    }
    Ok(())
}

pub fn show(app: &AppHandle) -> Result<()> {
    let Some(win) = app.get_webview_window(LABEL) else {
        return Ok(());
    };
    win.show()?;
    let _ = win.center();
    let _ = win.set_focus();
    Ok(())
}

pub fn hide(app: &AppHandle) -> Result<()> {
    let Some(win) = app.get_webview_window(LABEL) else {
        return Ok(());
    };
    win.hide()?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn apply_panel_style(win: &WebviewWindow) {
    use objc2::rc::Retained;
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};

    let Ok(handle) = win.ns_window() else { return };
    if handle.is_null() {
        return;
    }
    unsafe {
        let ns_window: Retained<NSWindow> = Retained::retain(handle.cast()).unwrap();
        let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::Stationary;
        ns_window.setCollectionBehavior(behavior);
    }
}

#[cfg(not(target_os = "macos"))]
fn apply_panel_style(_win: &WebviewWindow) {}
