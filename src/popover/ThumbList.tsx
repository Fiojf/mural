import { For, Show } from "solid-js";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { WallpaperItem, Layout } from "../lib/ipc";
import { ipc } from "../lib/ipc";
import { activeDisplay, config, selectedPath, setSelectedPath } from "../lib/store";

interface Props {
  items: WallpaperItem[];
  layout: Layout;
}

export function ThumbList(props: Props) {
  const horizontal = () => props.layout === "horizontal";

  const apply = (item: WallpaperItem) => {
    setSelectedPath(item.path);
    void ipc.setWallpaper(item.path, activeDisplay());
  };

  return (
    <div
      class="h-full w-full"
      classList={{
        "overflow-x-auto overflow-y-hidden": horizontal(),
        "overflow-y-auto overflow-x-hidden": !horizontal(),
      }}
    >
      <div
        class="p-2"
        classList={{
          "flex flex-row gap-2 items-center h-full": horizontal(),
          "flex flex-col gap-2": !horizontal(),
        }}
      >
        <For each={props.items}>
          {(item) => (
            <button
              class="thumb shrink-0 flex flex-col"
              data-selected={selectedPath() === item.path}
              classList={{
                "w-[160px]": horizontal(),
                "w-full": !horizontal(),
              }}
              onClick={() => apply(item)}
              title={item.display_name}
            >
              <Show
                when={item.thumb_url}
                fallback={<div class="w-full aspect-video bg-[var(--color-surface)] rounded-md" />}
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
                <div class="text-xs truncate text-[var(--color-muted)] mt-1 w-full text-left px-1">
                  {item.display_name}
                </div>
              </Show>
            </button>
          )}
        </For>
      </div>
    </div>
  );
}
