/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx,js,jsx}"],
  theme: {
    extend: {
      colors: {
        bg: "var(--color-bg)",
        surface: "var(--color-surface)",
        text: "var(--color-text)",
        muted: "var(--color-muted)",
        accent: "var(--color-accent)",
        border: "var(--color-border)",
        "selected-border": "var(--color-selected-border)",
      },
      fontFamily: {
        ui: "var(--font-ui), system-ui, -apple-system, sans-serif",
        mono: "var(--font-mono), ui-monospace, monospace",
      },
      borderRadius: {
        popover: "14px",
      },
      transitionTimingFunction: {
        mural: "cubic-bezier(0.16, 1, 0.3, 1)",
      },
    },
  },
  plugins: [],
};
