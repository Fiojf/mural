import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type WallpaperKind = "image" | "video";

export interface WallpaperItem {
  path: string;
  name: string;
  display_name: string;
  kind: WallpaperKind;
  source_id: string;
  source_label: string;
  thumb_url: string | null;
  mtime: number;
  dominant_color: string | null;
}

export type Layout = "horizontal" | "grid" | "vertical";
export type Sort = "name_asc" | "name_desc" | "date_added" | "recent" | "random";
export type AnimationKind = "fade" | "scale" | "slide-down" | "none";
export type RotateMode =
  | { kind: "off" }
  | { kind: "interval"; minutes: number }
  | { kind: "sunrise_sunset" }
  | { kind: "per_space" };

export interface ThemeColors {
  bg: string;
  surface: string;
  text: string;
  muted: string;
  accent: string;
  border: string;
  selected_border: string;
}

export interface Theme {
  id: string;
  name: string;
  base: "dark" | "light";
  colors: ThemeColors;
  builtin: boolean;
}

export interface FontEntry {
  id: string;
  name: string;
  builtin: boolean;
  family: string;
}

export interface SourceEntry {
  id: string;
  kind: "local" | "github";
  label: string;
  url: string | null;
  ref: string | null;
  path: string | null;
  enabled: boolean;
  sync_interval_hours: number;
  last_sync_iso: string | null;
  last_sync_sha: string | null;
  item_count: number;
  status: "ok" | "syncing" | "error";
  error: string | null;
}

export interface Config {
  folder: string;
  hotkey: string;
  layout: Layout;
  sort: Sort;
  show_searchbar: boolean;
  show_filenames: boolean;
  strip_extension: boolean;
  per_screen: boolean;
  per_space: boolean;
  lock_screen_mirror: boolean;
  open_animation: AnimationKind;
  theme_id: string;
  font_id: string;
  rotate: RotateMode;
  first_run_done: boolean;
  color_search_enabled: boolean;
}

export const ipc = {
  getConfig: () => invoke<Config>("get_config"),
  setConfig: (patch: Partial<Config>) => invoke<Config>("set_config", { patch }),
  listThemes: () => invoke<Theme[]>("list_themes"),
  listFonts: () => invoke<FontEntry[]>("list_fonts"),
  listWallpapers: () => invoke<WallpaperItem[]>("list_wallpapers"),
  setWallpaper: (path: string, displayId?: string | null) =>
    invoke<void>("set_wallpaper", { path, displayId: displayId ?? null }),
  openSettings: () => invoke<void>("open_settings"),
  openPopover: () => invoke<void>("open_popover"),
  closePopover: () => invoke<void>("close_popover"),
  revealInFinder: (path: string) => invoke<void>("reveal_in_finder", { path }),
  sourcesList: () => invoke<SourceEntry[]>("sources_list"),
  sourcesAddGithub: (input: {
    url: string;
    ref?: string | null;
    path?: string | null;
    sync_interval_hours: number;
  }) => invoke<SourceEntry>("sources_add_github", { input }),
  sourcesRemove: (id: string) => invoke<void>("sources_remove", { id }),
  sourcesSync: (id: string) => invoke<void>("sources_sync", { id }),
  sourcesSetEnabled: (id: string, enabled: boolean) =>
    invoke<void>("sources_set_enabled", { id, enabled }),
  onboardingComplete: () => invoke<void>("onboarding_complete"),
  requestLocation: () => invoke<{ lat: number; lon: number } | null>("request_location"),
};

export type WallpaperEvent =
  | { type: "list-changed" }
  | { type: "syncing"; source_id: string }
  | { type: "synced"; source_id: string }
  | { type: "sync-error"; source_id: string; error: string }
  | { type: "thumb-ready"; path: string };

export function onWallpaperEvent(cb: (e: WallpaperEvent) => void): Promise<UnlistenFn> {
  return listen<WallpaperEvent>("mural:wallpaper", (e) => cb(e.payload));
}

export function onPopoverDismiss(cb: () => void): Promise<UnlistenFn> {
  return listen("mural:popover-dismiss", () => cb());
}

export function onConfigChanged(cb: (cfg: Config) => void): Promise<UnlistenFn> {
  return listen<Config>("mural:config-changed", (e) => cb(e.payload));
}
