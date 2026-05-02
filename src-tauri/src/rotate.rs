//! Auto-rotate scheduler. Reads [`crate::config::RotateMode`] and applies
//! a different wallpaper at the configured cadence.

use rand::seq::SliceRandom;
use std::sync::Arc;
use std::time::Duration;
use tauri::AppHandle;

use crate::config::RotateMode;
use crate::scan::WallpaperItem;
use crate::state::AppState;

pub async fn run(_handle: &AppHandle, state: &Arc<AppState>) {
    loop {
        let mode = state.config.read().rotate.clone();
        match mode {
            RotateMode::Off => {
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
            RotateMode::Interval { minutes } => {
                tokio::time::sleep(Duration::from_secs((minutes.max(1) as u64) * 60)).await;
                let _ = pick_and_apply(state).await;
            }
            RotateMode::SunriseSunset => {
                let next = next_sunrise_or_sunset(state).await;
                tokio::time::sleep(next).await;
                let _ = pick_and_apply(state).await;
            }
            RotateMode::PerSpace => {
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        }
    }
}

async fn pick_and_apply(state: &Arc<AppState>) -> anyhow::Result<()> {
    let cfg = state.config.read().clone();
    let items: Vec<WallpaperItem> = state.sources.collect_items(&cfg);
    if items.is_empty() {
        return Ok(());
    }
    let mut rng = rand::thread_rng();
    let pick = items.choose(&mut rng).cloned().unwrap();
    crate::wallpaper::apply(state, &pick.path, None)
}

async fn next_sunrise_or_sunset(_state: &Arc<AppState>) -> Duration {
    if let Ok(Some(loc)) = crate::location::request().await {
        let now = chrono::Utc::now();
        let date = now.date_naive();
        #[allow(deprecated)]
        let (sr, ss) =
            sunrise::sunrise_sunset(loc.lat, loc.lon, date.year(), date.month(), date.day());
        let sr_ts = chrono::DateTime::from_timestamp(sr, 0).unwrap_or(now);
        let ss_ts = chrono::DateTime::from_timestamp(ss, 0).unwrap_or(now);
        let candidates = [sr_ts, ss_ts]
            .into_iter()
            .filter(|t| *t > now)
            .min()
            .map(|t| (t - now).to_std().unwrap_or(Duration::from_secs(3600)))
            .unwrap_or(Duration::from_secs(60 * 60 * 6));
        return candidates;
    }
    Duration::from_secs(60 * 60)
}

use chrono::Datelike;
