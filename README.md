# Mural

Floating wallpaper picker for macOS 12+, written in Rust + Tauri 2.

Mural lives in the menu bar. Press a hotkey from anywhere to summon a floating
popover, click any wallpaper to apply it, dismiss with `Esc`. Pull wallpapers
from a local folder, from GitHub repositories, or both.

## Highlights

- **Floating popover** centered on the screen with the cursor, with vibrancy/blur
  background, fade+scale animation, theme-aware accent border on the active item.
- **45+ built-in themes** (Catppuccin, Tokyo Night, Gruvbox, Nord, Dracula,
  Rosé Pine, Solarized, Everforest, Kanagawa, One Dark/Light, Material, Ayu,
  Monokai Pro, GitHub) plus user-dropped TOML themes.
- **GitHub sources**: register any public repo of wallpapers; Mural shallow-clones,
  syncs on a schedule, and merges its images into the picker.
- **All formats**: JPG, PNG, HEIC, WebP, GIF, BMP, TIFF, plus MP4/MOV video
  wallpapers via a desktop-level AVPlayer overlay.
- **Per-display, per-Space, lock-screen mirror, auto-rotate** (interval or
  sunrise/sunset).
- **Custom hotkey** (default `⌘⇧W`), launch at login, system fonts + Inter +
  JetBrains Mono.

## Build from source

Requires Rust (stable, ≥1.77), Node.js 18+, and Xcode command line tools.

```sh
git clone https://github.com/Fiojf/mural.git
cd mural
npm install
npm run tauri dev      # development run
npm run tauri build    # produces unsigned .app + .dmg under src-tauri/target/release/bundle/
```

The first build downloads and vendors `libgit2` and `openssl` via `git2`'s
vendored features, so no system dependencies beyond Xcode CLT are required.

## Install

### Homebrew (cask)

```sh
brew install --cask Fiojf/mural/mural
```

### MacPorts

```sh
sudo port install mural
```

### Pre-built `.app`

Download the latest `.dmg` from the [Releases](https://github.com/Fiojf/mural/releases)
page. Mural is **unsigned** (no Apple Developer cert), so the first launch needs
a Gatekeeper bypass:

```sh
xattr -d com.apple.quarantine /Applications/Mural.app
```

Or right-click the app and choose **Open**.

## Configuration

`~/Library/Application Support/Mural/config.toml` is rewritten by the Settings
window; you don't have to edit it by hand. To add a custom theme drop a TOML
file into `~/Library/Application Support/Mural/themes/`:

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

Custom fonts work the same way: drop `.ttf`/`.otf` into
`~/Library/Application Support/Mural/fonts/` and pick them in Settings →
Appearance.

## Architecture

```
src/                   SolidJS + Tailwind frontend (popover, settings, onboarding)
src-tauri/             Rust backend (Tauri 2 + objc2 macOS bindings)
src-tauri/resources/   Bundled themes, fonts, sample wallpapers
packaging/             MacPorts portfile + Homebrew cask
```

See [`mural-prompt.md`](mural-prompt.md) for the full design brief.

## License

GPL-3.0-only. See [`LICENSE`](LICENSE).
