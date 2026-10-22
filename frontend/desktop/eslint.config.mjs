const nodeGlobals = {
  AbortController: "readonly",
  Buffer: "readonly",
  URL: "readonly",
  __dirname: "readonly",
  __filename: "readonly",
  clearInterval: "readonly",
  clearTimeout: "readonly",
  console: "readonly",
  fetch: "readonly",
  module: "readonly",
  process: "readonly",
  require: "readonly",
  setInterval: "readonly",
  setTimeout: "readonly",
  window: "readonly",
};

export default [
  {
    // 第三方代码按原样放着（见 src/main/vendor/cordis/README.md），不按本仓库的规则检查。
    ignores: ["src/main/vendor/**"],
  },
  {
    files: ["main.js", "preload.js", "src/main/**/*.js", "src/main/**/*.mjs", "scripts/run-local.mjs"],
    languageOptions: {
      ecmaVersion: "latest",
      globals: nodeGlobals,
      sourceType: "commonjs",
    },
    rules: {
      "no-undef": "error",
    },
  },
  {
    files: ["src/main/**/*.mjs", "scripts/run-local.mjs"],
    languageOptions: {
      sourceType: "module",
    },
  },
];
