import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./browser-smoke",
  outputDir: process.env.PLAYWRIGHT_OUTPUT_DIR ?? "../test-results",
  fullyParallel: false,
  use: { baseURL: "http://127.0.0.1:4173" },
  projects: [
    { name: "chromium", use: { browserName: "chromium" } },
    { name: "firefox", use: { browserName: "firefox" } },
    { name: "webkit", use: { browserName: "webkit" } },
  ],
  webServer: {
    command: "node browser-smoke/server.mjs",
    url: "http://127.0.0.1:4173/health",
    reuseExistingServer: false,
  },
});
