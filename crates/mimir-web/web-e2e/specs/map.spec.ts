import { CAMPAIGN, TOKEN, campaignId, expect, mapId, moduleId, signIn, test } from './session'

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
  await page.request.put(`/api/v1/campaigns/${await campaignId(page)}/display`, {
    headers: auth,
    data: { map_id: null },
  })
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

  // Line of sight is saved with the map (MIMIR-T-0728).
  await expect
    .poll(async () => (await (await page.request.get(`/api/v1/maps/${map}`, { headers: auth })).json()).line_of_sight)
    .toBe(false)
  await page.getByText('Line of sight', { exact: true }).click()
  await expect
    .poll(async () => (await (await page.request.get(`/api/v1/maps/${map}`, { headers: auth })).json()).line_of_sight)
    .toBe(true)

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

test('DM map controls: presets, keys, overlays, full screen, Add PCs, details', async ({ page }) => {
  await signIn(page)
  const campaign = await campaignId(page)
  const map = await mapId(page)
  const tokensOf = async () =>
    (await (await page.request.get(`/api/v1/maps/${map}/tokens`, { headers: auth })).json()) as {
      id: string
      name: string
      token_type: string
      visible_to_players: boolean
      dead: boolean
      vision_bright_ft: number | null
      vision_dark_ft: number
    }[]
  const klargNow = async () => (await tokensOf()).find((t) => t.name === 'Klarg')!

  // Play on the module page opens its first map.
  const module = await moduleId(page, campaign)
  await page.goto(`/modules/${module}`)
  await page.getByRole('link', { name: 'Play', exact: true }).click()
  await expect(page).toHaveURL(new RegExp(`/maps/${map}$`))
  await expect(page.locator('.mimir-map__svg image').first()).toBeAttached()

  // A vision preset.
  await page.locator('.mimir-map__token[aria-label="Klarg"]').click()
  await page.getByLabel('Vision preset').selectOption('Blindsight 30 ft')
  await expect.poll(async () => (await klargNow()).vision_bright_ft).toBe(30)
  expect((await klargNow()).vision_dark_ft).toBe(30)

  // Keys: H hides and shows, D marks dead and alive, Esc deselects.
  const klargToken = page.locator('.mimir-map__token[aria-label="Klarg"]')
  await page.keyboard.press('h')
  await expect(klargToken).toHaveClass(/mimir-map__token--hidden/)
  expect((await klargNow()).visible_to_players).toBe(false)
  await page.keyboard.press('h')
  await expect(klargToken).not.toHaveClass(/mimir-map__token--hidden/)
  expect((await klargNow()).visible_to_players).toBe(true)
  await page.keyboard.press('d')
  await expect(klargToken).toHaveClass(/mimir-map__token--dead/)
  expect((await klargNow()).dead).toBe(true)
  await page.keyboard.press('d')
  await expect(klargToken).not.toHaveClass(/mimir-map__token--dead/)
  expect((await klargNow()).dead).toBe(false)
  await page.keyboard.press('Escape')
  await expect(page.locator('.mimir-map__panel')).toHaveCount(0)
  // + zooms in; 0 fits again.
  const transform = () => page.locator('.mimir-map__svg > g').first().getAttribute('transform')
  const fitted = await transform()
  await page.keyboard.press('+')
  await expect.poll(transform).not.toBe(fitted)
  await page.keyboard.press('0')
  await expect.poll(transform).toBe(fitted)

  // Overlays: the walls and how far Klarg sees (30 ft blindsight).
  await page.getByText('Overlays', { exact: true }).click()
  await expect(page.locator('.mimir-map__walls')).toBeAttached()
  await expect(page.locator('.mimir-map__ring')).not.toHaveCount(0)
  await page.getByText('Overlays', { exact: true }).click()
  await expect(page.locator('.mimir-map__ring')).toHaveCount(0)
  const k = await klargNow()
  await page.request.patch(`/api/v1/tokens/${k.id}`, {
    headers: auth,
    data: { vision_bright_ft: null, vision_dim_ft: null, vision_dark_ft: 0, light_radius_ft: 0 },
  })

  // A double click on a monster token shows its stat block.
  await page.locator('.mimir-map__token[aria-label="Klarg"]').dblclick()
  await expect(page.getByRole('dialog', { name: 'Bugbear' })).toBeVisible()
  await page.keyboard.press('Escape')

  // Full screen.
  await page.getByTitle('Full screen').click()
  await expect.poll(() => page.evaluate(() => document.fullscreenElement?.className ?? '')).toContain('mimir-map')
  await page.evaluate(() => document.exitFullscreen())

  // Add PCs: a token for each PC of the campaign with none on the map.
  const pcs = (await (await page.request.get(`/api/v1/campaigns/${campaign}/pcs`, { headers: auth })).json()) as {
    name: string
  }[]
  expect(pcs.length).toBeGreaterThan(0)
  const beforeTokens = await tokensOf()
  const before = new Set(beforeTokens.map((t) => t.id))
  const missing = pcs
    .map((p) => p.name)
    .filter((n) => !beforeTokens.some((t) => t.token_type === 'pc' && t.name.toLowerCase() === n.toLowerCase()))
  await page.getByRole('button', { name: 'Add PCs' }).click()
  for (const pc of pcs) {
    await expect(page.locator(`.mimir-map__token[aria-label="${pc.name}"]`).first()).toBeAttached()
  }
  // A second click adds none.
  await page.getByRole('button', { name: 'Add PCs' }).click()
  await page.waitForTimeout(300)
  const added = (await tokensOf()).filter((t) => !before.has(t.id))
  expect(added.map((t) => t.name).sort()).toEqual([...missing].sort())
  for (const t of added) await page.request.delete(`/api/v1/tokens/${t.id}`, { headers: auth })
})
