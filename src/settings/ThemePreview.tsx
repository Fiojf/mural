import type { Theme } from "../lib/ipc";

export function ThemePreview(props: { theme: Theme }) {
  const c = props.theme.colors;
  return (
    <div
      class="w-full h-16 rounded-md flex items-center px-3 gap-2"
      style={{ background: c.bg, color: c.text, border: `1px solid ${c.border}` }}
    >
      <Swatch color={c.accent} />
      <Swatch color={c.selected_border} />
      <Swatch color={c.surface} />
      <Swatch color={c.muted} />
      <span class="ml-auto text-xs opacity-80">{props.theme.base}</span>
    </div>
  );
}

function Swatch(props: { color: string }) {
  return (
    <span
      class="inline-block w-4 h-4 rounded-full border border-black/20"
      style={{ background: props.color }}
    />
  );
}
