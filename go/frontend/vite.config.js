/* eslint-disable no-undef */
import { defineConfig } from "vite";
import { resolve } from "node:path";

export default defineConfig({
  plugins: [],
  build: {
    outDir: "dist",
    rollupOptions: {
      input: {
        index: resolve(__dirname, "index.html"),
      },
    },
  },
  server: {
    port: 8081,
    strictPort: true,
    // Proxy API calls to the go backend on port 8080
    proxy: {
      "/api": {
        target: "http://localhost:8080",
        changeOrigin: true,
        secure: false,
      },
    },
  },
});
