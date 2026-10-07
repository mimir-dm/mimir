import { test } from '@playwright/test'
import { resolveFixture, type FixtureIds } from './fixture'
import { SCREENS, VIEWPORTS } from './screens'

/**
 * Full design-review capture set: every screen in screens.ts at every
 * viewport in every theme, named `<screen>--<viewport>--<theme>.png`,
 * against the UI fixture. Screens of windows with a fixed theme (DM map,
 * player display) are captured once, as `<screen>--<viewport>.png`.
 *
 * Output dir: SCREENSHOT_DIR env (used by `npm run screenshots` for
 * timestamped run directories) or playwright/screenshots by default.
 * Themes: THEMES env, comma-separated (default `light`), for example
 * `THEMES=light,dark,hyper`.
 */

const OUT_DIR = process.env.SCREENSHOT_DIR ?? 'playwright/screenshots'
const THEMES = (process.env.THEMES ?? 'light')
  .split(',')
  .map((t) => t.trim())
  .filter(Boolean)

let ids: FixtureIds
test.beforeAll(async () => {
  ids = await resolveFixture()
})

for (const theme of THEMES) {
  for (const viewport of VIEWPORTS) {
    for (const screen of SCREENS) {
      // A fixed-theme window looks the same in every theme: capture it once.
      if (screen.fixedTheme && theme !== THEMES[0]) continue
      const file = screen.fixedTheme
        ? `${screen.name}--${viewport.name}.png`
        : `${screen.name}--${viewport.name}--${theme}.png`
      test(`capture ${file}`, async ({ page }) => {
        // The app reads the theme from localStorage when it starts.
        await page.addInitScript((t) => localStorage.setItem('theme', t), theme)
        await page.setViewportSize({ width: viewport.width, height: viewport.height })
        await page.goto(screen.path(ids))
        await page.waitForLoadState('networkidle')
        if (screen.setup) {
          await screen.setup(page, ids)
          await page.waitForLoadState('networkidle')
        }
        await page.waitForTimeout(500)
        // Animations frozen: two runs of the same code give the same pixels.
        await page.screenshot({ path: `${OUT_DIR}/${file}`, fullPage: true, animations: 'disabled' })
      })
    }
  }
}
