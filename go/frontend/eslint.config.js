import globals from "globals";
import { default as reactPlugin } from "eslint-plugin-react";
import js from "@eslint/js";

export default [
  {
    ...js.configs.recommended,
    ...reactPlugin.configs.flat.recommended, // This is not a plugin object, but a shareable config object
    ...reactPlugin.configs.flat["jsx-runtime"],
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: {
        ...globals.browser,
      },
      parserOptions: {
        ecmaFeatures: {
          jsx: true,
        },
      },
    },
    files: ["src/**/*.{js,jsx}"],
    plugins: {
      reactPlugin,
    },
    settings: {
      react: {
        version: "detect",
      },
    },
    rules: {
      "react/prop-types": [0],
    },
  },
];
