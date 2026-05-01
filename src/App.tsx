import { Match, Switch, createMemo, onMount } from "solid-js";
import { Popover } from "./popover/Popover";
import { Settings } from "./settings/Settings";
import { Onboarding } from "./onboarding/Onboarding";
import { applyThemeFromBackend } from "./lib/theme";

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
