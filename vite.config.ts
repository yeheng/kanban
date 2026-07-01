import path from "path";
import browserslist from "browserslist";
import tailwindcss from "@tailwindcss/vite";
import vue from "@vitejs/plugin-vue";
import { browserslistToTargets } from "lightningcss";
import AutoImport from "unplugin-auto-import/vite";
import Component from "unplugin-vue-components/vite";
import { defineConfig } from "vitest/config";
import vueDevTools from "vite-plugin-vue-devtools";
import Layouts from "vite-plugin-vue-layouts";
import VueRouter from "vue-router/vite";
import { VueRouterAutoImports } from "vue-router/unplugin";

export default defineConfig({
  plugins: [
    VueRouter({
      exclude: ["**/components/**", "**/layouts/**", "**/data/**", "**/types/**", "**/validators/**"],
      dts: "src/types/route-map.d.ts",
    }),
    Layouts({ defaultLayout: "default" }),
    vue(),
    tailwindcss(),
    vueDevTools(),
    AutoImport({
      imports: ["vue", VueRouterAutoImports],
      dirs: ["src/composables/**/*.ts", "src/constants/**/*.ts", "src/stores/**/*.ts"],
      ignore: ["**/*.test.ts", "**/*.spec.ts"],
      defaultExportByFilename: true,
      dts: "src/types/auto-import.d.ts",
    }),
    Component({
      dirs: ["src/components"],
      collapseSamePrefixes: true,
      directoryAsNamespace: true,
      dts: "src/types/auto-import-components.d.ts",
    }),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  css: {
    transformer: "lightningcss",
    lightningcss: {
      targets: browserslistToTargets(browserslist(["> 1%", "last 2 versions"])),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    proxy: {
      "/api": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
    },
  },
  test: { environment: "jsdom", globals: true },
});
