/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        app: 'var(--bg-app)',
        surface: 'var(--bg-surface)',
        elevated: 'var(--bg-elevated)',
        'border-subtle': 'var(--border-subtle)',
        'border-bold': 'var(--border-bold)',
        accent: {
          amber: 'var(--accent-amber)',
          cyan: 'var(--accent-cyan)',
        },
      },
    },
  },
  plugins: [],
}
