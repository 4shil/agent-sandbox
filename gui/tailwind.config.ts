/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: [
    './index.html',
    './src/**/*.{js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {
      colors: {
        // Dark theme colors matching agent-sandbox aesthetic
        background: '#0a0a0f',
        surface: '#12121a',
        'surface-hover': '#1a1a24',
        border: '#2a2a3a',
        primary: '#64b4ff',
        secondary: '#b482ff',
        accent: '#ffb43c',
        success: '#50dc8c',
        warning: '#ffc83c',
        error: '#ff5050',
        'text-primary': '#dcdcff',
        'text-muted': '#646478',
      },
    },
  },
  plugins: [],
}
