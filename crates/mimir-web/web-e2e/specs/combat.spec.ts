import { TOKEN, campaignId, expect, mapId, signIn, test } from './session'

// The initiative tracker beside the DM map, and the turn order on the
// player display. Runs in CI with the smoke suite.

const auth = { Authorization: `Bearer ${TOKEN}` }

async function cleanCombat(page: import('@playwright/test').Page, module: string) {
  const c = await (await page.request.get(`/api/v1/modules/${module}/combat`, { headers: auth })).json()
  if (c) await page.request.delete(`/api/v1/combat/${c.session.id}`, { headers: auth })
}

test('run a combat from the tracker and show the order to players', async ({ page, context }) => {
  await signIn(page)
  const campaign = await campaignId(page)
  const map = await mapId(page)
  const detail = await (await page.request.get(`/api/v1/maps/${map}`, { headers: auth })).json()
  await cleanCombat(page, detail.module_id)
  await page.request.put(`/api/v1/campaigns/${campaign}/display`, { headers: auth, data: { map_id: null } })

  await page.goto(`/maps/${map}`)
  await expect(page.locator('.mimir-map__svg image').first()).toBeAttached()
  await page.getByRole('button', { name: 'Start combat' }).click()
  await expect(page.locator('.mimir-tracker').getByText('Round 1')).toBeVisible()

  // Add Klarg (a monster group) and a custom entry.
  const add = page.getByLabel('Add to the combat')
  const klarg = (await add.locator('option').allTextContents()).find((o) => o.startsWith('Klarg'))!
  await add.selectOption({ label: klarg })
  await page.locator('.mimir-tracker__add').first().getByRole('button', { name: 'Add' }).click()
  await expect(page.locator('.mimir-tracker__row', { hasText: 'Klarg' })).toBeVisible()
  await page.getByPlaceholder('Custom name').fill('Sam')
  await page.getByPlaceholder('Custom name').press('Enter')
  await expect(page.locator('.mimir-tracker__row', { hasText: 'Sam' })).toBeVisible()
  // Klarg is linked to his token.
  await expect(page.locator('.mimir-tracker__row', { hasText: 'Klarg' }).locator('.mimir-tracker__linked')).toBeVisible()

  // Initiative orders the rows.
  await page.getByLabel('Initiative of Sam').fill('5')
  await page.getByLabel('Initiative of Sam').press('Tab')
  await page.getByLabel('Initiative of Klarg').fill('18')
  await page.getByLabel('Initiative of Klarg').press('Tab')
  await expect(page.locator('.mimir-tracker__row').first()).toContainText('Klarg')
  // Klarg's turn: his token is marked on the map.
  await expect(page.locator('.mimir-map__token--turn[aria-label="Klarg"]')).toBeAttached()

  // Sam: HP, concentration, damage → the save prompt; Lost ends it.
  await page.locator('.mimir-tracker__name', { hasText: 'Sam' }).click()
  await page.getByPlaceholder('Set HP').fill('20')
  await page.getByPlaceholder('Set HP').press('Enter')
  await expect(page.locator('.mimir-tracker__row', { hasText: 'Sam' }).locator('.mimir-tracker__hp')).toHaveText('20/20')
  await page.getByText('Concentrating', { exact: true }).click()
  await expect(page.locator('.mimir-tracker__row', { hasText: 'Sam' }).getByText('Conc.')).toBeVisible()
  await page.getByPlaceholder('Amount').fill('8')
  await page.getByPlaceholder('Amount').press('Enter')
  await expect(page.getByText('Concentration save DC 10')).toBeVisible()
  await page.getByRole('button', { name: 'Lost' }).click()
  await expect(page.locator('.mimir-tracker__row', { hasText: 'Sam' }).getByText('Conc.')).toHaveCount(0)
  await expect(page.locator('.mimir-tracker__row', { hasText: 'Sam' }).locator('.mimir-tracker__hp')).toHaveText('12/20')
  await expect(page.getByLabel('Recent HP changes')).toContainText('−8 (round 1)')

  // A condition with a duration; a click on its pill removes it.
  await page.getByLabel('Condition').selectOption('poisoned')
  await page.getByPlaceholder('Rnds').fill('2')
  await page.getByPlaceholder('Rnds').press('Enter')
  const pill = page.locator('.mimir-tracker__condition', { hasText: 'poisoned' })
  await expect(pill).toHaveAttribute('title', 'poisoned until the end of round 2')
  await pill.click()
  await expect(pill).toHaveCount(0)

  // Next turn → Sam; the display shows the order (names only).
  await page.getByRole('button', { name: 'Next turn ›' }).click()
  await expect(page.locator('.mimir-tracker__row--current')).toContainText('Sam')
  await page.getByRole('button', { name: 'Show to players' }).click()
  await page.getByText('Show order to players', { exact: true }).click()
  const display = await context.newPage()
  await signIn(display)
  await display.goto(`/display/${campaign}`)
  const order = display.locator('.mimir-display__order')
  await expect(order).toContainText('Klarg')
  await expect(order.locator('li[aria-current="step"]')).toHaveText('Sam')
  await expect(order).not.toContainText('20')
  // The order hidden again: it leaves the display.
  await page.getByText('Show order to players', { exact: true }).click()
  await expect(order).toHaveCount(0)

  // End the combat (asks first).
  await page.getByRole('button', { name: 'End combat' }).click()
  await page.getByRole('button', { name: 'End', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Start combat' })).toBeVisible()
  await page.request.put(`/api/v1/campaigns/${campaign}/display`, { headers: auth, data: { map_id: null } })
})
