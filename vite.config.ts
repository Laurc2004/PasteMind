import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => {
  const plugins = await sveltekit();

  return {
    clearScreen: false,
    plugins,
    envPrefix: ['VITE_', 'TAURI_'],
    server: {
      port: 1420,
      strictPort: true,
      host: host || false,
      hmr: host
        ? {
            protocol: 'ws',
            host,
            port: 1421
          }
        : undefined,
      watch: {
        ignored: ['**/src-tauri/**']
      }
    },
    build: {
      target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
      minify: process.env.TAURI_DEBUG ? false : ('esbuild' as const),
      sourcemap: !!process.env.TAURI_DEBUG
    }
  };
});
