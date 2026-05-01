export function About() {
  return (
    <div class="space-y-3">
      <h1 class="text-xl font-semibold">About Mural</h1>
      <p class="text-sm text-[var(--color-muted)]">Version 0.1.0</p>
      <p class="text-sm text-[var(--color-muted)]">Licensed under GPL-3.0-only.</p>
      <p class="text-sm">
        <a
          href="https://github.com/Fiojf/mural"
          target="_blank"
          rel="noreferrer"
          class="text-[var(--color-accent)] hover:underline"
        >
          github.com/Fiojf/mural
        </a>
      </p>
    </div>
  );
}
