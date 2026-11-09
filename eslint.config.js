import parser from "@typescript-eslint/parser";
import sonarjs from "eslint-plugin-sonarjs";
const rules = {
  "sonarjs/cognitive-complexity": ["error", 5],
  "constructor-super": "error",
  "no-async-promise-executor": "error",
  "no-constant-condition": "error",
  "no-debugger": "error",
  "no-dupe-args": "error",
  "no-duplicate-case": "error",
  "no-self-assign": "error",
  "no-unreachable": "error",
  "use-isnan": "error",
  "valid-typeof": "error",
  eqeqeq: ["error", "always"],
  "prefer-const": "error",
  "no-var": "error",
};
export default [
  {
    ignores: [
      "dist/**",
      "src-tauri/target/**",
      "artifacts/**",
      "src-tauri/gen/**",
    ],
  },
  {
    files: ["src/**/*.{ts,tsx}", "tests/**/*.{ts,tsx}", "*.{ts,js}"],
    languageOptions: {
      parser,
      parserOptions: {
        ecmaVersion: "latest",
        sourceType: "module",
        ecmaFeatures: { jsx: true },
      },
    },
    plugins: { sonarjs },
    rules,
  },
  {
    files: ["tests/**/*.cjs"],
    languageOptions: { ecmaVersion: "latest", sourceType: "commonjs" },
    plugins: { sonarjs },
    rules,
  },
];
