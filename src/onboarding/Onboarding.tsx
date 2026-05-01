import { For, Show, createResource, createSignal } from "solid-js";
import { ipc } from "../lib/ipc";
import { applyTheme } from "../lib/theme";
import { ThemePreview } from "../settings/ThemePreview";
import { config, patchConfig } from "../lib/store";
import { getCurrentWindow } from "@tauri-apps/api/window";

type Step = "welcome" | "folder" | "theme" | "hotkey" | "done";

export function Onboarding() {
  const [step, setStep] = createSignal<Step>("welcome");
  const [themes] = createResource(() => ipc.listThemes());

  async function finish() {
    await ipc.onboardingComplete();
    const w = getCurrentWindow();
    await w.close();
  }

  return (
    <div class="h-screen w-screen flex flex-col bg-[var(--color-bg)] text-[var(--color-text)]">
      <header class="px-8 pt-8 pb-4">
        <h1 class="text-2xl font-semibold">Mural</h1>
        <p class="text-sm text-[var(--color-muted)]">A fast wallpaper picker for macOS.</p>
      </header>
      <main class="flex-1 px-8 overflow-y-auto">
        <Show when={step() === "welcome"}>
          <Welcome onNext={() => setStep("folder")} />
        </Show>
        <Show when={step() === "folder"}>
          <Folder onNext={() => setStep("theme")} />
        </Show>
        <Show when={step() === "theme"}>
          <ThemePick themes={themes() ?? []} onNext={() => setStep("hotkey")} />
        </Show>
        <Show when={step() === "hotkey"}>
          <Hotkey onNext={() => setStep("done")} />
        </Show>
        <Show when={step() === "done"}>
          <Done onFinish={finish} />
        </Show>
      </main>
    </div>
  );
}

function Welcome(props: { onNext: () => void }) {
  return (
    <div class="space-y-6 max-w-prose">
      <p>
        Mural lives in your menu bar. Press a hotkey from anywhere to summon a floating picker,
        then click any wallpaper to apply it.
      </p>
      <button
        class="px-4 py-2 rounded-md bg-[var(--color-accent)] text-[var(--color-bg)] font-medium"
        onClick={props.onNext}
      >
        Get started
      </button>
    </div>
  );
}

function Folder(props: { onNext: () => void }) {
  return (
    <div class="space-y-4 max-w-prose">
      <h2 class="text-lg font-medium">Wallpaper folder</h2>
      <Show when={config()}>
        {(c) => (
          <p class="text-sm text-[var(--color-muted)]">
            Mural will watch <code class="text-[var(--color-text)]">{c().folder}</code> for
            wallpapers. We'll create it and drop a few samples in there if it doesn't exist.
          </p>
        )}
      </Show>
      <button
        class="px-4 py-2 rounded-md bg-[var(--color-accent)] text-[var(--color-bg)] font-medium"
        onClick={props.onNext}
      >
        Continue
      </button>
    </div>
  );
}

function ThemePick(props: { themes: ReturnType<typeof Array.prototype.slice> | any[]; onNext: () => void }) {
  return (
    <div class="space-y-4">
      <h2 class="text-lg font-medium">Pick a theme</h2>
      <div class="grid grid-cols-2 gap-3 max-h-96 overflow-y-auto">
        <For each={props.themes}>
          {(t: any) => (
            <button
              class="text-left rounded-lg border-2 border-[var(--color-border)] p-3 hover:border-[var(--color-accent)]"
              onMouseEnter={() => applyTheme(t)}
              onClick={async () => {
                applyTheme(t);
                await patchConfig({ theme_id: t.id });
              }}
            >
              <ThemePreview theme={t} />
              <div class="mt-2 text-sm">{t.name}</div>
            </button>
          )}
        </For>
      </div>
      <button
        class="px-4 py-2 rounded-md bg-[var(--color-accent)] text-[var(--color-bg)] font-medium"
        onClick={props.onNext}
      >
        Continue
      </button>
    </div>
  );
}

function Hotkey(props: { onNext: () => void }) {
  return (
    <div class="space-y-4 max-w-prose">
      <h2 class="text-lg font-medium">Hotkey</h2>
      <Show when={config()}>
        {(c) => (
          <input
            value={c().hotkey}
            onChange={(e) => patchConfig({ hotkey: e.currentTarget.value })}
            class="w-64 px-3 py-2 bg-[var(--color-surface)] rounded-md text-sm font-mono"
          />
        )}
      </Show>
      <p class="text-xs text-[var(--color-muted)]">
        Default <code>CmdOrCtrl+Shift+W</code>. Change anytime in Settings.
      </p>
      <button
        class="px-4 py-2 rounded-md bg-[var(--color-accent)] text-[var(--color-bg)] font-medium"
        onClick={props.onNext}
      >
        Continue
      </button>
    </div>
  );
}

function Done(props: { onFinish: () => void }) {
  return (
    <div class="space-y-4 max-w-prose">
      <h2 class="text-lg font-medium">All set</h2>
      <p>
        Press your hotkey to summon Mural. Drop images into your wallpaper folder anytime — they
        appear automatically.
      </p>
      <button
        class="px-4 py-2 rounded-md bg-[var(--color-accent)] text-[var(--color-bg)] font-medium"
        onClick={props.onFinish}
      >
        Done
      </button>
    </div>
  );
}
