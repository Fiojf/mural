import { search, setSearch } from "../lib/store";
import { debounce } from "../lib/format";

const update = debounce((v: string) => setSearch(v), 80);

export function Searchbar() {
  return (
    <div class="px-3 pt-3 pb-2">
      <input
        type="search"
        value={search()}
        placeholder="Search wallpapers…"
        autofocus
        onInput={(e) => update(e.currentTarget.value)}
        class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded-md px-3 py-2 text-sm outline-none focus:border-[var(--color-accent)]"
      />
    </div>
  );
}
