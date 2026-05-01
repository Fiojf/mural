# Mural — Build Prompt

Build a macOS 12+ wallpaper picker app called **Mural**, written in Rust. License GPL-3.0. Inspired by FloatPane (https://github.com/floatpane/floatpane — study its source, screenshots, demo videos, and any docs to mirror its floating-popover feel and motion). Distribute as both a `.app` bundle and a MacPorts portfile. Unsigned (no Apple Developer cert).

## Stack

- **Tauri 2** (Rust backend, web frontend) — best aesthetic + theming flexibility.
- Frontend: **SolidJS + Vite + TailwindCSS** (small, fast, snappy animations).
- Rust crates: `tauri`, `image` (thumbnail gen), `notify` (folder watch — even though only one folder, watch for new files), `objc2` + `objc2-app-kit` (set desktop wallpaper via `NSWorkspace.setDesktopImageURL:forScreen:options:error:`), `serde` + `toml` (config), `dirs` (paths), `walkdir`, `tokio`, `global-hotkey` (hotkey registration).
- Config file: **TOML** at `~/Library/Application Support/Mural/config.toml`.
- Thumbnail cache: `~/Library/Caches/Mural/thumbs/` as WebP. Hash original path+mtime for cache key. Lazy-generated, generated in background tokio task, served to frontend via Tauri command.

## Core Behavior

- **Wallpaper folder**: single folder at `~/Mural`. Created on first launch if missing. NOT recursive (top-level only). Watch with `notify` for live add/remove → push event to frontend.
- **Formats supported**: JPG, JPEG, PNG, HEIC, WebP, GIF, BMP, TIFF, MP4, MOV (video wallpaper — use `wallpaper` crate or AVKit-backed approach for video; static fallback if video wallpaper not feasible per-API, document the limitation).
- **Hotkey**: default `Cmd+Shift+W` opens floating popover, configurable in settings. Press again or `Esc` or click outside to dismiss.
- **Menu bar icon**: yes, present in status bar. Click → toggle popover. Right-click → menu (Open, Settings, Quit).
- **Window**: floating popover centered on the screen with the cursor. Frameless, rounded corners, drop shadow, vibrancy/blur background (`NSVisualEffectView` via objc2). NOT pinned to menu bar — appears mid-screen like FloatPane.
- **Open animation** (toggle in settings, default ON): fade + slight scale-up (0.96 → 1.0) over 180ms with cubic-bezier easing. Settings options: fade, scale, slide-down, none.
- **Layout** (settings-toggleable):
  - Default: **horizontal list** with searchbar at top.
  - Alternatives: **grid** (responsive columns), **vertical list**.
  - Toggles: show searchbar (default ON), show filenames under thumbs (default ON), strip file extension from displayed name (default ON).
- **Selected indicator**: lighter-colored border around current wallpaper thumbnail (theme-aware accent).
- **Sort order** (toggleable in settings): name (A→Z, Z→A), date added, recently used, random/shuffle.
- **Per-screen vs all-screens** (settings toggle): "Same wallpaper on all displays" OR "Pick per display" (when per-display, show display selector tabs in popover).
- **Per-Space wallpaper** (settings toggle): if on, set wallpaper only on the active Space; if off, set on all Spaces. Use macOS APIs accordingly.
- **Lock screen** (settings toggle): mirror desktop wallpaper to lock screen via the same `NSWorkspace` mechanism.

## Themes

- Built-in themes: **Catppuccin** (Latte, Frappé, Macchiato, Mocha), **Tokyo Night** (Night, Storm, Day, Moon), **Gruvbox** (Dark Hard/Medium/Soft, Light Hard/Medium/Soft), **Nord** (Polar Night, Snow Storm, Frost, Aurora variants), **Dracula**, **Rosé Pine** (Main, Moon, Dawn), **Solarized** (Dark, Light), **Everforest** (Dark/Light Hard/Medium/Soft), **Kanagawa** (Wave, Dragon, Lotus), **One Dark / One Light**, **Material** (Darker, Palenight, Deep Ocean, Lighter), **Ayu** (Dark, Mirage, Light), **Monokai Pro**, **GitHub** (Dark, Light, Dimmed). Ship as TOML files under `themes/builtin/`.
- **Custom themes**: user drops a `.toml` file in `~/Library/Application Support/Mural/themes/`. Schema:

  ```toml
  name = "My Theme"
  base = "dark"  # or "light"
  [colors]
  bg = "#1e1e2e"
  surface = "#181825"
  text = "#cdd6f4"
  muted = "#a6adc8"
  accent = "#cba6f7"
  border = "#313244"
  selected_border = "#f5c2e7"
  ```

- Theme picker in settings shows previews. Live-apply (no restart).

## Fonts

- Default: system (SF Pro).
- Bundled selectable: **Inter** (UI), **JetBrains Mono** (mono). Plus system option.
- Custom font: user drops `.ttf`/`.otf` in `~/Library/Application Support/Mural/fonts/`, appears in font dropdown.

## Settings UI

Dedicated settings window (separate from popover). Sections:

1. **General** — wallpaper folder path (read-only display + reveal in Finder), launch at login, hotkey rebind.
2. **Display** — per-screen mode toggle, per-Space toggle, lock-screen mirror toggle.
3. **Layout** — horizontal/grid/vertical, searchbar on/off, filenames on/off, strip extension on/off, sort order.
4. **Appearance** — theme picker w/ live preview, font picker, open animation picker.
5. **Auto-rotate** — off / interval (5min / 15min / 30min / 1h / 6h / 24h / custom) / sunrise-sunset / per-Space rotation. Uses macOS location services (with permission prompt) for sunrise-sunset.
6. **About** — version, license, repo link.

## First Launch

Onboarding flow: welcome → pick/confirm wallpaper folder (default `~/Mural`) → pick theme → set hotkey → done. Drops a few sample wallpapers into the folder.

## Performance

- Virtualize the thumb list (only render visible).
- Generate thumbs concurrently (Rayon or Tokio) on first scan.
- Debounce searchbar input 80ms.
- Memoize theme CSS variables; swap via `:root` data-theme attribute.
- Pre-warm thumb cache on app launch in background.

## Build & Distribution

- `cargo tauri build` → `.app` in `dist/`.
- MacPorts portfile in `packaging/macports/Portfile` (build from source via cargo).
- Homebrew cask in `packaging/homebrew/mural.rb` (optional bonus).
- README with build instructions, screenshots, theme gallery, GPL-3.0 LICENSE file, CHANGELOG.

## Code Quality

- Workspace layout: `src-tauri/` (Rust), `src/` (SolidJS).
- `clippy` clean, `cargo fmt`, `prettier` on JS.
- Unit tests for config parsing, theme loading, thumbnail cache keying.
- Integration test that scans a fixture folder and validates emitted wallpaper list.

## Deliverables

1. Full source tree.
2. `cargo tauri dev` runs.
3. `cargo tauri build` produces signable-but-unsigned `.app`.
4. README with screenshots (placeholders OK).
5. All built-in themes as TOML files.
6. Onboarding works on fresh `~/Library/Application Support/Mural/` state.

## Research Step

Study FloatPane's repo (https://github.com/floatpane/floatpane), README, demo media, and issues before starting. Mirror its popover summon-anywhere feel, smooth motion, and minimalist chrome. Where FloatPane uses platform-specific APIs you can't see, infer from macOS docs (NSPanel, NSVisualEffectView, NSWorkspace).

## Build Order

Lay out workspace → config + theme loading → wallpaper scan + thumb cache → popover UI → settings window → auto-rotate → packaging.
