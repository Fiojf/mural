<div align="center">

# Mural

**Wallpapers, one keystroke away.**

A floating wallpaper picker for macOS. Lives in your menu bar, summons with `⌘⇧W`,
applies wallpapers instantly. Built with Rust + Tauri 2 + SolidJS.

[![CI](https://github.com/Fiojf/mural/actions/workflows/ci.yml/badge.svg)](https://github.com/Fiojf/mural/actions/workflows/ci.yml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Latest Release](https://img.shields.io/github/v/release/Fiojf/mural?include_prereleases&label=release)](https://github.com/Fiojf/mural/releases/latest)
[![macOS 12+](https://img.shields.io/badge/macOS-12%2B-lightgrey)](https://github.com/Fiojf/mural/releases/latest)

[Download](https://github.com/Fiojf/mural/releases/latest) · [Website](https://fiojf.github.io/mural/) · [Changelog](CHANGELOG.md) · [Contributing](CONTRIBUTING.md)

</div>

---

## Features

- **Floating popover** centered on the cursor's screen, dismissed with `Esc` or focus-loss.
- **Three layouts** — horizontal scroller, grid, vertical list. Popover auto-resizes per layout.
- **46 built-in themes** — Catppuccin, Tokyo Night, Gruvbox, Nord, Dracula, Rosé Pine,
  Solarized, Everforest, Kanagawa, One Dark/Light, Material, Ayu, Monokai Pro, GitHub.
  Drop your own `.toml` in the themes folder.
- **GitHub sources** — point Mural at any public repo of images. Shallow clone via libgit2,
  per-source sync interval, cheap `ls-remote` polling.
- **Per-display targeting** — multi-monitor users get individual wallpapers per screen.
- **Auto-rotate** — interval-based or sunrise/sunset (CoreLocation).
- **All Spaces** — popover follows you across macOS Spaces.
- **Custom fonts** — system, Inter, JetBrains Mono, or drop in TTF/OTF.

## Install

### Pre-built DMG

Download the latest `.dmg` from [Releases](https://github.com/Fiojf/mural/releases/latest).
Mural is **unsigned** (no Apple Developer cert). First launch needs a Gatekeeper bypass:

```sh
xattr -dr com.apple.quarantine /Applications/Mural.app
```

Or right-click the app and choose **Open**.

### Homebrew (via cask file)

Mural is not yet published to a Homebrew tap, but the cask is in this repo. Install it directly:

```sh
brew install --cask https://raw.githubusercontent.com/Fiojf/mural/main/packaging/homebrew/mural.rb
```

### MacPorts

A `Portfile` is in [`packaging/macports/`](packaging/macports/Portfile) for future submission
to the upstream ports tree. Not yet available via `port install` — use the DMG for now.

### Build from source

Requires Rust ≥ 1.77, Node.js 18+, and Xcode Command Line Tools.

```sh
git clone https://github.com/Fiojf/mural.git
cd mural
npm install
npm run tauri dev          # dev with HMR
npm run tauri build        # release build → src-tauri/target/release/bundle/
```

`libgit2` and `openssl` are vendored via `git2`'s features — no system deps needed.

## Usage

| Action                         | Default shortcut |
| ------------------------------ | ---------------- |
| Toggle popover                 | `⌘⇧W`            |
| Dismiss popover                | `Esc`            |
| Open Settings                  | gear icon / tray |
| Apply wallpaper                | click thumbnail  |

Drop wallpapers into `~/Mural`. Mural watches the folder and updates the picker live.

## Configuration

Config lives at `~/Library/Application Support/Mural/config.toml`. The Settings window
rewrites it; manual edits aren't needed.

### Custom theme

Drop a `.toml` into `~/Library/Application Support/Mural/themes/`:

```toml
name = "My Theme"
base = "dark"   # or "light"

[colors]
bg              = "#1e1e2e"
surface         = "#181825"
text            = "#cdd6f4"
muted           = "#a6adc8"
accent          = "#cba6f7"
border          = "#313244"
selected_border = "#f5c2e7"
```

### Custom font

Drop a `.ttf` or `.otf` into `~/Library/Application Support/Mural/fonts/` and pick
it from Settings → Appearance.

### GitHub source

Settings → Sources → **Add GitHub source**. Or directly in `config.toml`:

```toml
[[sources]]
kind = "github"
url  = "https://github.com/owner/repo"
ref  = "main"
path = "wallpapers"      # optional subdir
sync_interval_hours = 24
```

## Project layout

```
src/                   SolidJS + Tailwind frontend (popover, settings, onboarding)
src-tauri/             Rust backend (Tauri 2 + objc2 macOS bindings)
src-tauri/resources/   Bundled themes + sample wallpapers
scripts/               Theme + sample generators (Node)
docs/                  GitHub Pages site
.github/workflows/     CI + release pipelines
```

## Contributing

Issues and PRs welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Disclaimers

This project is in early beta and lacks some features, if you have an idea of a feature suggest it. Read CONTRIBUTING.md
for further information.
This project has been made with ai 
assistance - take of that what you will.

## License

[GPL-3.0-only](LICENSE).
