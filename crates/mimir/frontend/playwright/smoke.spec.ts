import { test, expect } from '@playwright/test'

/**
 * Smoke: the app boots in the browser, talks to the real backend through the
 * bridge, and shows real campaign data.
 */
test('app loads with real campaign data', async ({ page }) => {
  await page.goto('/')
  await page.waitForLoadState('networkidle')
  await expect(page.getByText('The Frost Architect').first()).toBeVisible()
  await page.screenshot({ path: 'playwright/screenshots/smoke.png' })
})
