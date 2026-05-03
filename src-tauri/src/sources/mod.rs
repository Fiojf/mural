//! Wallpaper sources: a registry of pluggable image providers.
//!
//! The local `~/Mural` folder is always present as the implicit `local` source.
//! Additional GitHub-repo sources are configured by the user.

pub mod github;

use anyhow::Result;
use parking_lot::RwLock;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

use crate::config::{Config, GithubSource};
use crate::scan::WallpaperItem;
use crate::state::{AppEvent, AppState};

pub const SOURCE_LOCAL: &str = "local";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceStatus {
    Ok,
    Syncing,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceEntry {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub url: Option<String>,
    pub r#ref: Option<String>,
    pub path: Option<String>,
    pub enabled: bool,
    pub sync_interval_hours: u32,
    pub last_sync_iso: Option<String>,
    pub last_sync_sha: Option<String>,
    pub item_count: usize,
    pub status: SourceStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GithubState {
    pub status: SourceStatus,
    pub error: Option<String>,
}

pub struct SourceRegistry {
    pub cache_root: PathBuf,
    pub statuses: RwLock<std::collections::HashMap<String, GithubState>>,
    /// Per-source serializer: prevents two sync_one calls (recurring loop +
    /// manual "Sync now" click) from racing on the same `.git` dir and
    /// corrupting the lock files.
    pub sync_locks:
        parking_lot::Mutex<std::collections::HashMap<String, Arc<parking_lot::Mutex<()>>>>,
}

impl SourceRegistry {
    pub fn new(_cfg: &Config, cache_dir: &std::path::Path) -> Result<Self> {
        let cache_root = cache_dir.join("sources/github");
        std::fs::create_dir_all(&cache_root).ok();
        Ok(Self {
            cache_root,
            statuses: RwLock::new(std::collections::HashMap::new()),
            sync_locks: parking_lot::Mutex::new(std::collections::HashMap::new()),
        })
    }

    pub fn sync_lock_for(&self, id: &str) -> Arc<parking_lot::Mutex<()>> {
        self.sync_locks
            .lock()
            .entry(id.to_string())
            .or_insert_with(|| Arc::new(parking_lot::Mutex::new(())))
            .clone()
    }

    pub fn list(&self, cfg: &Config) -> Vec<SourceEntry> {
        let mut out = Vec::new();
        let local_items = crate::scan::list_local(&cfg.folder);
        out.push(SourceEntry {
            id: SOURCE_LOCAL.into(),
            kind: "local".into(),
            label: "~/Mural".into(),
            url: None,
            r#ref: None,
            path: None,
            enabled: true,
            sync_interval_hours: 0,
            last_sync_iso: None,
            last_sync_sha: None,
            item_count: local_items.len(),
            status: SourceStatus::Ok,
            error: None,
        });
        let statuses = self.statuses.read();
        for src in &cfg.sources {
            let dir = github::cache_dir_for(&self.cache_root, src);
            let count = github::count_items(&dir, src.path.as_deref());
            let st = statuses.get(&src.id).cloned().unwrap_or(GithubState {
                status: SourceStatus::Ok,
                error: None,
            });
            out.push(SourceEntry {
                id: src.id.clone(),
                kind: "github".into(),
                label: github::label(&src.url),
                url: Some(src.url.clone()),
                r#ref: src.r#ref.clone(),
                path: src.path.clone(),
                enabled: src.enabled,
                sync_interval_hours: src.sync_interval_hours,
                last_sync_iso: src.last_sync_iso.clone(),
                last_sync_sha: src.last_sync_sha.clone(),
                item_count: count,
                status: st.status,
                error: st.error,
            });
        }
        out
    }

    pub fn collect_items(&self, cfg: &Config) -> Vec<WallpaperItem> {
        let mut out = crate::scan::list_local(&cfg.folder);
        for src in cfg.sources.iter().filter(|s| s.enabled) {
            let items = github::list_items(&self.cache_root, src);
            out.extend(items);
        }
        out
    }
}

/// Spawn background sync loops for all configured GitHub sources.
pub async fn start(handle: &AppHandle, state: &Arc<AppState>) -> Result<()> {
    let ids: Vec<String> = state
        .config
        .read()
        .sources
        .iter()
        .map(|s| s.id.clone())
        .collect();
    for id in ids {
        spawn_loop(handle.clone(), state.clone(), id);
    }
    Ok(())
}

/// Spawn a recurring sync loop for the source with `id`. Re-reads config each
/// iteration so toggling `enabled`, changing `sync_interval_hours`, or removing
/// the source takes effect on the running loop. Exits when the source is gone.
pub fn spawn_loop(handle: AppHandle, state: Arc<AppState>, id: String) {
    tauri::async_runtime::spawn(async move {
        loop {
            let snapshot = state
                .config
                .read()
                .sources
                .iter()
                .find(|s| s.id == id)
                .cloned();
            let Some(src) = snapshot else {
                // Source removed from config — terminate loop.
                return;
            };
            if !src.enabled {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                continue;
            }
            let _ = sync_one(&handle, &state, &src).await;
            let secs = (src.sync_interval_hours.max(1) as u64) * 3600;
            tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
        }
    });
}

pub async fn sync_one(handle: &AppHandle, state: &Arc<AppState>, src: &GithubSource) -> Result<()> {
    state.statuses_set(&src.id, SourceStatus::Syncing, None);
    let _ = handle.emit(
        "mural:wallpaper",
        serde_json::json!({"type": "syncing", "source_id": src.id}),
    );
    let cache_root = state.sources.cache_root.clone();
    let src_clone = src.clone();
    let lock = state.sources.sync_lock_for(&src.id);
    let result = tokio::task::spawn_blocking(move || {
        let _guard = lock.lock();
        github::sync(&cache_root, &src_clone)
    })
    .await?;
    match result {
        Ok(new_sha) => {
            // Update last_sync metadata in config
            {
                let mut cfg = state.config.write();
                if let Some(s) = cfg.sources.iter_mut().find(|s| s.id == src.id) {
                    s.last_sync_iso = Some(chrono::Utc::now().to_rfc3339());
                    s.last_sync_sha = Some(new_sha.clone());
                }
            }
            state.save_config().ok();
            state.statuses_set(&src.id, SourceStatus::Ok, None);
            let _ = state.events.send(AppEvent::SourceSynced(src.id.clone()));
            let _ = handle.emit(
                "mural:wallpaper",
                serde_json::json!({"type": "synced", "source_id": src.id}),
            );
            let _ = handle.emit(
                "mural:wallpaper",
                serde_json::json!({"type": "list-changed"}),
            );
            Ok(())
        }
        Err(e) => {
            let msg = format!("{e:#}");
            state.statuses_set(&src.id, SourceStatus::Error, Some(msg.clone()));
            let _ = state.events.send(AppEvent::SourceError {
                id: src.id.clone(),
                error: msg.clone(),
            });
            let _ = handle.emit(
                "mural:wallpaper",
                serde_json::json!({"type": "sync-error", "source_id": src.id, "error": msg}),
            );
            Err(e)
        }
    }
}

impl AppState {
    pub fn statuses_set(&self, id: &str, status: SourceStatus, error: Option<String>) {
        self.sources
            .statuses
            .write()
            .insert(id.to_string(), GithubState { status, error });
    }
}
