//! TOML config file living at `~/Library/Application Support/Mural/config.toml`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    Horizontal,
    Grid,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    NameAsc,
    NameDesc,
    DateAdded,
    Recent,
    Random,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Animation {
    Fade,
    Scale,
    SlideDown,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RotateMode {
    Off,
    Interval { minutes: u32 },
    SunriseSunset,
    PerSpace,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Github,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GithubSource {
    pub id: String,
    pub kind: SourceKind,
    pub url: String,
    #[serde(default)]
    pub r#ref: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_sync_interval")]
    pub sync_interval_hours: u32,
    #[serde(default)]
    pub last_sync_iso: Option<String>,
    #[serde(default)]
    pub last_sync_sha: Option<String>,
}

fn default_true() -> bool {
    true
}
fn default_sync_interval() -> u32 {
    24
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub folder: PathBuf,
    pub hotkey: String,
    pub layout: Layout,
    pub sort: Sort,
    pub show_searchbar: bool,
    pub show_filenames: bool,
    pub strip_extension: bool,
    pub per_screen: bool,
    pub per_space: bool,
    pub lock_screen_mirror: bool,
    pub open_animation: Animation,
    pub theme_id: String,
    pub font_id: String,
    pub rotate: RotateMode,
    #[serde(default)]
    pub first_run_done: bool,
    #[serde(default)]
    pub sources: Vec<GithubSource>,
    #[serde(default)]
    pub color_search_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            folder: dirs::home_dir().unwrap_or_default().join("Mural"),
            hotkey: "CmdOrCtrl+Shift+W".to_string(),
            layout: Layout::Horizontal,
            sort: Sort::NameAsc,
            show_searchbar: true,
            show_filenames: true,
            strip_extension: true,
            per_screen: false,
            per_space: false,
            lock_screen_mirror: false,
            open_animation: Animation::Scale,
            theme_id: "catppuccin-mocha".to_string(),
            font_id: "system".to_string(),
            rotate: RotateMode::Off,
            first_run_done: false,
            sources: Vec::new(),
            color_search_enabled: false,
        }
    }
}

pub fn data_dir(_handle: &AppHandle) -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .context("no Application Support dir")?
        .join("Mural"))
}

pub fn cache_dir(_handle: &AppHandle) -> Result<PathBuf> {
    Ok(dirs::cache_dir().context("no Caches dir")?.join("Mural"))
}

pub fn load_or_default(path: &Path) -> Result<(Config, bool)> {
    if path.exists() {
        let s = std::fs::read_to_string(path)
            .with_context(|| format!("read config at {}", path.display()))?;
        let cfg: Config = toml::from_str(&s).context("parse config.toml")?;
        let first_run = !cfg.first_run_done;
        Ok((cfg, first_run))
    } else {
        let cfg = Config::default();
        save(path, &cfg)?;
        Ok((cfg, true))
    }
}

pub fn save(path: &Path, cfg: &Config) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let s = toml::to_string_pretty(cfg).context("serialize config")?;
    std::fs::write(path, s).with_context(|| format!("write config at {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn roundtrip_defaults() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let (cfg1, first) = load_or_default(&path).unwrap();
        assert!(first);
        let (cfg2, first2) = load_or_default(&path).unwrap();
        assert!(first2);
        assert_eq!(cfg1.hotkey, cfg2.hotkey);
        assert_eq!(cfg1.theme_id, cfg2.theme_id);
    }

    #[test]
    fn roundtrip_customized() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let cfg = Config {
            hotkey: "Cmd+Shift+P".into(),
            layout: Layout::Grid,
            first_run_done: true,
            rotate: RotateMode::Interval { minutes: 30 },
            ..Config::default()
        };
        save(&path, &cfg).unwrap();
        let (loaded, first) = load_or_default(&path).unwrap();
        assert!(!first);
        assert_eq!(loaded.hotkey, "Cmd+Shift+P");
        assert_eq!(loaded.layout, Layout::Grid);
        assert!(matches!(
            loaded.rotate,
            RotateMode::Interval { minutes: 30 }
        ));
    }
}
