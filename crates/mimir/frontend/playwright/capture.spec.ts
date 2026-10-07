import { test } from '@playwright/test'
import { resolveFixture, type FixtureIds } from './fixture'
import { SCREENS, VIEWPORTS } from './screens'

/**
 * Full design-review capture set: every screen in screens.ts at every
 * viewport, named `<screen>--<viewport>.png`, against the UI fixture.
 *
 * Output dir: SCREENSHOT_DIR env (used by `npm run screenshots` for
 * timestamped run directories) or playwright/screenshots by default.
 */

const OUT_DIR = process.env.SCREENSHOT_DIR ?? 'playwright/screenshots'

let ids: FixtureIds
test.beforeAll(async () => {
  ids = await resolveFixture()
})

for (const viewport of VIEWPORTS) {
  for (const screen of SCREENS) {
    test(`capture ${screen.name}--${viewport.name}`, async ({ page }) => {
      await page.setViewportSize({ width: viewport.width, height: viewport.height })
      await page.goto(screen.path(ids))
      await page.waitForLoadState('networkidle')
      if (screen.setup) {
        await screen.setup(page, ids)
        await page.waitForLoadState('networkidle')
      }
      await page.waitForTimeout(500)
      await page.screenshot({
        path: `${OUT_DIR}/${screen.name}--${viewport.name}.png`,
        fullPage: true,
      })
    })
  }
}
