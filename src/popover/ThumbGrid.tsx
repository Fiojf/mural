import { For, Show } from "solid-js";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { WallpaperItem } from "../lib/ipc";
import { ipc } from "../lib/ipc";
import { activeDisplay, config } from "../lib/store";

interface Props {
  items: WallpaperItem[];
}

export function ThumbGrid(props: Props) {
  return (
    <div
      class="h-full overflow-y-auto p-3 grid gap-3"
      style={{ "grid-template-columns": "repeat(auto-fill, minmax(140px, 1fr))" }}
    >
      <For each={props.items}>
        {(item) => (
          <button
            class="thumb"
            data-selected={false}
            onClick={() => ipc.setWallpaper(item.path, activeDisplay())}
            title={item.display_name}
          >
            <Show
              when={item.thumb_url}
              fallback={
                <div class="aspect-video w-full bg-[var(--color-surface)] rounded-md" />
              }
            >
              <img
                src={convertFileSrc(item.thumb_url ?? "")}
                alt={item.display_name}
                class="w-full aspect-video object-cover rounded-md"
                loading="lazy"
                decoding="async"
              />
            </Show>
            <Show when={config()?.show_filenames}>
              <div class="text-xs mt-1 truncate text-[var(--color-muted)]">
                {item.display_name}
              </div>
            </Show>
          </button>
        )}
      </For>
    </div>
  );
}
