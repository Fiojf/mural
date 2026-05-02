# Contributing to Mural

Thanks for considering a contribution. Mural is a small project and PRs are welcome.

## Quick start

```sh
git clone https://github.com/Fiojf/mural.git
cd mural
npm install
npm run tauri dev
```

Requires Rust ≥ 1.77, Node.js 18+, Xcode Command Line Tools.

## Before you open a PR

Run these locally — CI runs them too:

```sh
# Rust
cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test

# Frontend
cd ..
npm run typecheck
npm run build
npx prettier --check src/
```

## Style

- **Rust**: `cargo fmt`. Avoid `unwrap`/`panic!` outside tests. Prefer `anyhow::Result`
  in command/IO layers, `thiserror` for typed errors.
- **TypeScript**: prettier defaults. Solid components: function-style, named exports.
- **Commits**: short imperative subject (~50 chars), wrapped body if needed. Reference
  issues with `Fixes #N`.

## IPC changes

Tauri commands live in `src-tauri/src/commands.rs` and are listed in the
`invoke_handler!` macro in `src-tauri/src/lib.rs`. The frontend mirror is
`src/lib/ipc.ts`. **Update all three** when adding/renaming a command.

## macOS-specific code

Anything that calls `objc2_*` lives under `src-tauri/src/wallpaper/` or
`src-tauri/src/popover.rs`. The `objc2-*` crates move quickly; if `cargo check`
breaks after a bump, look for renamed feature flags or moved methods between
minor versions before changing call sites.

## Adding a built-in theme

Edit `scripts/gen-themes.mjs` and re-run:

```sh
node scripts/gen-themes.mjs
```

This regenerates `src-tauri/resources/themes/builtin/*.toml`. Don't hand-edit
those files.

## Releases

Tag pushes (`git tag v1.2.3 && git push origin v1.2.3`) trigger
`.github/workflows/release.yml` which builds DMGs for both architectures and
publishes a GitHub release.
