import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "node:path";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: { "@": path.resolve(__dirname, "src") },
  },
  server: {
    port: 5173,
    // During dev, the UI talks to a separately-running nub on :8765.
    proxy: {
      "/api": { target: "http://127.0.0.1:8765", ws: true, changeOrigin: true },
    },
  },
  build: {
    // Output goes here so build.rs can find ui/dist/index.html when embed-ui is on.
    outDir: "dist",
    emptyOutDir: true,
  },
});
