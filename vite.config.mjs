import { defineConfig } from "vite";
import { resolve } from "node:path";
import deploymentConfig from "./site/public/staticwebapp.config.json" with { type: "json" };

export default defineConfig({
  root: "site",
  preview: {
    // Keep browser regressions on the same response policy as the static deployment.
    headers: deploymentConfig.globalHeaders
  },
  build: {
    outDir: "../dist/site",
    emptyOutDir: true,
    target: "es2022",
    rollupOptions: {
      input: {
        main: resolve(import.meta.dirname, "site/index.html"),
        privacy: resolve(import.meta.dirname, "site/privacy/index.html"),
        terms: resolve(import.meta.dirname, "site/terms/index.html"),
        offline: resolve(import.meta.dirname, "site/offline.html")
      }
    }
  }
});
