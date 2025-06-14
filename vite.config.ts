import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname (fileURLToPath(import.meta.url))

export default defineConfig({
  plugins: [
    tailwindcss(),
  ],
  build: {
    manifest: "manifest.json",
    rollupOptions: {
      input: {
        main : resolve(__dirname, 'web', 'index.ts'),
      }
    },
    outDir: 'static',
    emptyOutDir: true, // also necessary
  }, 
})

