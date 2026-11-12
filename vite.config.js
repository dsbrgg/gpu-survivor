import { defineConfig } from "vite";
import { newsPlugin } from "./src/director/plugin.mjs";

export default defineConfig({
  plugins: [newsPlugin()],
  root: "src/web",
  build: { outDir: "../../dist", emptyOutDir: true },
  server: { port: 4173, strictPort: true },
  preview: { port: 4173, strictPort: true },
});
