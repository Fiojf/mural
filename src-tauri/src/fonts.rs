//! Font discovery: built-in (system, Inter, JetBrains Mono) + user-dropped.

use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct FontEntry {
    pub id: String,
    pub name: String,
    pub builtin: bool,
    /// CSS font-family value.
    pub family: String,
}

pub fn list(data_dir: &Path) -> Result<Vec<FontEntry>> {
    let mut fonts = vec![
        FontEntry {
            id: "system".into(),
            name: "System (SF Pro)".into(),
            builtin: true,
            family: "-apple-system, 'SF Pro Text', system-ui, sans-serif".into(),
        },
        FontEntry {
            id: "inter".into(),
            name: "Inter".into(),
            builtin: true,
            family: "'Inter', -apple-system, system-ui, sans-serif".into(),
        },
        FontEntry {
            id: "jetbrains-mono".into(),
            name: "JetBrains Mono".into(),
            builtin: true,
            family: "'JetBrains Mono', ui-monospace, monospace".into(),
        },
    ];

    let user_dir = data_dir.join("fonts");
    if let Ok(entries) = std::fs::read_dir(&user_dir) {
        for e in entries.flatten() {
            let p = e.path();
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
            if !matches!(ext.to_ascii_lowercase().as_str(), "ttf" | "otf") {
                continue;
            }
            let stem = p
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("font")
                .to_string();
            fonts.push(FontEntry {
                id: format!("user:{stem}"),
                name: stem.clone(),
                builtin: false,
                family: format!("'{stem}', system-ui, sans-serif"),
            });
        }
    }

    Ok(fonts)
}
