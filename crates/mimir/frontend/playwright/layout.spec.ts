import { test, expect, type Locator } from '@playwright/test'
import { resolveFixture, type FixtureIds } from './fixture'

/**
 * Layout checks that screenshots cannot assert (COLLIERY-I-0466).
 */

let ids: FixtureIds
test.beforeAll(async () => {
  ids = await resolveFixture()
})

async function box(l: Locator) {
  const b = await l.boundingBox()
  if (!b) throw new Error('not visible')
  return b
}

const overlaps = (a: { x: number; y: number; width: number; height: number }, b: typeof a) =>
  a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height

for (const width of [1400, 768]) {
  test(`campaign dashboard header is one row with no overlap at ${width}px (MIMIR-T-0689)`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 })
    await page.goto(`/campaigns/${ids.campaign}/dashboard/modules`)
    const header = page.locator('.dashboard-header')
    const parts = [header.locator('.dashboard-title'), header.locator('.dashboard-tabs'), header.locator('.header-actions')]
    const boxes = await Promise.all(parts.map(box))
    for (let i = 0; i < boxes.length; i++)
      for (let j = i + 1; j < boxes.length; j++) expect(overlaps(boxes[i], boxes[j]), `parts ${i} and ${j}`).toBe(false)
    // One header instead of a campaign header plus a tab bar.
    const h = (await box(header)).height
    const top = (await box(header)).y
    console.log(`dashboard header at ${width}px: ${Math.round(h)}px tall; content starts at y=${Math.round(top + h)}`)
    if (width >= 1400) expect(h).toBeLessThan(60)
  })
}

test('Cinzel renders with no network to font hosts (MIMIR-T-0694)', async ({ page }) => {
  // The desktop app may be offline: the display face must come from the bundle.
  const external: string[] = []
  await page.route(/fonts\.(googleapis|gstatic)\.com/, (route) => {
    external.push(route.request().url())
    return route.abort()
  })
  await page.goto('/')
  // Load the faces the home title uses; with the font hosts blocked, they
  // can only come from the bundle.
  const loaded = await page.evaluate(async () =>
    (await document.fonts.load('700 24px Cinzel', 'Mimir')).map((f) => f.status),
  )
  expect(loaded.length).toBeGreaterThan(0)
  expect(loaded.every((s) => s === 'loaded')).toBe(true)
  expect(external).toEqual([])
})
