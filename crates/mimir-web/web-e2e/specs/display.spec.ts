import { TOKEN, campaignId, expect, mapId, signIn, test } from './session'

// The player display, driven by the DM page: it follows the DM live and
// never shows hidden data. Runs in CI with the smoke suite.

const auth = { Authorization: `Bearer ${TOKEN}` }

test('the display follows the DM and shows only what players see', async ({ page, context }) => {
  await signIn(page)
  const campaign = await campaignId(page)
  const map = await mapId(page)
  await page.request.put(`/api/v1/campaigns/${campaign}/display`, { headers: auth, data: { map_id: null } })
  await page.request.put(`/api/v1/maps/${map}/fog`, { headers: auth, data: { enabled: false } })

  const display = await context.newPage()
  await signIn(display)
  await display.goto(`/display/${campaign}`)
  await expect(display.getByText('Waiting for the DM to show a map.')).toBeVisible()
  // No app shell on the display.
  await expect(display.locator('.cl-appshell')).toHaveCount(0)

  // The DM shows the map from the map page.
  await page.goto(`/maps/${map}`)
  await page.getByRole('button', { name: 'Show to players' }).click()
  await expect(display.locator('.mimir-display__svg image')).toBeAttached()
  await expect(display.locator('.mimir-map__token[aria-label="Klarg"]')).toBeAttached()

  // The DM hides Klarg: he leaves the display.
  const tokens = (await (await page.request.get(`/api/v1/maps/${map}/tokens`, { headers: auth })).json()) as {
    id: string
    name: string
  }[]
  const klarg = tokens.find((t) => t.name === 'Klarg')!
  await page.request.patch(`/api/v1/tokens/${klarg.id}`, { headers: auth, data: { hidden: true } })
  await expect(display.locator('.mimir-map__token[aria-label="Klarg"]')).toHaveCount(0)
  await page.request.patch(`/api/v1/tokens/${klarg.id}`, { headers: auth, data: { hidden: false } })
  await expect(display.locator('.mimir-map__token[aria-label="Klarg"]')).toBeAttached()

  // A shown trap: its name, never its details. A hidden trap: nothing.
  await page.request.post(`/api/v1/maps/${map}/traps`, {
    headers: auth,
    data: { name: 'Tripwire', grid_x: 6, grid_y: 6, visible: true, effect_description: 'SECRET-EFFECT', dc: 15 },
  })
  await page.request.post(`/api/v1/maps/${map}/traps`, {
    headers: auth,
    data: { name: 'Hidden pit', grid_x: 7, grid_y: 7 },
  })
  await expect(display.locator('.mimir-map__marker[aria-label="Tripwire"]')).toBeAttached()
  await expect(display.locator('.mimir-map__marker[aria-label="Hidden pit"]')).toHaveCount(0)
  expect(await display.content()).not.toContain('SECRET-EFFECT')

  // Fog on with no PCs: the map is covered.
  await page.getByText('Fog', { exact: true }).click()
  await expect(display.locator('.mimir-display__cover')).toBeAttached()

  // Blackout pauses the display; ending it brings the map back.
  await page.getByRole('button', { name: 'Blackout' }).click()
  await expect(display.getByText('Paused')).toBeVisible()
  await page.getByRole('button', { name: 'End blackout' }).click()
  await expect(display.locator('.mimir-display__svg image')).toBeAttached()

  // Stop showing: the display waits again.
  await page.getByRole('button', { name: 'Stop showing' }).click()
  await expect(display.getByText('Waiting for the DM to show a map.')).toBeVisible()
  await page.request.put(`/api/v1/maps/${map}/fog`, { headers: auth, data: { enabled: false } })
})

test('the display pans and zooms for the viewer', async ({ page, context }) => {
  await signIn(page)
  const campaign = await campaignId(page)
  const map = await mapId(page)
  await page.request.put(`/api/v1/campaigns/${campaign}/display`, { headers: auth, data: { map_id: map } })
  const display = await context.newPage()
  await signIn(display)
  await display.goto(`/display/${campaign}`)
  await expect(display.locator('.mimir-display__svg image')).toBeAttached()
  const g = display.locator('.mimir-display__svg > g')
  const before = await g.getAttribute('transform')
  await display.getByTitle('Zoom in').click()
  await expect.poll(() => g.getAttribute('transform')).not.toBe(before)
  const box = (await display.locator('.mimir-display__scene').boundingBox())!
  const zoomed = await g.getAttribute('transform')
  await display.mouse.move(box.x + 200, box.y + 200)
  await display.mouse.down()
  await display.mouse.move(box.x + 300, box.y + 260, { steps: 4 })
  await display.mouse.up()
  await expect.poll(() => g.getAttribute('transform')).not.toBe(zoomed)
  await page.request.put(`/api/v1/campaigns/${campaign}/display`, { headers: auth, data: { map_id: null } })
})
