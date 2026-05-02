//! Tauri command surface (IPC).

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::config::{Config, GithubSource, SourceKind};
use crate::popover;
use crate::scan::WallpaperItem;
use crate::sources::{self, SourceEntry};
use crate::state::AppState;
use crate::theme::Theme;
use crate::wallpaper;

type Shared<'a> = State<'a, Arc<AppState>>;

fn err<E: std::fmt::Display>(e: E) -> String {
    format!("{e:#}")
}

#[tauri::command]
pub fn get_config(state: Shared<'_>) -> Config {
    state.config.read().clone()
}

#[derive(Debug, Default, Deserialize)]
pub struct ConfigPatch {
    pub folder: Option<PathBuf>,
    pub hotkey: Option<String>,
    pub layout: Option<crate::config::Layout>,
    pub sort: Option<crate::config::Sort>,
    pub show_searchbar: Option<bool>,
    pub show_filenames: Option<bool>,
    pub strip_extension: Option<bool>,
    pub per_screen: Option<bool>,
    pub per_space: Option<bool>,
    pub lock_screen_mirror: Option<bool>,
    pub open_animation: Option<crate::config::Animation>,
    pub theme_id: Option<String>,
    pub font_id: Option<String>,
    pub rotate: Option<crate::config::RotateMode>,
}

#[tauri::command]
pub fn set_config(patch: ConfigPatch, app: AppHandle, state: Shared<'_>) -> Result<Config, String> {
    let layout_changed;
    {
        let mut cfg = state.config.write();
        if let Some(v) = patch.folder {
            cfg.folder = v;
        }
        if let Some(v) = patch.hotkey {
            cfg.hotkey = v;
        }
        layout_changed = patch.layout.is_some();
        if let Some(v) = patch.layout {
            cfg.layout = v;
        }
        if let Some(v) = patch.sort {
            cfg.sort = v;
        }
        if let Some(v) = patch.show_searchbar {
            cfg.show_searchbar = v;
        }
        if let Some(v) = patch.show_filenames {
            cfg.show_filenames = v;
        }
        if let Some(v) = patch.strip_extension {
            cfg.strip_extension = v;
        }
        if let Some(v) = patch.per_screen {
            cfg.per_screen = v;
        }
        if let Some(v) = patch.per_space {
            cfg.per_space = v;
        }
        if let Some(v) = patch.lock_screen_mirror {
            cfg.lock_screen_mirror = v;
        }
        if let Some(v) = patch.open_animation {
            cfg.open_animation = v;
        }
        if let Some(v) = patch.theme_id {
            cfg.theme_id = v;
        }
        if let Some(v) = patch.font_id {
            cfg.font_id = v;
        }
        if let Some(v) = patch.rotate {
            cfg.rotate = v;
        }
    }
    state.save_config().map_err(err)?;
    let next = state.config.read().clone();
    if layout_changed {
        let _ = popover::resize_for_layout(&app, &next.layout);
    }
    let _ = app.emit("mural:config-changed", &next);
    Ok(next)
}

#[tauri::command]
pub fn list_themes(state: Shared<'_>) -> Vec<Theme> {
    state.themes.list().to_vec()
}

#[tauri::command]
pub fn list_fonts(state: Shared<'_>) -> Result<Vec<crate::fonts::FontEntry>, String> {
    crate::fonts::list(&state.data_dir).map_err(err)
}

#[tauri::command]
pub fn list_wallpapers(state: Shared<'_>) -> Vec<WallpaperItem> {
    let cfg = state.config.read().clone();
    let mut items = state.sources.collect_items(&cfg);

    // Generate (or look up) the cached thumbnail for each item. The absolute
    // path is returned as `thumb_url`; the frontend converts it to a webview
    // URL via Tauri's `convertFileSrc`. Parallelised across a small thread
    // pool — first-run thumb gen on a large folder is otherwise serial and
    // dominates the IPC latency.
    let thumbs = &state.thumbs;
    std::thread::scope(|s| {
        let chunk = (items.len() / 4).max(1);
        let mut handles = Vec::new();
        for slice in items.chunks_mut(chunk) {
            handles.push(s.spawn(move || {
                for item in slice.iter_mut() {
                    if let Ok(p) = thumbs.ensure(&item.path, &item.source_id) {
                        item.thumb_url = Some(p.to_string_lossy().into_owned());
                    }
                }
            }));
        }
        for h in handles {
            let _ = h.join();
        }
    });
    items
}

