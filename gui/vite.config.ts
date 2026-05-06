import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  // Tauri expects the frontend to be built to `../src-tauri/dist`
  build: {
    outDir: '../src-tauri/dist',
  },
  resolve: {
    alias: {
      '@': '/src',
    },
  },
})
