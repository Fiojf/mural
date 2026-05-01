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
import { ipc, onPopoverDismiss, onWallpaperEvent } from "../lib/ipc";
import { stripExtension } from "../lib/format";

export function Popover() {
  let unlistenList: (() => void) | undefined;
  let unlistenDismiss: (() => void) | undefined;

  onMount(async () => {
    unlistenList = await onWallpaperEvent((e) => {
      if (e.type === "list-changed" || e.type === "synced") void refetchWallpapers();
    });
    unlistenDismiss = await onPopoverDismiss(() => {
      void ipc.closePopover();
    });

    const onKey = (ev: KeyboardEvent) => {
      if (ev.key === "Escape") void ipc.closePopover();
    };
    window.addEventListener("keydown", onKey);
    onCleanup(() => window.removeEventListener("keydown", onKey));
  });

  onCleanup(() => {
    unlistenList?.();
    unlistenDismiss?.();
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
      const cmp = (a: typeof items[number], b: typeof items[number]) => {
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
      <Show when={config()?.show_searchbar ?? true}>
        <Searchbar />
      </Show>
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
