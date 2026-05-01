//! Wallpaper folder scanning + filesystem watching.
//!
//! Scans the user's local folder (non-recursive) and emits `mural:wallpaper`
//! events when files appear or disappear.

use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tauri::{AppHandle, Emitter};

use crate::sources::SOURCE_LOCAL;
use crate::state::{AppEvent, AppState};

const SUPPORTED: &[&str] = &[
    "jpg", "jpeg", "png", "heic", "webp", "gif", "bmp", "tiff", "tif", "mp4", "mov",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Image,
    Video,
}

#[derive(Debug, Clone, Serialize)]
pub struct WallpaperItem {
    pub path: PathBuf,
    pub name: String,
    pub display_name: String,
    pub kind: Kind,
    pub source_id: String,
    pub source_label: String,
    pub thumb_url: Option<String>,
    pub mtime: u64,
}

pub fn classify(path: &Path) -> Option<Kind> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    if !SUPPORTED.iter().any(|s| *s == ext) {
        return None;
    }
    Some(if matches!(ext.as_str(), "mp4" | "mov") {
        Kind::Video
    } else {
        Kind::Image
    })
}

pub fn list_local(folder: &Path) -> Vec<WallpaperItem> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(folder) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("scan {}: {e}", folder.display());
            return out;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(kind) = classify(&path) else { continue };
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();
        let mtime = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        out.push(WallpaperItem {
            display_name: name.clone(),
            name,
            path,
            kind,
            source_id: SOURCE_LOCAL.to_string(),
            source_label: "~/Mural".to_string(),
            thumb_url: None,
            mtime,
        });
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

pub async fn start(handle: &AppHandle, state: &Arc<AppState>) -> Result<()> {
    let folder = state.config.read().folder.clone();
    spawn_watcher(handle.clone(), state.clone(), folder)?;
    Ok(())
}

fn spawn_watcher(handle: AppHandle, state: Arc<AppState>, folder: PathBuf) -> Result<()> {
    use notify::{EventKind, RecursiveMode};
    use notify_debouncer_full::new_debouncer;
    use std::time::Duration;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(8);
    let txc = tx.clone();
    std::thread::spawn(move || {
        let mut debouncer = match new_debouncer(Duration::from_millis(250), None, move |res| {
            if let Ok(events) = res {
                let interesting = events.iter().any(|e: &notify_debouncer_full::DebouncedEvent| {
                    matches!(
                        e.event.kind,
                        EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_)
                    )
                });
                if interesting {
                    let _ = txc.blocking_send(());
                }
            }
        }) {
            Ok(d) => d,
            Err(e) => {
                tracing::error!("notify init failed: {e}");
                return;
            }
        };
        if let Err(e) = debouncer.watch(&folder, RecursiveMode::NonRecursive) {
            tracing::error!("notify watch failed: {e}");
            return;
        }
        std::thread::park();
    });

    tauri::async_runtime::spawn(async move {
        while rx.recv().await.is_some() {
            let _ = state.events.send(AppEvent::ListChanged);
            let _ = handle.emit("mural:wallpaper", serde_json::json!({"type": "list-changed"}));
        }
    });

    Ok(())
}
