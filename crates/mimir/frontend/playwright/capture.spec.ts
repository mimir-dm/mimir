import { test } from '@playwright/test'

/**
 * Design-review capture set: walks the main screens with real campaign data
 * and screenshots each to playwright/screenshots/.
 *
 * IDs reference The Frost Architect campaign in the scratch DB snapshot.
 */

const CAMPAIGN = 'cf8be92a-266e-4d40-b108-a42e2bb30f74'
const MODULE_M04 = 'b2003fd9-e0ce-4447-b0f6-0767dbbaf68c' // Caverns of Echo (roster)
const MODULE_M05 = 'd39397f4-a23e-4d98-8df2-1c1d339eb7da' // Dwarven Fortress (has map)
const PC_MATRIM = 'ac345851-d6c3-4344-ae97-1fb15d3f073e'
const PC_MAJOR = 'cb5da3b1-73e2-4ad9-870e-1ce5633dcb7b' // cleric: spells + inventory

const SCREENS: Array<{ name: string; path: string }> = [
  { name: 'home', path: '/' },
  { name: 'campaign-list', path: '/campaigns' },
  { name: 'dashboard-overview', path: `/campaigns/${CAMPAIGN}/dashboard` },
  { name: 'dashboard-campaign-docs', path: `/campaigns/${CAMPAIGN}/dashboard/campaign` },
  { name: 'dashboard-modules', path: `/campaigns/${CAMPAIGN}/dashboard/modules` },
  { name: 'dashboard-npcs', path: `/campaigns/${CAMPAIGN}/dashboard/npcs` },
  { name: 'dashboard-pcs', path: `/campaigns/${CAMPAIGN}/dashboard/pcs` },
  { name: 'dashboard-homebrew', path: `/campaigns/${CAMPAIGN}/dashboard/homebrew` },
  { name: 'module-m04', path: `/modules/${MODULE_M04}` },
  { name: 'module-m05', path: `/modules/${MODULE_M05}` },
  { name: 'characters', path: '/characters' },
  { name: 'character-matrim', path: `/characters/${PC_MATRIM}` },
  { name: 'character-major', path: `/characters/${PC_MAJOR}` },
  { name: 'settings', path: '/settings' },
]

for (const screen of SCREENS) {
  test(`capture ${screen.name}`, async ({ page }) => {
    const errors: string[] = []
    page.on('pageerror', (err) => errors.push(String(err)))
    await page.goto(screen.path)
    await page.waitForLoadState('networkidle')
    await page.waitForTimeout(500)
    await page.screenshot({
      path: `playwright/screenshots/${screen.name}.png`,
      fullPage: true,
    })
    if (errors.length) {
      console.warn(`[${screen.name}] page errors:\n  ${errors.join('\n  ')}`)
    }
  })
}
