import { Show } from "solid-js";
import { config, patchConfig } from "../../lib/store";

export function Display() {
  return (
    <div class="space-y-6">
      <h1 class="text-xl font-semibold">Display</h1>
      <Show when={config()}>
        {(c) => (
          <>
            <Toggle
              label="Pick wallpaper per display"
              hint="When on, choose a different wallpaper for each connected display."
              value={c().per_screen}
              onChange={(v) => patchConfig({ per_screen: v })}
            />
            <Toggle
              label="Per-Space wallpapers"
              hint="When on, set wallpaper only on the active Space."
              value={c().per_space}
              onChange={(v) => patchConfig({ per_space: v })}
            />
            <Toggle
              label="Mirror to lock screen"
              hint="Best-effort. Requires admin prompt; falls back gracefully."
              value={c().lock_screen_mirror}
              onChange={(v) => patchConfig({ lock_screen_mirror: v })}
            />
          </>
        )}
      </Show>
    </div>
  );
}

function Toggle(props: {
  label: string;
  hint?: string;
  value: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label class="flex items-start gap-3 cursor-pointer">
      <input
        type="checkbox"
        checked={props.value}
        onChange={(e) => props.onChange(e.currentTarget.checked)}
        class="mt-1 accent-[var(--color-accent)]"
      />
      <div>
        <div class="text-sm">{props.label}</div>
        {props.hint && <div class="text-xs text-[var(--color-muted)] mt-0.5">{props.hint}</div>}
      </div>
    </label>
  );
}
