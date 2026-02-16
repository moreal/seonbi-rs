import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import path from "path";

export default defineConfig({
  base: "/seonbi-rs/",
  plugins: [react(), wasm()],
  optimizeDeps: {
    exclude: ["@seonbi/wasm"],
  },
  server: {
    fs: {
      allow: [
        __dirname,
        path.resolve(__dirname, "../crates/wasm/pkg"),
      ],
    },
  },
});
