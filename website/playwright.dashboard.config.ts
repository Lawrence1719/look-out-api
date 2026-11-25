import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "./tests",
  testMatch: "dashboard.spec.ts",
  workers: 1,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: "list",
  use: { baseURL: "http://127.0.0.1:4181", trace: "retain-on-failure" },
  webServer: {
    command: "cargo run --locked --example dashboard_fixture",
    cwd: "..",
    env: {
      LOOK_OUT_API_HTTP_PROXY: "",
      HTTP_PROXY: "",
      HTTPS_PROXY: "",
      ALL_PROXY: "",
      http_proxy: "",
      https_proxy: "",
      all_proxy: "",
      NO_PROXY: "127.0.0.1,localhost,::1",
    },
    url: "http://127.0.0.1:4181/healthz",
    timeout: 300_000,
    reuseExistingServer: false,
    stdout: "ignore",
    stderr: "pipe",
  },
});
