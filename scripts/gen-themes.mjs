#!/usr/bin/env node
// Generates built-in theme TOML files into src-tauri/resources/themes/builtin/.
// Palettes adapted from each theme's official color reference.

import { writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const out = resolve(here, "../src-tauri/resources/themes/builtin");
mkdirSync(out, { recursive: true });

const themes = [
  // Catppuccin
  ["catppuccin-latte", "Catppuccin Latte", "light", { bg: "#eff1f5", surface: "#e6e9ef", text: "#4c4f69", muted: "#6c6f85", accent: "#8839ef", border: "#bcc0cc", selected_border: "#ea76cb" }],
  ["catppuccin-frappe", "Catppuccin Frappé", "dark", { bg: "#303446", surface: "#292c3c", text: "#c6d0f5", muted: "#a5adce", accent: "#ca9ee6", border: "#414559", selected_border: "#f4b8e4" }],
  ["catppuccin-macchiato", "Catppuccin Macchiato", "dark", { bg: "#24273a", surface: "#1e2030", text: "#cad3f5", muted: "#a5adcb", accent: "#c6a0f6", border: "#363a4f", selected_border: "#f5bde6" }],
  ["catppuccin-mocha", "Catppuccin Mocha", "dark", { bg: "#1e1e2e", surface: "#181825", text: "#cdd6f4", muted: "#a6adc8", accent: "#cba6f7", border: "#313244", selected_border: "#f5c2e7" }],

  // Tokyo Night
  ["tokyo-night", "Tokyo Night", "dark", { bg: "#1a1b26", surface: "#16161e", text: "#c0caf5", muted: "#9aa5ce", accent: "#7aa2f7", border: "#2a2b3d", selected_border: "#bb9af7" }],
  ["tokyo-night-storm", "Tokyo Night Storm", "dark", { bg: "#24283b", surface: "#1f2335", text: "#c0caf5", muted: "#a9b1d6", accent: "#7aa2f7", border: "#414868", selected_border: "#bb9af7" }],
  ["tokyo-night-day", "Tokyo Night Day", "light", { bg: "#e1e2e7", surface: "#d0d5e3", text: "#3760bf", muted: "#6172b0", accent: "#2e7de9", border: "#a8aecb", selected_border: "#9854f1" }],
  ["tokyo-night-moon", "Tokyo Night Moon", "dark", { bg: "#222436", surface: "#1e2030", text: "#c8d3f5", muted: "#a9b8e8", accent: "#82aaff", border: "#3b4261", selected_border: "#c099ff" }],

  // Gruvbox
  ["gruvbox-dark-hard", "Gruvbox Dark Hard", "dark", { bg: "#1d2021", surface: "#282828", text: "#ebdbb2", muted: "#a89984", accent: "#fabd2f", border: "#3c3836", selected_border: "#fb4934" }],
  ["gruvbox-dark-medium", "Gruvbox Dark Medium", "dark", { bg: "#282828", surface: "#3c3836", text: "#ebdbb2", muted: "#a89984", accent: "#fabd2f", border: "#504945", selected_border: "#fb4934" }],
  ["gruvbox-dark-soft", "Gruvbox Dark Soft", "dark", { bg: "#32302f", surface: "#3c3836", text: "#ebdbb2", muted: "#a89984", accent: "#fabd2f", border: "#504945", selected_border: "#fb4934" }],
  ["gruvbox-light-hard", "Gruvbox Light Hard", "light", { bg: "#f9f5d7", surface: "#fbf1c7", text: "#3c3836", muted: "#7c6f64", accent: "#b57614", border: "#d5c4a1", selected_border: "#9d0006" }],
  ["gruvbox-light-medium", "Gruvbox Light Medium", "light", { bg: "#fbf1c7", surface: "#ebdbb2", text: "#3c3836", muted: "#7c6f64", accent: "#b57614", border: "#d5c4a1", selected_border: "#9d0006" }],
  ["gruvbox-light-soft", "Gruvbox Light Soft", "light", { bg: "#f2e5bc", surface: "#ebdbb2", text: "#3c3836", muted: "#7c6f64", accent: "#b57614", border: "#d5c4a1", selected_border: "#9d0006" }],

  // Nord
  ["nord", "Nord", "dark", { bg: "#2e3440", surface: "#3b4252", text: "#eceff4", muted: "#d8dee9", accent: "#88c0d0", border: "#434c5e", selected_border: "#88c0d0" }],
  ["nord-polar-night", "Nord Polar Night", "dark", { bg: "#2e3440", surface: "#3b4252", text: "#eceff4", muted: "#d8dee9", accent: "#5e81ac", border: "#434c5e", selected_border: "#88c0d0" }],
  ["nord-snow-storm", "Nord Snow Storm", "light", { bg: "#eceff4", surface: "#e5e9f0", text: "#2e3440", muted: "#4c566a", accent: "#5e81ac", border: "#d8dee9", selected_border: "#88c0d0" }],
  ["nord-frost", "Nord Frost", "dark", { bg: "#2e3440", surface: "#3b4252", text: "#eceff4", muted: "#d8dee9", accent: "#8fbcbb", border: "#434c5e", selected_border: "#81a1c1" }],

  // Dracula
  ["dracula", "Dracula", "dark", { bg: "#282a36", surface: "#21222c", text: "#f8f8f2", muted: "#6272a4", accent: "#bd93f9", border: "#44475a", selected_border: "#ff79c6" }],

  // Rosé Pine
  ["rose-pine", "Rosé Pine", "dark", { bg: "#191724", surface: "#1f1d2e", text: "#e0def4", muted: "#908caa", accent: "#c4a7e7", border: "#26233a", selected_border: "#eb6f92" }],
  ["rose-pine-moon", "Rosé Pine Moon", "dark", { bg: "#232136", surface: "#2a273f", text: "#e0def4", muted: "#908caa", accent: "#c4a7e7", border: "#393552", selected_border: "#eb6f92" }],
  ["rose-pine-dawn", "Rosé Pine Dawn", "light", { bg: "#faf4ed", surface: "#fffaf3", text: "#575279", muted: "#797593", accent: "#907aa9", border: "#dfdad9", selected_border: "#b4637a" }],

  // Solarized
  ["solarized-dark", "Solarized Dark", "dark", { bg: "#002b36", surface: "#073642", text: "#fdf6e3", muted: "#93a1a1", accent: "#268bd2", border: "#586e75", selected_border: "#d33682" }],
  ["solarized-light", "Solarized Light", "light", { bg: "#fdf6e3", surface: "#eee8d5", text: "#586e75", muted: "#93a1a1", accent: "#268bd2", border: "#93a1a1", selected_border: "#d33682" }],

  // Everforest
  ["everforest-dark-hard", "Everforest Dark Hard", "dark", { bg: "#272e33", surface: "#2e383c", text: "#d3c6aa", muted: "#9da9a0", accent: "#a7c080", border: "#414b50", selected_border: "#e69875" }],
  ["everforest-dark-medium", "Everforest Dark Medium", "dark", { bg: "#2d353b", surface: "#343f44", text: "#d3c6aa", muted: "#9da9a0", accent: "#a7c080", border: "#475258", selected_border: "#e69875" }],
  ["everforest-dark-soft", "Everforest Dark Soft", "dark", { bg: "#333c43", surface: "#3a464c", text: "#d3c6aa", muted: "#9da9a0", accent: "#a7c080", border: "#4d5960", selected_border: "#e69875" }],
  ["everforest-light-hard", "Everforest Light Hard", "light", { bg: "#fffbef", surface: "#f8f5e4", text: "#5c6a72", muted: "#829181", accent: "#8da101", border: "#e0dcc7", selected_border: "#f57d26" }],
  ["everforest-light-medium", "Everforest Light Medium", "light", { bg: "#fdf6e3", surface: "#f4f0d9", text: "#5c6a72", muted: "#829181", accent: "#8da101", border: "#dcd9bf", selected_border: "#f57d26" }],
  ["everforest-light-soft", "Everforest Light Soft", "light", { bg: "#f3ead3", surface: "#ede4c5", text: "#5c6a72", muted: "#829181", accent: "#8da101", border: "#d8d2b3", selected_border: "#f57d26" }],

  // Kanagawa
  ["kanagawa-wave", "Kanagawa Wave", "dark", { bg: "#1f1f28", surface: "#16161d", text: "#dcd7ba", muted: "#727169", accent: "#7e9cd8", border: "#2a2a37", selected_border: "#e6c384" }],
  ["kanagawa-dragon", "Kanagawa Dragon", "dark", { bg: "#181616", surface: "#0d0c0c", text: "#c5c9c5", muted: "#737c73", accent: "#8ba4b0", border: "#282727", selected_border: "#c4b28a" }],
  ["kanagawa-lotus", "Kanagawa Lotus", "light", { bg: "#f2ecbc", surface: "#ede4c2", text: "#545464", muted: "#8a8980", accent: "#4d699b", border: "#dcd7a6", selected_border: "#cc6d00" }],

  // One Dark / Light
  ["one-dark", "One Dark", "dark", { bg: "#282c34", surface: "#21252b", text: "#abb2bf", muted: "#5c6370", accent: "#61afef", border: "#3e4451", selected_border: "#c678dd" }],
  ["one-light", "One Light", "light", { bg: "#fafafa", surface: "#f0f0f1", text: "#383a42", muted: "#a0a1a7", accent: "#4078f2", border: "#dcdfe4", selected_border: "#a626a4" }],

  // Material
  ["material-darker", "Material Darker", "dark", { bg: "#212121", surface: "#1a1a1a", text: "#eeffff", muted: "#545454", accent: "#82aaff", border: "#2a2a2a", selected_border: "#c792ea" }],
  ["material-palenight", "Material Palenight", "dark", { bg: "#292d3e", surface: "#202331", text: "#a6accd", muted: "#676e95", accent: "#82aaff", border: "#3a3f58", selected_border: "#c792ea" }],
  ["material-deep-ocean", "Material Deep Ocean", "dark", { bg: "#0f111a", surface: "#090b10", text: "#8f93a2", muted: "#464b5d", accent: "#82aaff", border: "#1a1c25", selected_border: "#c792ea" }],
  ["material-lighter", "Material Lighter", "light", { bg: "#fafafa", surface: "#f5f5f5", text: "#90a4ae", muted: "#cfd8dc", accent: "#6182b8", border: "#e0e0e0", selected_border: "#945eb8" }],

  // Ayu
  ["ayu-dark", "Ayu Dark", "dark", { bg: "#0b0e14", surface: "#0d1017", text: "#bfbdb6", muted: "#565b66", accent: "#39bae6", border: "#1c1f24", selected_border: "#ffb454" }],
  ["ayu-mirage", "Ayu Mirage", "dark", { bg: "#1f2430", surface: "#1a1f29", text: "#cbccc6", muted: "#707a8c", accent: "#73d0ff", border: "#2d3340", selected_border: "#ffd580" }],
  ["ayu-light", "Ayu Light", "light", { bg: "#fcfcfc", surface: "#f3f4f5", text: "#5c6166", muted: "#8a9199", accent: "#399ee6", border: "#e7e8e9", selected_border: "#f2ae49" }],

  // Monokai Pro
  ["monokai-pro", "Monokai Pro", "dark", { bg: "#2d2a2e", surface: "#221f22", text: "#fcfcfa", muted: "#727072", accent: "#ab9df2", border: "#403e41", selected_border: "#ff6188" }],

  // GitHub
  ["github-dark", "GitHub Dark", "dark", { bg: "#0d1117", surface: "#161b22", text: "#c9d1d9", muted: "#8b949e", accent: "#58a6ff", border: "#30363d", selected_border: "#f778ba" }],
  ["github-light", "GitHub Light", "light", { bg: "#ffffff", surface: "#f6f8fa", text: "#24292f", muted: "#57606a", accent: "#0969da", border: "#d0d7de", selected_border: "#bf3989" }],
  ["github-dimmed", "GitHub Dimmed", "dark", { bg: "#22272e", surface: "#2d333b", text: "#adbac7", muted: "#768390", accent: "#539bf5", border: "#444c56", selected_border: "#e275ad" }],
];

for (const [id, name, base, c] of themes) {
  const toml = `name = "${name}"
base = "${base}"

[colors]
bg = "${c.bg}"
surface = "${c.surface}"
text = "${c.text}"
muted = "${c.muted}"
accent = "${c.accent}"
border = "${c.border}"
selected_border = "${c.selected_border}"
`;
  writeFileSync(resolve(out, `${id}.toml`), toml);
}

console.log(`wrote ${themes.length} themes to ${out}`);
