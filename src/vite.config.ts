import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

// @ts-expect-error Vite loads this config in Node, but the browser tsconfig omits Node globals.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [sveltekit()],
  build: {
    assetsInlineLimit: 0,
    chunkSizeWarningLimit: 600,
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || "127.0.0.1",
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
});
