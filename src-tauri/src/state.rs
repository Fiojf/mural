//! Global application state. Constructed once in `setup`.

use anyhow::{Context, Result};
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::broadcast;

use crate::config::{self, Config};
use crate::sources::SourceRegistry;
use crate::theme::ThemeRegistry;
use crate::thumbs::ThumbCache;

#[derive(Clone, Debug)]
pub enum AppEvent {
    ListChanged,
    SourceSyncing(String),
    SourceSynced(String),
    SourceError { id: String, error: String },
    ThumbReady(PathBuf),
}

pub struct AppState {
    pub config_path: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub config: RwLock<Config>,
    pub themes: ThemeRegistry,
    pub sources: SourceRegistry,
    pub thumbs: ThumbCache,
    pub events: broadcast::Sender<AppEvent>,
    first_run: bool,
}

impl AppState {
    pub fn initialize(handle: &AppHandle) -> Result<Self> {
        let data_dir = config::data_dir(handle)?;
        let cache_dir = config::cache_dir(handle)?;
        std::fs::create_dir_all(&data_dir).context("create data dir")?;
        std::fs::create_dir_all(&cache_dir).context("create cache dir")?;
        std::fs::create_dir_all(data_dir.join("themes")).ok();
        std::fs::create_dir_all(data_dir.join("fonts")).ok();
        std::fs::create_dir_all(cache_dir.join("thumbs")).ok();
        std::fs::create_dir_all(cache_dir.join("sources/github")).ok();

        let config_path = data_dir.join("config.toml");
        let (cfg, first_run) = config::load_or_default(&config_path)?;

        // Ensure wallpaper folder exists
        if !cfg.folder.exists() {
            std::fs::create_dir_all(&cfg.folder).context("create wallpaper folder")?;
        }

        let themes = ThemeRegistry::load(handle, &data_dir)?;
        let sources = SourceRegistry::new(&cfg, &cache_dir)?;
        let thumbs = ThumbCache::new(cache_dir.join("thumbs"))?;
        let (tx, _) = broadcast::channel(64);

        Ok(Self {
            config_path,
            data_dir,
            cache_dir,
            config: RwLock::new(cfg),
            themes,
            sources,
            thumbs,
            events: tx,
            first_run,
        })
    }

    pub fn is_first_run(&self) -> bool {
        self.first_run
    }

    pub fn save_config(&self) -> Result<()> {
        let cfg = self.config.read().clone();
        config::save(&self.config_path, &cfg)
    }
}

pub type Shared = Arc<AppState>;
