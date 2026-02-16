import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";

export default defineConfig({
  base: "/seonbi-rs/",
  plugins: [react(), wasm()],
  optimizeDeps: {
    exclude: ["@seonbi/seonbi-wasm"],
  },
});
