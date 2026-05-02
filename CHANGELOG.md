# Changelog

All notable changes to Mural are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-beta.1] - 2026-05-02

### Added
- Layout-aware popover sizing (horizontal / grid / vertical).
- Settings button in popover header.
- `list_displays` IPC command (per-screen tabs in popover).
- All-Spaces visibility via `NSWindowCollectionBehavior`.
- GitHub URL normalization (accepts `owner/repo`, `git@…`, etc.).
- Live config broadcast — Settings changes propagate to popover/onboarding without reload.
- Inter and JetBrains Mono fonts loaded via Google Fonts.

### Fixed
- Hotkey double-toggle (filtered to `HotKeyState::Pressed`).
- `setDesktopImageURL_…` unsafe wrapping (objc2 0.3.x).
- Popover hides on focus loss only after first focus event (no spurious dismiss).
- Recurring GitHub source sync loop now re-reads config each tick — toggling
  `enabled`, changing `sync_interval_hours`, or removing a source takes effect
  on the running loop.
- Sample wallpapers now seed from bundled resources on onboarding finalization.
- Popover background renders solidly (dropped fragile `backdrop-filter` combo).
- Themed `<select>` and `<input>` elements.
- ThumbList no longer relies on stale virtualizer scrollEl ref.

### Removed
- Dead `seed_if_empty` / `bundled_samples` paths.
- Dangling `head_ref` reference in `sources/github.rs`.

## [0.1.0] - 2026-05-01

### Added
- Initial release.
- Floating popover, global hotkey (default `⌘⇧W`), menu-bar tray icon.
- Local `~/Mural` folder source plus arbitrary GitHub repository sources.
- 46 built-in themes (Catppuccin, Tokyo Night, Gruvbox, Nord, Dracula,
  Rosé Pine, Solarized, Everforest, Kanagawa, One Dark/Light, Material,
  Ayu, Monokai Pro, GitHub variants) and user-dropped TOML themes.
- Image (JPG/PNG/HEIC/WebP/GIF/BMP/TIFF) and video (MP4/MOV) wallpapers.
- Per-display, per-Space, and best-effort lock-screen mirror.
- Auto-rotate on interval or sunrise/sunset (location-permission gated).
- Settings window with live theme preview, custom font support, animation picker.
- Onboarding flow on first run; bundled CC0 sample wallpapers.

[Unreleased]: https://github.com/Fiojf/mural/compare/v0.1.0-beta.1...HEAD
[0.1.0-beta.1]: https://github.com/Fiojf/mural/releases/tag/v0.1.0-beta.1
[0.1.0]: https://github.com/Fiojf/mural/releases/tag/v0.1.0
