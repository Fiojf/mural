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
  const [showAdvanced, setShowAdvanced] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const [syncing, setSyncing] = createSignal<string | null>(null);
  const [rowError, setRowError] = createSignal<{ id: string; msg: string } | null>(null);

  function reset() {
    setUrl("");
    setRefField("");
    setPathField("");
    setInterval(24);
    setShowAdvanced(false);
    setError(null);
  }

  async function syncOne(id: string) {
    setSyncing(id);
    setRowError(null);
    try {
      await ipc.sourcesSync(id);
    } catch (e: any) {
      setRowError({ id, msg: String(e?.message ?? e) });
    } finally {
      setSyncing(null);
      void refetch();
    }
  }

  async function submit() {
    setError(null);
    if (!url().trim()) {
      setError("Paste a GitHub URL or owner/repo.");
      return;
    }
    setBusy(true);
    try {
      await ipc.sourcesAddGithub({
        url: url().trim(),
        ref: refField().trim() || null,
        path: pathField().trim() || null,
        sync_interval_hours: interval(),
      });
      setAdding(false);
      reset();
      void refetch();
    } catch (e: any) {
      setError(String(e?.message ?? e));
    } finally {
      setBusy(false);
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
            <Show when={rowError()?.id === s.id}>
              <span class="text-xs text-red-400 mr-2 max-w-[200px] truncate">
                {rowError()?.msg}
              </span>
            </Show>
            <Show when={s.kind === "github"}>
              <button
                class="px-2 py-1 text-xs rounded-md bg-[var(--color-bg)] disabled:opacity-50"
                disabled={syncing() === s.id}
                onClick={() => void syncOne(s.id)}
              >
                {syncing() === s.id ? "Syncing…" : "Sync now"}
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
          <h2 class="text-sm font-medium">Add a GitHub wallpaper repo</h2>
          <p class="text-xs text-[var(--color-muted)]">
            Paste any GitHub URL — or just <code>owner/repo</code>. Mural shallow-clones the default
            branch and picks up images automatically.
          </p>
          <input
            placeholder="dharmx/walls   or   https://github.com/dharmx/walls"
            value={url()}
            autofocus
            onInput={(e) => setUrl(e.currentTarget.value)}
            onKeyDown={(e) => e.key === "Enter" && void submit()}
            class="w-full px-3 py-2 bg-[var(--color-surface)] rounded-md text-sm"
          />
          <button
            type="button"
            class="text-xs text-[var(--color-muted)] hover:text-[var(--color-text)]"
            onClick={() => setShowAdvanced(!showAdvanced())}
          >
            {showAdvanced() ? "− Hide" : "+ Advanced"} (branch, subfolder, sync interval)
          </button>
          <Show when={showAdvanced()}>
            <div class="space-y-2 pt-1">
              <input
                placeholder="branch / tag / commit (default: main)"
                value={refField()}
                onInput={(e) => setRefField(e.currentTarget.value)}
                class="w-full px-3 py-2 bg-[var(--color-surface)] rounded-md text-sm"
              />
              <input
                placeholder="subfolder inside repo (e.g. macOS)"
                value={pathField()}
                onInput={(e) => setPathField(e.currentTarget.value)}
                class="w-full px-3 py-2 bg-[var(--color-surface)] rounded-md text-sm"
              />
              <label class="block text-sm">
                Sync every
                <select
                  value={interval()}
                  onChange={(e) => setInterval(Number(e.currentTarget.value))}
                  class="ml-2 px-2 py-1 bg-[var(--color-surface)] rounded-md text-sm"
                >
                  {INTERVALS.map((h) => (
                    <option value={h}>{h === 168 ? "week" : `${h}h`}</option>
                  ))}
                </select>
              </label>
            </div>
          </Show>
          <Show when={error()}>
            <div class="text-xs text-red-400">{error()}</div>
          </Show>
          <div class="flex gap-2 justify-end">
            <button
              class="px-3 py-2 rounded-md text-sm"
              onClick={() => {
                setAdding(false);
                reset();
              }}
            >
              Cancel
            </button>
            <button
              class="px-3 py-2 rounded-md bg-[var(--color-accent)] text-[var(--color-bg)] text-sm font-medium disabled:opacity-50"
              disabled={busy()}
              onClick={submit}
            >
              {busy() ? "Adding…" : "Add"}
            </button>
          </div>
        </div>
      </Show>
    </div>
  );
}
