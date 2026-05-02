import { Match, Switch, createEffect, createMemo, onMount } from "solid-js";
import { Popover } from "./popover/Popover";
import { Settings } from "./settings/Settings";
import { Onboarding } from "./onboarding/Onboarding";
import { applyFont, applyTheme, applyThemeFromBackend } from "./lib/theme";
import { config } from "./lib/store";
import { ipc } from "./lib/ipc";

type Route = "popover" | "settings" | "onboarding";

function currentRoute(): Route {
  const hash = window.location.hash.replace(/^#\//, "");
  if (hash === "settings" || hash === "onboarding") return hash;
  return "popover";
}

export function App() {
  const route = createMemo(currentRoute);

  onMount(() => {
    void applyThemeFromBackend();
  });

  createEffect(() => {
    const cfg = config();
    if (!cfg) return;
    void (async () => {
      const themes = await ipc.listThemes();
      const t = themes.find((x) => x.id === cfg.theme_id);
      if (t) applyTheme(t);
      const fonts = await ipc.listFonts();
      const f = fonts.find((x) => x.id === cfg.font_id);
      if (f) applyFont(f.family);
    })();
  });

  return (
    <Switch fallback={<Popover />}>
      <Match when={route() === "popover"}>
        <Popover />
      </Match>
      <Match when={route() === "settings"}>
        <Settings />
      </Match>
      <Match when={route() === "onboarding"}>
        <Onboarding />
      </Match>
    </Switch>
  );
}
