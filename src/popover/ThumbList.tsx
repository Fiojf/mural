import { For, Show, createMemo } from "solid-js";
import { createVirtualizer } from "@tanstack/solid-virtual";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { WallpaperItem, Layout } from "../lib/ipc";
import { ipc } from "../lib/ipc";
import { activeDisplay, config } from "../lib/store";

interface Props {
  items: WallpaperItem[];
  layout: Layout;
}

export function ThumbList(props: Props) {
  let scrollEl: HTMLDivElement | undefined;
  const horizontal = () => props.layout === "horizontal";
  const itemSize = () => (horizontal() ? 168 : 84);

  const virt = createMemo(() =>
    createVirtualizer({
      count: props.items.length,
      getScrollElement: () => scrollEl ?? null,
      estimateSize: () => itemSize(),
      horizontal: horizontal(),
      overscan: 6,
    }),
  );

  const apply = (item: WallpaperItem) => {
    void ipc.setWallpaper(item.path, activeDisplay());
  };

  return (
    <div
      ref={(el) => (scrollEl = el)}
      class="h-full"
      classList={{
        "overflow-x-auto overflow-y-hidden": horizontal(),
        "overflow-y-auto overflow-x-hidden": !horizontal(),
      }}
    >
      <div
        style={{
          [horizontal() ? "width" : "height"]: `${virt().getTotalSize()}px`,
          [horizontal() ? "height" : "width"]: "100%",
          position: "relative",
        }}
      >
        <For each={virt().getVirtualItems()}>
          {(v) => {
            const item = props.items[v.index];
            return (
              <button
                class="thumb absolute"
                data-selected={false}
                style={{
                  [horizontal() ? "left" : "top"]: `${v.start}px`,
                  [horizontal() ? "width" : "height"]: `${v.size - 8}px`,
                  [horizontal() ? "height" : "width"]: horizontal() ? "calc(100% - 16px)" : "72px",
                  [horizontal() ? "top" : "left"]: "8px",
                }}
                onClick={() => apply(item)}
                title={item.display_name}
              >
                <Show
                  when={item.thumb_url}
                  fallback={<div class="w-full h-full bg-[var(--color-surface)] rounded-md" />}
                >
                  <img
                    src={convertFileSrc(item.thumb_url ?? "")}
                    alt={item.display_name}
                    class="w-full h-full object-cover rounded-md"
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
            );
          }}
        </For>
      </div>
    </div>
  );
}
