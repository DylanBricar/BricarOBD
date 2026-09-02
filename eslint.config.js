import js from "@eslint/js";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";

export default tseslint.config(
  { ignores: ["dist", "src-tauri", "node_modules"] },
  {
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    files: ["**/*.{ts,tsx}"],
    plugins: {
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      // These compiler-oriented rules reject deliberate synchronization effects
      // used by this Tauri application. The core Rules of Hooks remain enabled.
      "react-hooks/set-state-in-effect": "off",
      "react-hooks/preserve-manual-memoization": "off",
      // React Compiler is not enabled; TanStack Virtual deliberately returns
      // functions that the compiler cannot memoize safely.
      "react-hooks/incompatible-library": "off",
      "react-refresh/only-export-components": ["warn", { allowConstantExport: true }],
      "no-empty": ["error", { allowEmptyCatch: true }],
      "@typescript-eslint/no-unused-vars": ["warn", { argsIgnorePattern: "^_" }],
      "@typescript-eslint/no-explicit-any": "warn",
    },
  },
  {
    files: ["**/*.test.{ts,tsx}"],
    rules: {
      // Test doubles for overloaded browser/Tauri APIs intentionally use loose
      // boundaries; production source remains subject to no-explicit-any.
      "@typescript-eslint/no-explicit-any": "off",
    },
  }
);
