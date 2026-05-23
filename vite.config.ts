import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "node:path";

// tauri reads the frontend from a fixed dev port and a static dist folder.
// the host override exists so tauri can serve the dev build to a device on the
// local network during mobile-style testing; on desktop it stays disabled.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "./src"),
    },
  },
  // tauri shows its own startup errors, so keep the vite banner from clearing them
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // the rust side has its own watcher, so vite should ignore it
      ignored: ["**/src-tauri/**"],
    },
  },
});
