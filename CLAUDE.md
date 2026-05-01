# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

All commands run from repo root unless noted.

```sh
# Frontend
npm install                  # one-time, after clone
npm run dev                  # vite dev server (port 1420) — usually invoked via `tauri dev`
npm run build                # production frontend bundle into dist/
npm run typecheck            # tsc --noEmit
npm run format               # prettier --write src/

# Tauri (full app, requires Rust)
npm run tauri dev            # launches the desktop app with HMR
npm run tauri build          # produces unsigned .app + .dmg under src-tauri/target/release/bundle/
npm run tauri build -- --no-bundle   # faster: just compiles, skips dmg/codesign

# Rust (run inside src-tauri/)
cargo check                  # type-check, no bundle
cargo test                   # unit + integration tests
cargo test config::tests::roundtrip_defaults     # single test
cargo test --test scan_integration               # single integration file
cargo clippy --all-targets -- -D warnings
cargo fmt

# Resource regeneration (re-run after editing the generators)
node scripts/gen-themes.mjs   # rewrites src-tauri/resources/themes/builtin/*.toml (46 files)
node scripts/gen-samples.mjs  # rewrites src-tauri/resources/samples/*.png  (5 files)
```

The `tauri.conf.json` `bundle.resources` glob lists `themes/builtin/*` and
`samples/*` — if you add a new bundled-resource directory you must list it
there or `cargo tauri build` fails with "glob pattern path not found."

## Architecture

### Two-process split

Tauri 2 runs a single binary that hosts (a) a Rust backend and (b) one or more
WebView windows running a SolidJS app. Code in `src/` is the frontend; code in
`src-tauri/src/` is the backend.

### Three windows, one bundle

`tauri.conf.json` declares **popover**, **settings**, and **onboarding**
windows. They all load `index.html` and `App.tsx` routes by URL hash
(`#/popover`, `#/settings`, `#/onboarding`). The popover is the only window
configured `decorations:false / transparent / alwaysOnTop`. Cursor-aware
centering and panel-style traits are applied from `src-tauri/src/popover.rs`
via `objc2-app-kit`. The popover starts hidden; toggle is wired through:

```
global hotkey ─► hotkey::install ─► popover::toggle ─► WebviewWindow.show/hide
tray click    ─► tray::install   ─► popover::toggle
```

### Shared state

`AppState` (`src-tauri/src/state.rs`) is built once in the `setup` callback and
stored via `app.manage(Arc::new(state))`. Tauri commands obtain it through
`State<'_, Arc<AppState>>`. It owns:

- `RwLock<Config>` — TOML at `~/Library/Application Support/Mural/config.toml`.
- `ThemeRegistry` — built-in themes loaded from bundled resources +
  user-dropped `~/Library/Application Support/Mural/themes/*.toml`.
- `SourceRegistry` — wallpaper providers; see below.
- `ThumbCache` — keyed by `blake3(source_id + canonical_path + mtime_ns)`.
- `broadcast::Sender<AppEvent>` — internal pub/sub. Frontend listens via
  `mural:wallpaper` Tauri events emitted from `scan.rs` and `sources/mod.rs`.

### Wallpaper sources

`src-tauri/src/sources/mod.rs` is a registry over two source kinds:

- **local** — `~/Mural` folder, watched with `notify` (non-recursive).
  Implemented inline in `scan.rs`.
- **github** — repository configured in `config.sources`, shallow-cloned via
  `git2` into `~/Library/Caches/Mural/sources/github/<sha256>/`. Sync runs on
  app launch + on a per-source interval timer + on demand. A cheap
  `git ls-remote HEAD` check skips full fetches when the SHA is unchanged.

`SourceRegistry::collect_items` merges items from all enabled sources;
duplicate filenames across sources are disambiguated by `source_id` in the
thumb-cache key.

### macOS-specific paths

`src-tauri/src/wallpaper/` calls into AppKit/AVFoundation through the
`objc2-*` crates (versions 0.3.x — feature flags must match the
crate's exact set, e.g. `objc2-av-foundation` has no `AVURLAsset` feature
because it's part of `AVAsset`). Static images use
`NSWorkspace.setDesktopImageURL_forScreen_options_error`. Per-display targeting
diff's against the `NSScreenNumber` device-description key.

`wallpaper/video.rs` currently falls through to static apply; a real
desktop-level NSWindow + AVPlayerLayer overlay is a documented TODO.
`wallpaper/lock_screen.rs` is best-effort (admin `osascript` copy to
`/Library/Caches/com.apple.desktop.admin.png`).

`wallpaper/per_space.rs` and `location.rs` are stubs — install a real
`NSWorkspace.activeSpaceDidChangeNotification` observer and a CoreLocation
delegate respectively.

### IPC surface

All `#[tauri::command]`s live in `src-tauri/src/commands.rs`; each is
explicitly listed in the `invoke_handler!` macro in `lib.rs`. The frontend
mirrors this surface in `src/lib/ipc.ts` — when adding or renaming a command
update **both** files. `setConfig` takes a partial patch (`ConfigPatch`); only
fields you want to change should be present.

### Thumbnails

`thumbs.rs::ensure` is a content-addressed cache: it returns the cache path
synchronously, generating on first call. Backend returns absolute paths via
`thumb_url`; the frontend converts them to webview URLs with
`convertFileSrc` from `@tauri-apps/api/core` (asset protocol scope is
configured in `tauri.conf.json` → `app.security.assetProtocol`).

HEIC decoding shells out to `sips` (macOS-only) to avoid bundling libheif.
Video poster-frame extraction is unimplemented — the cache call returns
`Err` and the UI falls back to a placeholder swatch.

## Conventions

- The `objc2_*` crate versions move quickly. If `cargo check` fails after a
  bump, look for renamed feature flags or moved methods between minor versions
  before changing call sites.
- Built-in themes are generated, not hand-edited — change palettes in
  `scripts/gen-themes.mjs` and re-run.
- Tests in `src-tauri/tests/` use `tempfile`; the integration test
  imports the crate as `mural_lib` (matching the `[lib].name` in `Cargo.toml`).
- Do not add `tauri-plugin-tray` as a separate dep — the `tray-icon` feature
  on the `tauri` crate is the supported entry point in Tauri 2.
- The `[dev-dependencies]` table must come **after** all `[dependencies]`
  entries; otherwise subsequent crates accidentally land in dev-deps.
