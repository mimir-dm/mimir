import { defineConfig } from '@playwright/test'

/**
 * UI harness config (MIMIR-I-0074).
 *
 * Expects the stack to already be running:
 *   1. ui-bridge against a scratch DB copy:
 *      MIMIR_BRIDGE_APP_DIR=<scratch> cargo run -p mimir --features ui-harness --bin ui-bridge
 *   2. Vite dev server: npm run dev
 *
 * The browser IPC shim lives in src/harness/bridge-shim.ts (imported by
 * main.ts, dev-only), so specs just navigate and the app talks to the real
 * backend over HTTP.
 */
export default defineConfig({
  testDir: './playwright',
  outputDir: './playwright/results',
  timeout: 60_000,
  use: {
    baseURL: 'http://localhost:5173',
    viewport: { width: 1440, height: 900 },
  },
})
