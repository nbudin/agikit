import { resolve } from 'path';
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

export default defineConfig({
  root: 'dev-site',
  plugins: [react(), wasm(), topLevelAwait()],
  optimizeDeps: {
    exclude: ['@agikit/core'],
  },
  build: {
    rollupOptions: {
      input: {
        index: resolve(__dirname, 'dev-site/index.html'),
        'pic-editor': resolve(__dirname, 'dev-site/pic-editor.html'),
        'view-editor': resolve(__dirname, 'dev-site/view-editor.html'),
        'sound-editor': resolve(__dirname, 'dev-site/sound-editor.html'),
      },
    },
  },
});
