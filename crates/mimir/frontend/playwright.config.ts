import { defineConfig } from '@playwright/test'

/**
 * UI harness config (MIMIR-I-0074).
 *
 * `webServer` starts the whole stack automatically:
 *   1. scripts/ui-session.sh — snapshots the production DB to a scratch dir,
 *      runs ui-bridge against the copy, tears the scratch down on exit
 *   2. the Vite dev server
 * Already-running servers are reused, so for local iteration you can keep
 * both up and re-run specs instantly.
 *
 * The browser IPC shim lives in src/harness/bridge-shim.ts (imported by the
 * window entrypoints, dev-only), so specs just navigate and the app talks to
 * the real backend over HTTP.
 *
 * SCREENSHOT_DIR overrides where capture specs write (used by
 * `npm run screenshots` to produce timestamped run directories).
 */
export default defineConfig({
  testDir: './playwright',
  outputDir: './playwright/results',
  timeout: 60_000,
  workers: 1,
  use: {
    baseURL: 'http://localhost:5173',
    // Match the app's default window size (tauri.conf.json: 1400x900)
    viewport: { width: 1400, height: 900 },
  },
  webServer: [
    {
      command: 'bash ../../../scripts/ui-session.sh',
      url: 'http://127.0.0.1:4175/health',
      reuseExistingServer: true,
      timeout: 300_000, // first run may compile the bridge
    },
    {
      command: 'npm run dev',
      url: 'http://localhost:5173',
      reuseExistingServer: true,
      timeout: 60_000,
    },
  ],
})
