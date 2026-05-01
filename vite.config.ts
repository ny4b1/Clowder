import tailwindcss from "@tailwindcss/vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

export default defineConfig({
  clearScreen: false,
  plugins: [svelte(), tailwindcss()],
  server: {
    strictPort: true,
  },
});