#[tauri::command]
pub fn set_wallpaper(
    path: PathBuf,
    display_id: Option<String>,
    app: AppHandle,
    state: Shared<'_>,
) -> Result<(), String> {
    // NSWorkspace.setDesktopImageURL must run on the main thread, but we don't
    // want to block the IPC handler waiting for it. Queue it on the main
    // thread via Tauri's runtime — the run_on_main_thread call returns as
    // soon as the closure is enqueued.
    let state_arc: Arc<AppState> = (*state).clone();
    app.run_on_main_thread(move || {
        if let Err(e) = wallpaper::apply(&state_arc, &path, display_id.as_deref()) {
            tracing::error!("set_wallpaper: {e:#}");
        }
    })
    .map_err(err)?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct DisplayInfo {
    pub id: String,
    pub name: String,
}

#[tauri::command]
pub fn list_displays() -> Result<Vec<DisplayInfo>, String> {
    crate::wallpaper::list_displays().map_err(err)
}

#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("settings") {
        w.show().map_err(err)?;
        w.set_focus().map_err(err)?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_popover(app: AppHandle) -> Result<(), String> {
    popover::show(&app).map_err(err)
}

#[tauri::command]
pub fn close_popover(app: AppHandle) -> Result<(), String> {
    popover::hide(&app).map_err(err)
}

#[tauri::command]
pub fn reveal_in_finder(path: PathBuf) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .status()
            .map_err(err)?;
    }
    let _ = path;
    Ok(())
}

#[tauri::command]
pub fn sources_list(state: Shared<'_>) -> Vec<SourceEntry> {
    let cfg = state.config.read().clone();
    state.sources.list(&cfg)
}

#[derive(Debug, Deserialize)]
pub struct GithubInput {
    pub url: String,
    pub r#ref: Option<String>,
    pub path: Option<String>,
    pub sync_interval_hours: u32,
}

#[tauri::command]
pub async fn sources_add_github(
    input: GithubInput,
    app: AppHandle,
    state: Shared<'_>,
) -> Result<SourceEntry, String> {
    let normalized = crate::sources::github::normalize_url(&input.url);
    crate::sources::github::validate_url(&normalized).map_err(err)?;
    let id = crate::sources::github::make_id(&normalized, input.r#ref.as_deref());
    let src = GithubSource {
        id: id.clone(),
        kind: SourceKind::Github,
        url: normalized.clone(),
        r#ref: input.r#ref.clone(),
        path: input.path.clone(),
        enabled: true,
        sync_interval_hours: input.sync_interval_hours.max(1),
        last_sync_iso: None,
        last_sync_sha: None,
    };
    {
        let mut cfg = state.config.write();
        if cfg.sources.iter().any(|s| s.id == id) {
            return Err(format!("source already exists: {id}"));
        }
        cfg.sources.push(src.clone());
    }
    state.save_config().map_err(err)?;

    // Kick off the recurring sync loop (also performs the first sync).
    let state_arc: Arc<AppState> = (*state).clone();
    sources::spawn_loop(app.clone(), state_arc, src.id.clone());

    let cfg = state.config.read().clone();
    let entry = state
        .sources
        .list(&cfg)
        .into_iter()
        .find(|s| s.id == id)
        .ok_or_else(|| "source missing after add".to_string())?;
    Ok(entry)
}

#[tauri::command]
pub fn sources_remove(id: String, app: AppHandle, state: Shared<'_>) -> Result<(), String> {
    {
        let mut cfg = state.config.write();
        cfg.sources.retain(|s| s.id != id);
    }
    state.save_config().map_err(err)?;
    let _ = app.emit(
        "mural:wallpaper",
        serde_json::json!({"type": "list-changed"}),
    );
    Ok(())
}

#[tauri::command]
pub async fn sources_sync(id: String, app: AppHandle, state: Shared<'_>) -> Result<(), String> {
    let src = state
        .config
        .read()
        .sources
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| format!("no such source: {id}"))?;
    let state_arc: Arc<AppState> = (*state).clone();
    sources::sync_one(&app, &state_arc, &src).await.map_err(err)
}

#[tauri::command]
pub fn sources_set_enabled(
    id: String,
    enabled: bool,
    app: AppHandle,
    state: Shared<'_>,
) -> Result<(), String> {
    {
        let mut cfg = state.config.write();
        if let Some(s) = cfg.sources.iter_mut().find(|s| s.id == id) {
            s.enabled = enabled;
        }
    }
    state.save_config().map_err(err)?;
    let _ = app.emit(
        "mural:wallpaper",
        serde_json::json!({"type": "list-changed"}),
    );
    Ok(())
}

#[tauri::command]
pub fn onboarding_complete(app: AppHandle, state: Shared<'_>) -> Result<(), String> {
    {
        let mut cfg = state.config.write();
        cfg.first_run_done = true;
    }
    state.save_config().map_err(err)?;
    let folder = state.config.read().folder.clone();
    crate::samples::seed_from_resources(&app, &folder).ok();
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct Loc {
    pub lat: f64,
    pub lon: f64,
}

#[tauri::command]
pub async fn request_location(_state: Shared<'_>) -> Result<Option<Loc>, String> {
    crate::location::request().await.map_err(err)
}
