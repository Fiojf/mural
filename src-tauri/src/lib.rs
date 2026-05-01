//! Mural — macOS wallpaper picker.
//!
//! Top-level entry. Wires config, theme, source registry, scan/thumb pipeline,
//! global hotkey, tray, popover/settings windows, and auto-rotate scheduler.

pub mod commands;
pub mod config;
pub mod fonts;
pub mod hotkey;
pub mod location;
pub mod popover;
pub mod rotate;
pub mod samples;
pub mod scan;
pub mod sources;
pub mod state;
pub mod theme;
pub mod thumbs;
pub mod tray;
pub mod wallpaper;

use std::sync::Arc;
use tauri::Manager;
use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with_target(false)
        .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            popover::toggle(app).ok();
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::set_config,
            commands::list_themes,
            commands::list_fonts,
            commands::list_wallpapers,
            commands::set_wallpaper,
            commands::open_settings,
            commands::open_popover,
            commands::close_popover,
            commands::reveal_in_finder,
            commands::sources_list,
            commands::sources_add_github,
            commands::sources_remove,
            commands::sources_sync,
            commands::sources_set_enabled,
            commands::onboarding_complete,
            commands::request_location,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let app_state = Arc::new(state::AppState::initialize(&handle)?);
            app.manage(app_state.clone());

            tray::install(&handle)?;
            hotkey::install(&handle, &app_state)?;
            popover::configure_window(&handle)?;

            // Spawn background subsystems
            let h = handle.clone();
            let s = app_state.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = scan::start(&h, &s).await {
                    tracing::error!("scan start failed: {e:#}");
                }
            });

            let h = handle.clone();
            let s = app_state.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = sources::start(&h, &s).await {
                    tracing::error!("sources start failed: {e:#}");
                }
            });

            let h = handle.clone();
            let s = app_state.clone();
            tauri::async_runtime::spawn(async move {
                rotate::run(&h, &s).await;
            });

            // First-run onboarding gate
            if app_state.is_first_run() {
                if let Some(w) = app.get_webview_window("onboarding") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
