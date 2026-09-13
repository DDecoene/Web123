import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  webServer: {
    command: 'npm run dev -- --port 5275',
    port: 5275,
    reuseExistingServer: !process.env.CI,
  },
  use: {
    baseURL: 'http://localhost:5275',
  },
});
