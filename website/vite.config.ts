import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Served from GitHub Pages as a project site, so assets live under /mp3-tagger/.
export default defineConfig({
  base: "/mp3-tagger/",
  plugins: [react(), tailwindcss()],
});
