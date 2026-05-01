import { Show, createSignal } from "solid-js";
import { General } from "./sections/General";
import { Display } from "./sections/Display";
import { Layout as LayoutSection } from "./sections/Layout";
import { Appearance } from "./sections/Appearance";
import { AutoRotate } from "./sections/AutoRotate";
import { Sources } from "./sections/Sources";
import { About } from "./sections/About";

type Tab = "general" | "display" | "layout" | "appearance" | "rotate" | "sources" | "about";

const TABS: { id: Tab; label: string }[] = [
  { id: "general", label: "General" },
  { id: "display", label: "Display" },
  { id: "layout", label: "Layout" },
  { id: "appearance", label: "Appearance" },
  { id: "rotate", label: "Auto-rotate" },
  { id: "sources", label: "Sources" },
  { id: "about", label: "About" },
];

export function Settings() {
  const [tab, setTab] = createSignal<Tab>("general");

  return (
    <div class="flex h-screen bg-[var(--color-bg)] text-[var(--color-text)]">
      <nav class="w-44 border-r border-[var(--color-border)] py-4 flex flex-col gap-1">
        {TABS.map((t) => (
          <button
            class="text-left px-4 py-2 mx-2 rounded-md text-sm transition-colors"
            classList={{
              "bg-[var(--color-surface)] text-[var(--color-text)]": tab() === t.id,
              "text-[var(--color-muted)] hover:bg-[var(--color-surface)]": tab() !== t.id,
            }}
            onClick={() => setTab(t.id)}
          >
            {t.label}
          </button>
        ))}
      </nav>
      <main class="flex-1 overflow-y-auto p-6">
        <Show when={tab() === "general"}>
          <General />
        </Show>
        <Show when={tab() === "display"}>
          <Display />
        </Show>
        <Show when={tab() === "layout"}>
          <LayoutSection />
        </Show>
        <Show when={tab() === "appearance"}>
          <Appearance />
        </Show>
        <Show when={tab() === "rotate"}>
          <AutoRotate />
        </Show>
        <Show when={tab() === "sources"}>
          <Sources />
        </Show>
        <Show when={tab() === "about"}>
          <About />
        </Show>
      </main>
    </div>
  );
}
