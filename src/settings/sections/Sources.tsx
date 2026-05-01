import { For, Show, createResource, createSignal } from "solid-js";
import { ipc, type SourceEntry } from "../../lib/ipc";
import { formatRelativeTime } from "../../lib/format";

const INTERVALS = [1, 6, 12, 24, 72, 168];

export function Sources() {
  const [list, { refetch }] = createResource<SourceEntry[]>(() => ipc.sourcesList());
  const [adding, setAdding] = createSignal(false);
  const [url, setUrl] = createSignal("");
  const [refField, setRefField] = createSignal("");
  const [pathField, setPathField] = createSignal("");
  const [interval, setInterval] = createSignal(24);
  const [error, setError] = createSignal<string | null>(null);

  async function submit() {
    setError(null);
    try {
      await ipc.sourcesAddGithub({
        url: url().trim(),
        ref: refField().trim() || null,
        path: pathField().trim() || null,
        sync_interval_hours: interval(),
      });
      setAdding(false);
      setUrl("");
      setRefField("");
      setPathField("");
      void refetch();
    } catch (e: any) {
      setError(String(e?.message ?? e));
    }
  }

  return (
    <div class="space-y-6">
      <div class="flex items-center justify-between">
        <h1 class="text-xl font-semibold">Sources</h1>
        <button
          class="px-3 py-2 rounded-md bg-[var(--color-accent)] text-[var(--color-bg)] text-sm font-medium"
          onClick={() => setAdding(true)}
        >
          Add GitHub source
        </button>
      </div>

      <For each={list() ?? []}>
        {(s) => (
          <div class="flex items-start gap-3 p-3 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]">
            <input
              type="checkbox"
              checked={s.enabled}
              disabled={s.kind === "local"}
              onChange={async (e) => {
                await ipc.sourcesSetEnabled(s.id, e.currentTarget.checked);
                void refetch();
              }}
              class="mt-1 accent-[var(--color-accent)]"
            />
            <div class="flex-1 min-w-0">
              <div class="font-medium text-sm">{s.label}</div>
              <Show when={s.url}>
                <div class="text-xs text-[var(--color-muted)] truncate">{s.url}</div>
              </Show>
              <div class="text-xs text-[var(--color-muted)] mt-1">
                {s.item_count} items · last sync {formatRelativeTime(s.last_sync_iso)}
                <Show when={s.error}>
                  <span class="text-red-400 ml-2">{s.error}</span>
                </Show>
              </div>
            </div>
            <Show when={s.kind === "github"}>
              <button
                class="px-2 py-1 text-xs rounded-md bg-[var(--color-bg)]"
                onClick={async () => {
                  await ipc.sourcesSync(s.id);
                  void refetch();
                }}
              >
                Sync now
              </button>
              <button
                class="px-2 py-1 text-xs rounded-md bg-[var(--color-bg)]"
                onClick={async () => {
                  await ipc.sourcesRemove(s.id);
                  void refetch();
                }}
              >
                Remove
              </button>
            </Show>
          </div>
        )}
      </For>

      <Show when={adding()}>
        <div class="p-4 rounded-lg border border-[var(--color-border)] space-y-3">
          <h2 class="text-sm font-medium">Add GitHub repository</h2>
          <input
            placeholder="https://github.com/user/wallpapers"
            value={url()}
            onInput={(e) => setUrl(e.currentTarget.value)}
            class="w-full px-3 py-2 bg-[var(--color-surface)] rounded-md text-sm"
          />
          <div class="flex gap-2">
            <input
              placeholder="ref (branch/tag/commit, optional)"
              value={refField()}
              onInput={(e) => setRefField(e.currentTarget.value)}
              class="flex-1 px-3 py-2 bg-[var(--color-surface)] rounded-md text-sm"
            />
            <input
              placeholder="path (subdir, optional)"
              value={pathField()}
              onInput={(e) => setPathField(e.currentTarget.value)}
              class="flex-1 px-3 py-2 bg-[var(--color-surface)] rounded-md text-sm"
            />
          </div>
          <label class="block text-sm">
            Sync every
            <select
              value={interval()}
              onChange={(e) => setInterval(Number(e.currentTarget.value))}
              class="ml-2 px-2 py-1 bg-[var(--color-surface)] rounded-md text-sm"
            >
              {INTERVALS.map((h) => (
                <option value={h}>{h}h</option>
              ))}
            </select>
          </label>
          <Show when={error()}>
            <div class="text-xs text-red-400">{error()}</div>
          </Show>
          <div class="flex gap-2 justify-end">
            <button
              class="px-3 py-2 rounded-md text-sm"
              onClick={() => {
                setAdding(false);
                setError(null);
              }}
            >
              Cancel
            </button>
            <button
              class="px-3 py-2 rounded-md bg-[var(--color-accent)] text-[var(--color-bg)] text-sm font-medium"
              onClick={submit}
            >
              Add
            </button>
          </div>
        </div>
      </Show>
    </div>
  );
}
