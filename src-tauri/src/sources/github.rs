//! GitHub repository source.
//!
//! Sync strategy: shallow clone via `git2` on first sync; subsequent syncs run
//! `git fetch` + `git reset --hard origin/<ref>`. Skips full fetch when remote
//! HEAD SHA matches the cached one (cheap `ls-remote` check).

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use url::Url;

use crate::config::GithubSource;
use crate::scan::{classify, Kind, WallpaperItem};

#[derive(Debug, Serialize)]
pub struct AddedSource {
    pub id: String,
}

pub fn cache_dir_for(cache_root: &Path, src: &GithubSource) -> PathBuf {
    let key = stable_key(&src.url, src.r#ref.as_deref());
    cache_root.join(key)
}

pub fn label(url: &str) -> String {
    parse_owner_repo(url)
        .map(|(o, r)| format!("{o}/{r}"))
        .unwrap_or_else(|_| url.to_string())
}

pub fn validate_url(url: &str) -> Result<()> {
    let (_, _) = parse_owner_repo(url)?;
    Ok(())
}

/// Normalize user-provided GitHub references into a canonical https URL.
/// Accepts: "owner/repo", "github.com/owner/repo",
/// "https://github.com/owner/repo[.git]", "git@github.com:owner/repo.git".
pub fn normalize_url(input: &str) -> String {
    let s = input.trim();
    if let Some(rest) = s.strip_prefix("git@github.com:") {
        return format!("https://github.com/{}", rest.trim_end_matches(".git"));
    }
    if s.starts_with("http://") || s.starts_with("https://") {
        return s.to_string();
    }
    if let Some(rest) = s.strip_prefix("github.com/") {
        return format!("https://github.com/{rest}");
    }
    // Bare "owner/repo".
    if s.split('/').count() == 2 && !s.contains(' ') {
        return format!("https://github.com/{s}");
    }
    s.to_string()
}

fn parse_owner_repo(url: &str) -> Result<(String, String)> {
    let normalized = normalize_url(url);
    let parsed = Url::parse(&normalized).with_context(|| format!("invalid URL: {url}"))?;
    let host = parsed.host_str().unwrap_or("");
    if !host.contains("github.com") {
        bail!("only github.com URLs are supported");
    }
    let segs: Vec<&str> = parsed
        .path_segments()
        .map(|s| s.collect())
        .unwrap_or_default();
    if segs.len() < 2 || segs[0].is_empty() || segs[1].is_empty() {
        bail!("URL must look like https://github.com/<owner>/<repo>");
    }
    let owner = segs[0].to_string();
    let repo = segs[1].trim_end_matches(".git").to_string();
    Ok((owner, repo))
}

fn stable_key(url: &str, r#ref: Option<&str>) -> String {
    let mut h = Sha256::new();
    h.update(url.as_bytes());
    h.update(b"\0");
    h.update(r#ref.unwrap_or("").as_bytes());
    let bytes = h.finalize();
    hex::encode(&bytes[..16])
}

pub fn make_id(url: &str, r#ref: Option<&str>) -> String {
    format!("github:{}", stable_key(url, r#ref))
}

pub fn count_items(dir: &Path, sub: Option<&str>) -> usize {
    let scan_root = sub
        .map(|s| dir.join(s))
        .unwrap_or_else(|| dir.to_path_buf());
    walkdir::WalkDir::new(&scan_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| classify(e.path()).is_some())
        .count()
}

pub fn list_items(cache_root: &Path, src: &GithubSource) -> Vec<WallpaperItem> {
    let dir = cache_dir_for(cache_root, src);
    let scan_root = src
        .path
        .as_deref()
        .map(|s| dir.join(s))
        .unwrap_or_else(|| dir.clone());
    let label_text = label(&src.url);
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(&scan_root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path().to_path_buf();
        let Some(kind) = classify(&path) else {
            continue;
        };
        let _ = kind; // kept for future use; preserved via Kind below
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
            kind: classify_again(entry.path()),
            source_id: src.id.clone(),
            source_label: label_text.clone(),
            thumb_url: None,
            mtime,
        });
    }
    out
}

fn classify_again(p: &Path) -> Kind {
    classify(p).unwrap_or(Kind::Image)
}

/// Sync the repository. Returns the resolved commit SHA after sync.
pub fn sync(cache_root: &Path, src: &GithubSource) -> Result<String> {
    validate_url(&src.url)?;
    let dir = cache_dir_for(cache_root, src);
    let r#ref = src.r#ref.as_deref().unwrap_or("HEAD");

    let remote_sha = ls_remote_sha(&src.url, r#ref).ok();
    if let Some(rs) = &remote_sha {
        if Some(rs.as_str()) == src.last_sync_sha.as_deref() && dir.exists() {
            tracing::info!("github source {} unchanged ({rs})", src.id);
            return Ok(rs.clone());
        }
    }

    if dir.exists() && dir.join(".git").exists() {
        // Strip any stale lock files left by a crashed/aborted earlier sync.
        for lock in [".git/index.lock", ".git/shallow.lock", ".git/HEAD.lock"] {
            let _ = std::fs::remove_file(dir.join(lock));
        }
        if let Err(e) = fetch_and_reset(&dir, r#ref) {
            tracing::warn!(
                "fetch on {} failed ({e}); nuking cache dir and re-cloning",
                src.id
            );
            std::fs::remove_dir_all(&dir).ok();
            std::fs::create_dir_all(&dir).context("create cache dir")?;
            shallow_clone(&src.url, r#ref, &dir)?;
        }
    } else {
        if dir.exists() {
            std::fs::remove_dir_all(&dir).ok();
        }
        std::fs::create_dir_all(&dir).context("create cache dir")?;
        shallow_clone(&src.url, r#ref, &dir)?;
    }

    let sha = head_sha(&dir).or(remote_sha).unwrap_or_default();
    Ok(sha)
}

fn run_git(args: &[&str], cwd: Option<&Path>) -> Result<std::process::Output> {
    let mut cmd = Command::new("git");
    if let Some(d) = cwd {
        cmd.current_dir(d);
    }
    let out = cmd
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .context("invoke git (Xcode CLT installed?)")?;
    if !out.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(out)
}

fn shallow_clone(url: &str, r#ref: &str, dst: &Path) -> Result<()> {
    let mut args: Vec<&str> = vec!["clone", "--depth", "1"];
    if r#ref != "HEAD" {
        args.push("--branch");
        args.push(r#ref);
    }
    let dst_str = dst.to_str().context("dst not UTF-8")?;
    args.push(url);
    args.push(dst_str);
    run_git(&args, None)?;
    Ok(())
}

fn fetch_and_reset(dir: &Path, r#ref: &str) -> Result<()> {
    let needle = if r#ref == "HEAD" { "HEAD" } else { r#ref };
    run_git(&["fetch", "--depth", "1", "origin", needle], Some(dir))?;
    run_git(&["reset", "--hard", "FETCH_HEAD"], Some(dir))?;
    Ok(())
}

fn head_sha(dir: &Path) -> Option<String> {
    let out = run_git(&["rev-parse", "HEAD"], Some(dir)).ok()?;
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Cheap remote HEAD SHA lookup without full fetch.
fn ls_remote_sha(url: &str, r#ref: &str) -> Result<String> {
    let needle = if r#ref == "HEAD" { "HEAD" } else { r#ref };
    let out = run_git(&["ls-remote", url, needle], None)?;
    let line = String::from_utf8_lossy(&out.stdout);
    let sha = line
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow!("ls-remote returned nothing for {needle}"))?
        .to_string();
    if sha.len() != 40 {
        bail!("unexpected ls-remote output: {line}");
    }
    Ok(sha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_owner_repo() {
        let (o, r) = parse_owner_repo("https://github.com/foo/bar").unwrap();
        assert_eq!(o, "foo");
        assert_eq!(r, "bar");
    }

    #[test]
    fn parses_owner_repo_dot_git() {
        let (_, r) = parse_owner_repo("https://github.com/foo/bar.git").unwrap();
        assert_eq!(r, "bar");
    }

    #[test]
    fn rejects_non_github() {
        assert!(parse_owner_repo("https://gitlab.com/foo/bar").is_err());
    }

    #[test]
    fn id_stable_for_same_inputs() {
        assert_eq!(
            make_id("https://github.com/a/b", None),
            make_id("https://github.com/a/b", None)
        );
    }

    #[test]
    fn id_changes_with_ref() {
        assert_ne!(
            make_id("https://github.com/a/b", None),
            make_id("https://github.com/a/b", Some("dev"))
        );
    }
}
