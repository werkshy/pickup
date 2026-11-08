const { defineConfig } = require("@playwright/test");

/**
 * Browser tests for the web UI. The Rust server is started (and awaited)
 * automatically; `reuseExistingServer` lets a `cargo run -- serve` that is
 * already running on the port be used as-is.
 */
module.exports = defineConfig({
  testDir: ".",
  timeout: 30_000,
  retries: 0,
  use: {
    baseURL: "http://localhost:9394",
    // The browser player needs no gesture prompt in tests, and silence is
    // kinder to whoever is running them.
    launchOptions: {
      args: ["--autoplay-policy=no-user-gesture-required", "--mute-audio"],
    },
  },
  webServer: {
    command: "cargo run -- serve --port 9394",
    url: "http://localhost:9394/hello",
    reuseExistingServer: true,
    timeout: 120_000,
    cwd: "..",
  },
});
