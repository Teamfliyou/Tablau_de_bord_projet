import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

// Le frontend est embarqué dans l'exécutable Tauri : aucun serveur n'est nécessaire en production.
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { target: 'es2021', outDir: 'dist', emptyOutDir: true },
});
