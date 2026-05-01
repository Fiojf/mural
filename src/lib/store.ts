import { createSignal, createResource } from "solid-js";
import { ipc, type Config, type WallpaperItem } from "./ipc";

export const [config, { refetch: refetchConfig, mutate: setConfigSignal }] =
  createResource<Config>(() => ipc.getConfig());

export const [wallpapers, { refetch: refetchWallpapers }] =
  createResource<WallpaperItem[]>(() => ipc.listWallpapers());

export const [search, setSearch] = createSignal("");
export const [activeSource, setActiveSource] = createSignal<string | null>(null);
export const [activeDisplay, setActiveDisplay] = createSignal<string | null>(null);

export async function patchConfig(patch: Partial<Config>): Promise<void> {
  const next = await ipc.setConfig(patch);
  setConfigSignal(next);
}
