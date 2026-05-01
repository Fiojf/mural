import { For, Show, createResource } from "solid-js";
import { ipc, type AnimationKind } from "../../lib/ipc";
import { config, patchConfig } from "../../lib/store";
import { applyFont, applyTheme } from "../../lib/theme";
import { ThemePreview } from "../ThemePreview";

const ANIMATIONS: AnimationKind[] = ["fade", "scale", "slide-down", "none"];

export function Appearance() {
  const [themes] = createResource(() => ipc.listThemes());
  const [fonts] = createResource(() => ipc.listFonts());

  return (
    <div class="space-y-6">
      <h1 class="text-xl font-semibold">Appearance</h1>
      <Show when={config()}>
        {(c) => (
          <>
            <section>
              <h2 class="text-sm text-[var(--color-muted)] mb-3">Theme</h2>
              <div class="grid grid-cols-2 gap-3">
                <For each={themes() ?? []}>
                  {(t) => (
                    <button
                      class="text-left rounded-lg border-2 p-3 transition-colors"
                      classList={{
                        "border-[var(--color-accent)]": c().theme_id === t.id,
                        "border-[var(--color-border)]": c().theme_id !== t.id,
                      }}
                      onMouseEnter={() => applyTheme(t)}
                      onMouseLeave={() => {
                        const cur = (themes() ?? []).find((x) => x.id === c().theme_id);
                        if (cur) applyTheme(cur);
                      }}
                      onClick={() => patchConfig({ theme_id: t.id })}
                    >
                      <ThemePreview theme={t} />
                      <div class="mt-2 text-sm">{t.name}</div>
                    </button>
                  )}
                </For>
              </div>
            </section>
            <Field label="Font">
              <select
                value={c().font_id}
                onChange={(e) => {
                  const id = e.currentTarget.value;
                  patchConfig({ font_id: id });
                  const f = (fonts() ?? []).find((x) => x.id === id);
                  if (f) applyFont(f.family);
                }}
                class="px-3 py-2 bg-[var(--color-surface)] rounded-md text-sm"
              >
                <For each={fonts() ?? []}>
                  {(f) => <option value={f.id}>{f.name}</option>}
                </For>
              </select>
            </Field>
            <Field label="Open animation">
              <select
                value={c().open_animation}
                onChange={(e) =>
                  patchConfig({ open_animation: e.currentTarget.value as AnimationKind })
                }
                class="px-3 py-2 bg-[var(--color-surface)] rounded-md text-sm"
              >
                {ANIMATIONS.map((a) => (
                  <option value={a}>{a}</option>
                ))}
              </select>
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
