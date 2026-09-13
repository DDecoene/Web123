// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

import { defineConfig } from 'vitest/config';

export default defineConfig({
  server: { port: 5275 },
  test: {
    passWithNoTests: true,
    exclude: ['tests/e2e/**', '**/node_modules/**'],
  },
});
