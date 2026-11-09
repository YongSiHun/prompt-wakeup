import parser from "@typescript-eslint/parser";
import sonarjs from "eslint-plugin-sonarjs";
const complexity = { "sonarjs/cognitive-complexity": ["error", 5] };
export default [
  { ignores: ["dist/**", "src-tauri/target/**", "artifacts/**"] },
  {
    files: ["src/**/*.{ts,tsx}", "tests/**/*.{ts,tsx}", "*.ts"],
    languageOptions: {
      parser,
      parserOptions: {
        ecmaVersion: "latest",
        sourceType: "module",
        ecmaFeatures: { jsx: true },
      },
    },
    plugins: { sonarjs },
    rules: complexity,
  },
  {
    files: ["tests/**/*.cjs"],
    languageOptions: { ecmaVersion: "latest", sourceType: "commonjs" },
    plugins: { sonarjs },
    rules: complexity,
  },
];
