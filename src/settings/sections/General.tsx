import { Show } from "solid-js";
import { config, patchConfig } from "../../lib/store";
import { ipc } from "../../lib/ipc";

export function General() {
  return (
    <div class="space-y-6">
      <h1 class="text-xl font-semibold">General</h1>
      <Show when={config()}>
        {(c) => (
          <>
            <Field label="Wallpaper folder">
              <div class="flex gap-2 items-center">
                <code class="flex-1 px-3 py-2 bg-[var(--color-surface)] rounded-md text-sm">
                  {c().folder}
                </code>
                <button
                  class="px-3 py-2 rounded-md bg-[var(--color-surface)] text-sm hover:bg-[var(--color-border)]"
                  onClick={() => ipc.revealInFinder(c().folder)}
                >
                  Reveal
                </button>
              </div>
            </Field>
            <Field label="Hotkey">
              <input
                value={c().hotkey}
                onChange={(e) => patchConfig({ hotkey: e.currentTarget.value })}
                class="w-48 px-3 py-2 bg-[var(--color-surface)] rounded-md text-sm font-mono"
              />
              <p class="text-xs text-[var(--color-muted)] mt-1">
                Format: modifiers + key, e.g. CmdOrCtrl+Shift+W
              </p>
            </Field>
          </>
        )}
      </Show>
    </div>
  );
}

function Field(props: { label: string; children: any }) {
  return (
    <label class="block">
      <span class="text-sm text-[var(--color-muted)] block mb-2">{props.label}</span>
      {props.children}
    </label>
  );
}
