import {defineConfig} from "vite";

export default defineConfig({
    define: {"process.env.NODE_ENV": JSON.stringify("production")},
    build: {
        outDir: "dist/frontend",
        lib: {entry: "frontend/index.tsx", formats: ["es"], fileName: () => "index.js"},
        cssCodeSplit: false,
        rollupOptions: {output: {assetFileNames: "[name][extname]"}},
    },
});
