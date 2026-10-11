import { defineConfig } from '@playwright/test'

/**
 * mimir-web e2e (MIMIR-T-0710). `webServer` runs session.sh: mimir-server on
 * a scratch data dir seeded with the UI fixture, serving the built app
 * (crates/mimir-web/dist), in token mode. The scratch dir is removed on
 * exit. A server already up on the port is reused.
 *
 * SCREENSHOT_DIR sets where capture.spec.ts writes.
 */
const port = Number(process.env.MIMIR_E2E_PORT ?? 8790)

export default defineConfig({
  testDir: './specs',
  outputDir: './results',
  timeout: 60_000,
  workers: 1,
  reporter: process.env.CI ? 'line' : 'list',
  use: {
    baseURL: `http://127.0.0.1:${port}`,
    viewport: { width: 1280, height: 800 },
  },
  webServer: {
    command: 'bash session.sh',
    url: `http://127.0.0.1:${port}/readyz`,
    reuseExistingServer: !process.env.CI,
    timeout: 600_000, // the first run compiles the server
    // SIGTERM so session.sh removes the scratch dir.
    gracefulShutdown: { signal: 'SIGTERM', timeout: 10_000 },
  },
})
