import { For, Show, createEffect, createMemo, onCleanup, onMount } from "solid-js";
import { Searchbar } from "./Searchbar";
import { ThumbList } from "./ThumbList";
import { ThumbGrid } from "./ThumbGrid";
import { DisplayTabs } from "./DisplayTabs";
import {
  activeDisplay,
  activeSource,
  config,
  refetchWallpapers,
  search,
  setActiveSource,
  wallpapers,
} from "../lib/store";
import { ipc, onWallpaperEvent } from "../lib/ipc";
import { stripExtension } from "../lib/format";

export function Popover() {
  let unlistenList: (() => void) | undefined;

  onMount(async () => {
    unlistenList = await onWallpaperEvent((e) => {
      if (e.type === "list-changed" || e.type === "synced") void refetchWallpapers();
    });

    const onKey = (ev: KeyboardEvent) => {
      if (ev.key === "Escape") void ipc.closePopover();
    };
    window.addEventListener("keydown", onKey);
    onCleanup(() => window.removeEventListener("keydown", onKey));
  });

  onCleanup(() => {
    unlistenList?.();
  });

  const sources = createMemo(() => {
    const items = wallpapers() ?? [];
    const seen = new Map<string, string>();
    for (const it of items) seen.set(it.source_id, it.source_label);
    return Array.from(seen, ([id, label]) => ({ id, label }));
  });

  const filtered = createMemo(() => {
    const items = wallpapers() ?? [];
    const cfg = config();
    const q = search().toLowerCase().trim();
    const src = activeSource();
    const disp = activeDisplay();
    let list = items;
    if (src) list = list.filter((x) => x.source_id === src);
    if (q) list = list.filter((x) => x.display_name.toLowerCase().includes(q));
    if (cfg) {
      const cmp = (a: (typeof items)[number], b: (typeof items)[number]) => {
        switch (cfg.sort) {
          case "name_asc":
            return a.display_name.localeCompare(b.display_name);
          case "name_desc":
            return b.display_name.localeCompare(a.display_name);
          case "date_added":
            return b.mtime - a.mtime;
          case "recent":
            return 0;
          case "random":
            return Math.random() - 0.5;
        }
      };
      list = [...list].sort(cmp);
    }
    void disp;
    return list;
  });

  createEffect(() => {
    const cfg = config();
    if (!cfg) return;
    const items = wallpapers() ?? [];
    for (const it of items) {
      it.display_name = cfg.strip_extension ? stripExtension(it.name) : it.name;
    }
  });

  return (
    <div class="popover-shell w-screen h-screen flex flex-col">
      <div class="flex items-center gap-2 px-2 pt-2">
        <div class="flex-1">
          <Show when={config()?.show_searchbar ?? true}>
            <Searchbar />
          </Show>
        </div>
        <button
          class="icon-btn mr-1"
          title="Settings"
          aria-label="Settings"
          onClick={() => void ipc.openSettings()}
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
        </button>
      </div>
      <Show when={config()?.per_screen}>
        <DisplayTabs />
      </Show>
      <Show when={sources().length > 1}>
        <div class="flex gap-1 px-3 py-1 text-xs">
          <button
            class="px-2 py-0.5 rounded-md"
            classList={{
              "bg-[var(--color-accent)] text-[var(--color-bg)]": activeSource() === null,
              "text-[var(--color-muted)]": activeSource() !== null,
            }}
            onClick={() => setActiveSource(null)}
          >
            All
          </button>
          <For each={sources()}>
            {(s) => (
              <button
                class="px-2 py-0.5 rounded-md"
                classList={{
                  "bg-[var(--color-accent)] text-[var(--color-bg)]": activeSource() === s.id,
                  "text-[var(--color-muted)]": activeSource() !== s.id,
                }}
                onClick={() => setActiveSource(s.id)}
              >
                {s.label}
              </button>
            )}
          </For>
        </div>
      </Show>
      <div class="flex-1 min-h-0">
        <Show
          when={config()?.layout === "grid"}
          fallback={<ThumbList items={filtered()} layout={config()?.layout ?? "horizontal"} />}
        >
          <ThumbGrid items={filtered()} />
        </Show>
      </div>
    </div>
  );
}
