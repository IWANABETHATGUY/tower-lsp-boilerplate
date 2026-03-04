import { defineConfig } from "rolldown";

const production = process.env.NODE_ENV === "production";

const problemMatcherPlugin = {
  name: "problem-matcher",
  buildStart() {
    console.log("[watch] build started");
  },
  buildEnd() {
    console.log("[watch] build finished");
  },
};

export default defineConfig(({ watch }) => ({
  input: "client/src/extension.ts",
  platform: "node",
  external: ["vscode"],
  plugins: [watch && problemMatcherPlugin].filter(Boolean),
  output: {
    file: "dist/extension.js",
    format: "cjs",
    sourcemap: !production,
    minify: production,
  },
}));
