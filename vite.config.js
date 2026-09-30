import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

// Tryb deweloperski: `npm run dev` (frontend, port 5173) + `npm run dev:server` (API, port 8420).
// Vite przekazuje /api do serwera Rust, więc w przeglądarce wszystko jest pod jednym adresem.
export default defineConfig({
  plugins: [sveltekit()],
  server: {
    proxy: {
      "/api": "http://127.0.0.1:8420",
    },
  },
});
