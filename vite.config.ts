import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import path from 'path';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

export default defineConfig({
  plugins: [wasm(), topLevelAwait(), vue()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@workers': path.resolve(__dirname, './src/workers'),
      '@components': path.resolve(__dirname, './src/components'),
      '@utils': path.resolve(__dirname, './src/utils'),
      '@composables': path.resolve(__dirname, './src/composables'),
      'synapse-core': path.resolve(__dirname, './crates/synapse-core/pkg/synapse_core.js'),
    },
  },
  server: {
    port: 5173,
    host: '0.0.0.0',
  },
  build: {
    target: 'ES2020',
    minify: 'terser',
    sourcemap: true,
  },
  optimizeDeps: {
    exclude: ['synapse-core'],
  },
  worker: {
    format: 'es',
  },
});
