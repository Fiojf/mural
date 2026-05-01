//! Menu-bar (tray) icon + menu.

use anyhow::Result;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::popover;

pub fn install(app: &AppHandle) -> Result<()> {
    let open = MenuItem::with_id(app, "open", "Open Mural", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, Some("CmdOrCtrl+,"))?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Mural", true, Some("CmdOrCtrl+Q"))?;
    let menu = Menu::with_items(app, &[&open, &settings, &separator, &quit])?;

    let icon = Image::from_bytes(include_bytes!("../icons/tray.png"))
        .unwrap_or_else(|_| Image::from_bytes(&[]).unwrap_or_else(|_| Image::default()));
    let _ = icon;

    let _tray = TrayIconBuilder::with_id("mural-tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => {
                let _ = popover::show(app);
            }
            "settings" => {
                if let Some(w) = app.get_webview_window("settings") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = popover::toggle(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}
