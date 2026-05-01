import { ipc, type Theme } from "./ipc";

export function applyTheme(theme: Theme): void {
  const r = document.documentElement;
  r.dataset.theme = theme.id;
  r.dataset.base = theme.base;
  const c = theme.colors;
  r.style.setProperty("--color-bg", c.bg);
  r.style.setProperty("--color-surface", c.surface);
  r.style.setProperty("--color-text", c.text);
  r.style.setProperty("--color-muted", c.muted);
  r.style.setProperty("--color-accent", c.accent);
  r.style.setProperty("--color-border", c.border);
  r.style.setProperty("--color-selected-border", c.selected_border);
}

export function applyFont(family: string): void {
  document.documentElement.style.setProperty("--font-ui", family);
}

export async function applyThemeFromBackend(): Promise<void> {
  try {
    const cfg = await ipc.getConfig();
    const themes = await ipc.listThemes();
    const t = themes.find((x) => x.id === cfg.theme_id) ?? themes[0];
    if (t) applyTheme(t);
    const fonts = await ipc.listFonts();
    const f = fonts.find((x) => x.id === cfg.font_id);
    if (f) applyFont(f.family);
  } catch (e) {
    console.error("theme bootstrap failed", e);
  }
}
