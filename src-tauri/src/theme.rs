//! Theme registry: loads built-in themes from bundled resources and user themes
//! from `~/Library/Application Support/Mural/themes/*.toml`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Base {
    Dark,
    Light,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Colors {
    pub bg: String,
    pub surface: String,
    pub text: String,
    pub muted: String,
    pub accent: String,
    pub border: String,
    pub selected_border: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ThemeFile {
    pub name: String,
    pub base: Base,
    pub colors: Colors,
}

#[derive(Debug, Clone, Serialize)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub base: Base,
    pub colors: Colors,
    pub builtin: bool,
}

pub struct ThemeRegistry {
    themes: Vec<Theme>,
}

impl ThemeRegistry {
    pub fn load(handle: &AppHandle, data_dir: &Path) -> Result<Self> {
        let mut themes = Vec::new();
        let resource_root = handle
            .path()
            .resolve("resources/themes/builtin", tauri::path::BaseDirectory::Resource)
            .context("resolve builtin themes dir")?;
        load_dir(&resource_root, true, &mut themes)?;

        let user = data_dir.join("themes");
        if user.exists() {
            load_dir(&user, false, &mut themes).ok();
        }

        themes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(Self { themes })
    }

    pub fn list(&self) -> &[Theme] {
        &self.themes
    }

    pub fn find(&self, id: &str) -> Option<&Theme> {
        self.themes.iter().find(|t| t.id == id)
    }
}

fn load_dir(dir: &Path, builtin: bool, out: &mut Vec<Theme>) -> Result<()> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("theme dir {} unreadable: {e}", dir.display());
            return Ok(());
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }
        match parse_one(&path) {
            Ok(file) => {
                let id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("untitled")
                    .to_string();
                out.push(Theme {
                    id,
                    name: file.name,
                    base: file.base,
                    colors: file.colors,
                    builtin,
                });
            }
            Err(e) => tracing::warn!("skip theme {}: {e:#}", path.display()),
        }
    }
    Ok(())
}

fn parse_one(path: &Path) -> Result<ThemeFile> {
    let s = std::fs::read_to_string(path)?;
    let f: ThemeFile = toml::from_str(&s)?;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_theme() {
        let s = r##"
            name = "Test"
            base = "dark"
            [colors]
            bg = "#000"
            surface = "#111"
            text = "#fff"
            muted = "#888"
            accent = "#0af"
            border = "#222"
            selected_border = "#0af"
        "##;
        let t: ThemeFile = toml::from_str(s).unwrap();
        assert_eq!(t.name, "Test");
        assert!(matches!(t.base, Base::Dark));
        assert_eq!(t.colors.bg, "#000");
    }
}
