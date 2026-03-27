import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    include: ["src/**/*.test.{ts,tsx}"],
    coverage: {
      provider: "v8",
      reporter: ["text", "text-summary"],
      include: ["src/**/*.{ts,tsx}"],
      exclude: [
        "src/**/*.test.{ts,tsx}",
        "src/test/**",
        "src/__mocks__/**",
        "src/main.tsx",
        "src/vite-env.d.ts",
        "src/lib/i18n/**",
        "src/pages/**",
        "src/App.tsx",
        "src/stores/vehicle.ts",
        "src/stores/vehicleTypes.ts",
        "src/hooks/useConnectionEffects.ts",
        "src/components/advanced/types.ts",
      ],
    },
  },
});
