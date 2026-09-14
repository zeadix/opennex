import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Tauri expects a fixed dev port; the Rust shell loads this URL in dev.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 5183,
    strictPort: true,
  },
  build: {
    outDir: "dist",
    target: "chrome110",
  },
});
