import { Show, createSignal } from "solid-js";
import { config, patchConfig } from "../../lib/store";
import type { RotateMode } from "../../lib/ipc";
import { ipc } from "../../lib/ipc";

const PRESETS: { label: string; minutes: number }[] = [
  { label: "5 minutes", minutes: 5 },
  { label: "15 minutes", minutes: 15 },
  { label: "30 minutes", minutes: 30 },
  { label: "1 hour", minutes: 60 },
  { label: "6 hours", minutes: 360 },
  { label: "24 hours", minutes: 1440 },
];

type ModeKey = RotateMode["kind"] | "interval-custom";

function modeKey(m: RotateMode): ModeKey {
  if (m.kind !== "interval") return m.kind;
  return PRESETS.some((p) => p.minutes === m.minutes) ? "interval" : "interval-custom";
}

export function AutoRotate() {
  const [custom, setCustom] = createSignal(60);

  return (
    <div class="space-y-6">
      <h1 class="text-xl font-semibold">Auto-rotate</h1>
      <Show when={config()}>
        {(c) => {
          const m = c().rotate;
          const key = modeKey(m);
          return (
            <>
              <Field label="Mode">
                <select
                  value={key}
                  onChange={(e) => {
                    const v = e.currentTarget.value as ModeKey;
                    if (v === "off") patchConfig({ rotate: { kind: "off" } });
                    else if (v === "interval")
                      patchConfig({ rotate: { kind: "interval", minutes: 15 } });
                    else if (v === "interval-custom")
                      patchConfig({ rotate: { kind: "interval", minutes: custom() } });
                    else if (v === "sunrise_sunset")
                      patchConfig({ rotate: { kind: "sunrise_sunset" } });
                    else if (v === "per_space") patchConfig({ rotate: { kind: "per_space" } });
                  }}
                  class="px-3 py-2 bg-[var(--color-surface)] rounded-md text-sm"
                >
                  <option value="off">Off</option>
                  <option value="interval">Interval (preset)</option>
                  <option value="interval-custom">Interval (custom)</option>
                  <option value="sunrise_sunset">Sunrise / sunset</option>
                  <option value="per_space">Per-Space rotation</option>
                </select>
              </Field>
              <Show when={m.kind === "interval" && key === "interval"}>
                <Field label="Interval">
                  <select
                    value={m.kind === "interval" ? m.minutes : 15}
                    onChange={(e) =>
                      patchConfig({
                        rotate: { kind: "interval", minutes: Number(e.currentTarget.value) },
                      })
                    }
                    class="px-3 py-2 bg-[var(--color-surface)] rounded-md text-sm"
                  >
                    {PRESETS.map((p) => (
                      <option value={p.minutes}>{p.label}</option>
                    ))}
                  </select>
                </Field>
              </Show>
              <Show when={key === "interval-custom"}>
                <Field label="Custom interval (minutes)">
                  <input
                    type="number"
                    min="1"
                    value={custom()}
                    onInput={(e) => setCustom(Number(e.currentTarget.value))}
                    onChange={(e) =>
                      patchConfig({
                        rotate: {
                          kind: "interval",
                          minutes: Math.max(1, Number(e.currentTarget.value)),
                        },
                      })
                    }
                    class="w-32 px-3 py-2 bg-[var(--color-surface)] rounded-md text-sm"
                  />
                </Field>
              </Show>
              <Show when={m.kind === "sunrise_sunset"}>
                <button
                  class="px-3 py-2 rounded-md bg-[var(--color-surface)] text-sm"
                  onClick={() => ipc.requestLocation()}
                >
                  Grant location permission
                </button>
              </Show>
            </>
          );
        }}
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
