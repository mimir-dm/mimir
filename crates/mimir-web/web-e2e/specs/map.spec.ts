import { CAMPAIGN, TOKEN, expect, mapId, signIn, test } from './session'

// The DM map: open it from the module page, select and drag a token, place
// a PC, fog and line of sight, show to players. Runs in CI with the smoke
// suite. Each test changes the shared session data, so names are unique.

const auth = { Authorization: `Bearer ${TOKEN}` }

test('open a map from the module page and drag a token', async ({ page }) => {
  await signIn(page)
  const map = await mapId(page)
  await page.goto('/')
  await page.getByText(CAMPAIGN).first().click()
  await page.getByRole('tab', { name: 'Modules', exact: true }).click()
  await page.getByRole('link', { name: 'Cragmaw Hideout' }).click()
  await page.getByRole('tab', { name: 'Maps' }).click()
  await page.getByRole('link', { name: 'Goblin Hideout' }).click()
  await expect(page).toHaveURL(new RegExp(`/maps/${map}$`))
  await expect(page.locator('.mimir-map__svg image').first()).toBeAttached()

  const klarg = page.locator('.mimir-map__token[aria-label="Klarg"]')
  const box = (await klarg.boundingBox())!
  const before = (await (await page.request.get(`/api/v1/maps/${map}/tokens`, { headers: auth })).json()) as {
    name: string
    grid_x: number
  }[]
  const x0 = before.find((t) => t.name === 'Klarg')!.grid_x
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await expect(page.locator('.mimir-map__panel')).toContainText('Klarg')
  await page.mouse.move(box.x + box.width * 1.6, box.y + box.height / 2, { steps: 4 })
  await page.mouse.move(box.x + box.width * 2.6, box.y + box.height / 2, { steps: 4 })
  await page.mouse.up()
  await expect
    .poll(async () => {
      const after = (await (await page.request.get(`/api/v1/maps/${map}/tokens`, { headers: auth })).json()) as {
        name: string
        grid_x: number
      }[]
      return after.find((t) => t.name === 'Klarg')!.grid_x
    })
    .toBe(x0 + 2)
})

test('place a PC; fog and line of sight; show to players', async ({ page }) => {
  await signIn(page)
  const map = await mapId(page)
  await page.request.put(`/api/v1/maps/${map}/fog`, { headers: auth, data: { enabled: false } })
  page.on('dialog', (d) => d.accept('Sam'))
  await page.goto(`/maps/${map}`)
  await expect(page.locator('.mimir-map__svg image').first()).toBeAttached()

  await page.getByLabel('Place on the map').selectOption('pc')
  await expect(page.getByText('Tap the map to place it.')).toBeVisible()
  const scene = (await page.locator('.mimir-map__scene').boundingBox())!
  await page.mouse.click(scene.x + 280, scene.y + 420)
  await expect(page.locator('.mimir-map__token[aria-label="Sam"]')).toBeAttached()

  // Fog on: the shade appears, with a hole for the PC's sight.
  await page.getByText('Fog', { exact: true }).click()
  await expect(page.locator('.mimir-map__fog')).toBeAttached()
  await expect(page.locator('#mimir-fog-mask path')).toHaveCount(1)
  const fog = await (await page.request.get(`/api/v1/maps/${map}/fog`, { headers: auth })).json()
  expect(fog.enabled).toBe(true)
  const withLos = (await page.locator('#mimir-fog-mask path').getAttribute('d'))!
  // Line of sight off: the sight is no longer cut by walls.
  await page.getByText('Line of sight', { exact: true }).click()
  await expect
    .poll(async () => (await page.locator('#mimir-fog-mask path').getAttribute('d'))!.length)
    .toBeLessThan(withLos.length)

  await page.getByRole('button', { name: 'Show to players' }).click()
  await expect(page.getByRole('button', { name: 'Stop showing' })).toBeVisible()
  await page.getByRole('button', { name: 'Blackout' }).click()
  await expect(page.getByRole('button', { name: 'End blackout' })).toBeVisible()
  await page.getByRole('button', { name: 'End blackout' }).click()
  await page.getByRole('button', { name: 'Stop showing' }).click()
  await expect(page.getByRole('button', { name: 'Show to players' })).toBeVisible()
})

test('a change from another client reaches the map', async ({ page, request }) => {
  await signIn(page)
  const map = await mapId(page)
  await page.goto(`/maps/${map}`)
  await expect(page.locator('.mimir-map__svg image').first()).toBeAttached()
  // Give the socket a moment to watch the campaign.
  await page.waitForTimeout(500)
  await request.post(`/api/v1/maps/${map}/pois`, {
    headers: auth,
    data: { name: 'Hidden shrine', grid_x: 3, grid_y: 3 },
  })
  await expect(page.locator('.mimir-map__marker--poi', { hasText: 'Hidden shrine' })).toBeAttached()
})
