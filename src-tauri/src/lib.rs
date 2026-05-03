//! Mural — macOS wallpaper picker.
//!
//! Top-level entry. Wires config, theme, source registry, scan/thumb pipeline,
//! global hotkey, tray, popover/settings windows, and auto-rotate scheduler.

pub mod colors;
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
            commands::list_displays,
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
            // Hide dock icon — Mural is a menu-bar agent. The bundled .app
            // also sets LSUIElement=true via Info.plist for production
            // launches; this call covers `cargo run` / `tauri dev`.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

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

            // Pre-warm the thumbnail (and color) cache in the background so
            // the first popover open is instant after a cold start.
            let s = app_state.clone();
            tauri::async_runtime::spawn(async move {
                tokio::task::spawn_blocking(move || prewarm_caches(&s))
                    .await
                    .ok();
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

/// Walk every wallpaper across every enabled source and ensure its thumbnail
/// (and dominant color, when the toggle is on) is on disk. Cheap on warm
/// caches — the per-item ensure_* calls return immediately when the sidecar
/// already exists.
fn prewarm_caches(state: &Arc<state::AppState>) {
    let cfg = state.config.read().clone();
    let items = state.sources.collect_items(&cfg);
    let thumbs = &state.thumbs;
    let color_search = cfg.color_search_enabled;
    std::thread::scope(|s| {
        let chunk = (items.len() / 4).max(1);
        for slice in items.chunks(chunk) {
            s.spawn(move || {
                for item in slice {
                    let _ = thumbs.ensure(&item.path, &item.source_id);
                    if color_search {
                        let _ = thumbs.ensure_color(&item.path, &item.source_id);
                    }
                }
            });
        }
    });
    tracing::info!("prewarm: {} items processed", items.len());
}
