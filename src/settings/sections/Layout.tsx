import { Show } from "solid-js";
import { config, patchConfig } from "../../lib/store";
import type { Layout as L, Sort } from "../../lib/ipc";

const LAYOUTS: { v: L; label: string }[] = [
  { v: "horizontal", label: "Horizontal list" },
  { v: "grid", label: "Grid" },
  { v: "vertical", label: "Vertical list" },
];

const SORTS: { v: Sort; label: string }[] = [
  { v: "name_asc", label: "Name (A→Z)" },
  { v: "name_desc", label: "Name (Z→A)" },
  { v: "date_added", label: "Date added" },
  { v: "recent", label: "Recently used" },
  { v: "random", label: "Random / Shuffle" },
];

export function Layout() {
  return (
    <div class="space-y-6">
      <h1 class="text-xl font-semibold">Layout</h1>
      <Show when={config()}>
        {(c) => (
          <>
            <Field label="Layout">
              <select
                value={c().layout}
                onChange={(e) => patchConfig({ layout: e.currentTarget.value as L })}
                class="px-3 py-2 bg-[var(--color-surface)] rounded-md text-sm"
              >
                {LAYOUTS.map((x) => (
                  <option value={x.v}>{x.label}</option>
                ))}
              </select>
            </Field>
            <Field label="Sort order">
              <select
                value={c().sort}
                onChange={(e) => patchConfig({ sort: e.currentTarget.value as Sort })}
                class="px-3 py-2 bg-[var(--color-surface)] rounded-md text-sm"
              >
                {SORTS.map((x) => (
                  <option value={x.v}>{x.label}</option>
                ))}
              </select>
            </Field>
            <Toggle
              label="Show searchbar"
              value={c().show_searchbar}
              onChange={(v) => patchConfig({ show_searchbar: v })}
            />
            <Toggle
              label="Show filenames under thumbnails"
              value={c().show_filenames}
              onChange={(v) => patchConfig({ show_filenames: v })}
            />
            <Toggle
              label="Strip file extension from displayed name"
              value={c().strip_extension}
              onChange={(v) => patchConfig({ strip_extension: v })}
            />
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

function Toggle(props: { label: string; value: boolean; onChange: (v: boolean) => void }) {
  return (
    <label class="flex items-center gap-2 cursor-pointer text-sm">
      <input
        type="checkbox"
        checked={props.value}
        onChange={(e) => props.onChange(e.currentTarget.checked)}
        class="accent-[var(--color-accent)]"
      />
      {props.label}
    </label>
  );
}
