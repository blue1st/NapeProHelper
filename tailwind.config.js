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
        brand: {
          50: '#f0f9ff',
          100: '#e0f2fe',
          400: '#38bdf8',
          500: '#0284c7',
          600: '#0369a1',
          900: '#0c4a6e',
        },
        nape: {
          bg: '#0f172a',
          card: '#1e293b',
          border: '#334155',
          accent: '#6366f1',
        }
      },
    },
  },
  plugins: [],
}
