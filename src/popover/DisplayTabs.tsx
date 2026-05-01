import { For, createResource } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { activeDisplay, setActiveDisplay } from "../lib/store";

interface DisplayInfo {
  id: string;
  name: string;
}

export function DisplayTabs() {
  const [displays] = createResource<DisplayInfo[]>(async () => {
    try {
      return await invoke<DisplayInfo[]>("list_displays");
    } catch {
      return [];
    }
  });

  return (
    <div class="flex gap-1 px-3 pb-1 text-xs">
      <For each={displays() ?? []}>
        {(d) => (
          <button
            class="px-2 py-0.5 rounded-md"
            classList={{
              "bg-[var(--color-accent)] text-[var(--color-bg)]": activeDisplay() === d.id,
              "text-[var(--color-muted)]": activeDisplay() !== d.id,
            }}
            onClick={() => setActiveDisplay(d.id)}
          >
            {d.name}
          </button>
        )}
      </For>
    </div>
  );
}
