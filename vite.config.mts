import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import viteReact from "@vitejs/plugin-react-swc";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";
import wasm from "vite-plugin-wasm";
import tailwindcss from "@tailwindcss/vite";

const shellDir = (...path: string[]) =>
  resolve(fileURLToPath(import.meta.url), "..", ...path);

export default defineConfig((env) => ({
  appType: "spa",
  base: process.env.PUBLIC_BASE_PATH || "/",
  root: "./src",
  publicDir: "./public",
  server: {
    port: 10880,
  },
  esbuild: {
    target: ["esnext", "chrome115", "firefox120", "safari16"],
    format: "esm",
    platform: "browser",
  },
  optimizeDeps: {
    exclude: ["@livestore/wa-sqlite"],
  },
  plugins: [
    tailwindcss(),
    wasm(),
    tanstackRouter({
      routesDirectory: shellDir("src/routes"),
      generatedRouteTree: shellDir("src/routeTree.gen.ts"),
    }),
    viteReact(),
    tsconfigPaths(),
  ],
  build: {
    target: ["chrome115", "firefox120", "safari16"],
    sourcemap: true,
    emptyOutDir: true,
    outDir: "../build",
  },
  test: {
    environment: "happy-dom",
    globals: true,
    server: {
      deps: {
        inline: ["wasm_pathfinder"],
      },
    },
  },
}));
