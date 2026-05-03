//! Thumbnail cache. Key = blake3(source_id + canonical_path + mtime_ns) → WebP.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct ThumbCache {
    root: PathBuf,
}

impl ThumbCache {
    pub fn new(root: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&root).context("create thumb cache dir")?;
        Ok(Self { root })
    }

    pub fn key(source_id: &str, path: &Path, mtime_ns: u128) -> String {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let s = format!("{source_id}|{}|{mtime_ns}", canonical.display());
        let h = blake3::hash(s.as_bytes());
        h.to_hex().to_string()
    }

    pub fn path_for(&self, key: &str) -> PathBuf {
        self.root.join(format!("{key}.webp"))
    }

    pub fn color_path_for(&self, key: &str) -> PathBuf {
        self.root.join("colors").join(format!("{key}.color"))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Generate a thumbnail for `src`, writing it to the cached path.
    /// Returns the cached file path. No-op if already present.
    pub fn ensure(&self, src: &Path, source_id: &str) -> Result<PathBuf> {
        let mtime = file_mtime_ns(src).unwrap_or(0);
        let key = Self::key(source_id, src, mtime);
        let dst = self.path_for(&key);
        if dst.exists() {
            return Ok(dst);
        }
        generate_thumb(src, &dst)?;
        Ok(dst)
    }

    /// Look up or compute the dominant color for `src`. Cached as a 6-char
    /// hex sidecar next to the thumbnail.
    pub fn ensure_color(&self, src: &Path, source_id: &str) -> Result<[u8; 3]> {
        let mtime = file_mtime_ns(src).unwrap_or(0);
        let key = Self::key(source_id, src, mtime);
        let dst = self.color_path_for(&key);
        if let Ok(s) = std::fs::read_to_string(&dst) {
            if let Some(rgb) = crate::colors::parse_hex(s.trim()) {
                return Ok(rgb);
            }
        }
        let rgb = crate::colors::analyze(src)?;
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&dst, crate::colors::to_hex(rgb))
            .with_context(|| format!("write color cache {}", dst.display()))?;
        Ok(rgb)
    }
}

fn file_mtime_ns(path: &Path) -> Option<u128> {
    let m = std::fs::metadata(path).ok()?;
    let t = m.modified().ok()?;
    let d = t.duration_since(std::time::UNIX_EPOCH).ok()?;
    Some(d.as_nanos())
}

const THUMB_SHORT_EDGE: u32 = 256;

fn generate_thumb(src: &Path, dst: &Path) -> Result<()> {
    let ext = src
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let img = match ext.as_str() {
        "mp4" | "mov" => video_poster_frame(src)?,
        "heic" => decode_heic_via_sips(src)?,
        _ => image::open(src).with_context(|| format!("decode {}", src.display()))?,
    };

    let resized = resize_short_edge(&img, THUMB_SHORT_EDGE);
    write_webp(&resized, dst)?;
    Ok(())
}

fn resize_short_edge(img: &image::DynamicImage, target: u32) -> image::DynamicImage {
    let (w, h) = (img.width(), img.height());
    if w == 0 || h == 0 {
        return img.clone();
    }
    let (nw, nh) = if w < h {
        (target, target * h / w)
    } else {
        (target * w / h, target)
    };
    img.resize(nw, nh, image::imageops::FilterType::Triangle)
}

fn write_webp(img: &image::DynamicImage, dst: &Path) -> Result<()> {
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    // libwebp rejects dimensions outside [1, 16383]; the encoder panics on
    // VP8_ENC_ERROR_BAD_DIMENSION otherwise.
    if w == 0 || h == 0 || w > 16383 || h > 16383 {
        anyhow::bail!("thumb dimensions {w}x{h} out of webp range");
    }
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    // Some pathological inputs still trip libwebp internally; catch the panic
    // so a single bad image can't take down the worker thread.
    let bytes = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let encoder = webp::Encoder::from_rgba(&rgba, w, h);
        encoder.encode(80.0).to_vec()
    }))
    .map_err(|_| anyhow::anyhow!("webp encoder panicked on {w}x{h}"))?;
    std::fs::write(dst, &bytes).with_context(|| format!("write thumb {}", dst.display()))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn decode_heic_via_sips(src: &Path) -> Result<image::DynamicImage> {
    let tmp = std::env::temp_dir().join(format!(
        "mural-heic-{}.png",
        blake3::hash(src.to_string_lossy().as_bytes()).to_hex()
    ));
    let status = std::process::Command::new("sips")
        .args([
            "-s",
            "format",
            "png",
            src.to_str().unwrap_or(""),
            "--out",
            tmp.to_str().unwrap_or(""),
        ])
        .status()
        .context("invoke sips")?;
    if !status.success() {
        anyhow::bail!("sips failed for {}", src.display());
    }
    let img = image::open(&tmp).context("decode sips output")?;
    let _ = std::fs::remove_file(&tmp);
    Ok(img)
}

#[cfg(not(target_os = "macos"))]
fn decode_heic_via_sips(_src: &Path) -> Result<image::DynamicImage> {
    anyhow::bail!("HEIC decode requires macOS")
}

#[cfg(target_os = "macos")]
fn video_poster_frame(src: &Path) -> Result<image::DynamicImage> {
    crate::wallpaper::video::poster_frame(src)
}

#[cfg(not(target_os = "macos"))]
fn video_poster_frame(_src: &Path) -> Result<image::DynamicImage> {
    anyhow::bail!("video poster requires macOS")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn key_stable_for_same_inputs() {
        let p = PathBuf::from("/tmp/foo.png");
        let a = ThumbCache::key("local", &p, 12345);
        let b = ThumbCache::key("local", &p, 12345);
        assert_eq!(a, b);
    }

    #[test]
    fn key_differs_on_mtime_change() {
        let p = PathBuf::from("/tmp/foo.png");
        let a = ThumbCache::key("local", &p, 1);
        let b = ThumbCache::key("local", &p, 2);
        assert_ne!(a, b);
    }

    #[test]
    fn key_differs_across_sources() {
        let p = PathBuf::from("/tmp/foo.png");
        let a = ThumbCache::key("local", &p, 1);
        let b = ThumbCache::key("github:abc", &p, 1);
        assert_ne!(a, b);
    }
}
