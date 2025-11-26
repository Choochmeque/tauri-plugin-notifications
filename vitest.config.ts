import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    globals: true,
    environment: "node",
    include: ["**/*.test.ts"],
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html"],
      include: ["guest-js/**/*.ts"],
      exclude: ["guest-js/**/*.test.ts"],
    },
  },
});
