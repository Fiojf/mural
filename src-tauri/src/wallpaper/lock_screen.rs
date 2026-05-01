//! Best-effort lock-screen wallpaper mirror.
//!
//! macOS does not expose a public API for the lock-screen wallpaper. As a
//! best-effort approach we copy the chosen image to
//! `/Library/Caches/com.apple.desktop.admin.png` (used internally by macOS for
//! the login window background on some versions). This requires admin and is
//! triggered through `osascript`'s privilege prompt; failures are non-fatal.

use anyhow::{Context, Result};
use std::path::Path;

const TARGET: &str = "/Library/Caches/com.apple.desktop.admin.png";

pub fn mirror(src: &Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let src_str = src.to_str().context("path not UTF-8")?;
        let script = format!(
            r#"do shell script "cp '{src}' '{dst}'" with administrator privileges"#,
            src = src_str.replace('\'', "'\\''"),
            dst = TARGET
        );
        let status = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .status()
            .context("invoke osascript")?;
        if !status.success() {
            anyhow::bail!("osascript exited non-zero (user denied?)");
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = src;
        anyhow::bail!("lock-screen mirror only implemented on macOS")
    }
}
