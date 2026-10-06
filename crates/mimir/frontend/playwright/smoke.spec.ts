import { test, expect } from '@playwright/test'
import { FIXTURE } from './fixture'

/**
 * Smoke: the app boots in the browser, talks to the real backend through the
 * bridge, and shows the fixture campaign.
 */
test('app loads with the fixture campaign', async ({ page }) => {
  await page.goto('/')
  await page.waitForLoadState('networkidle')
  await expect(page.getByText(FIXTURE.campaign).first()).toBeVisible()
  await page.screenshot({ path: 'playwright/screenshots/smoke.png' })
})
