import { defineConfig } from "vite";

export default defineConfig({
  base: process.env.BASE_PATH || "/",
  build: {
    target: "es2022",
    outDir: "dist",
    emptyOutDir: true,
  },
});
