import { CAMPAIGN, TOKEN, campaignId, expect, signIn, test } from './session'

// Character screens (MIMIR-T-0717): the DM's sheet, inventory, spells and
// level-up; a player's own sheet, and the DM seeing the player's edit live.

const auth = { Authorization: `Bearer ${TOKEN}` }

test('the DM works a sheet: inventory, spells, level-up', async ({ page }) => {
  await signIn(page)
  await page.goto('/')
  await page.getByText(CAMPAIGN).first().click()
  await page.getByRole('tab', { name: 'PCs', exact: true }).click()
  await page.getByRole('link', { name: 'Elara Moonwhisper' }).click()
  await expect(page.getByRole('heading', { name: 'Elara Moonwhisper' })).toBeVisible()
  await expect(page.locator('.mimir-sheet__stat', { hasText: 'Armor class' })).toBeVisible()
  await expect(page.locator('.mimir-sheet__ability')).toHaveCount(6)
  const header = await page.locator('.cl-page-header').first().innerText()
  const level = Number(/Level (\d+)/.exec(header)![1])

  // Inventory: add a shortsword, equip it: it shows as an attack.
  await page.getByRole('tab', { name: 'Equipment' }).click()
  await page.getByLabel('Item name').fill('shortsword')
  await page.getByRole('button', { name: 'Search' }).click()
  await page.locator('.mimir-inv__results li', { hasText: 'Shortsword' }).getByRole('button', { name: 'Add' }).click()
  const row = page.locator('.mimir-inv__item', { hasText: 'Shortsword' }).first()
  await expect(row).toBeVisible()
  await row.getByText('Equipped').click()
  await page.getByRole('tab', { name: 'Character' }).click()
  await expect(page.locator('.mimir-sheet__list li', { hasText: 'Shortsword' })).toBeVisible()
  await page.getByRole('tab', { name: 'Equipment' }).click()
  await page.locator('.mimir-inv__item', { hasText: 'Shortsword' }).first().getByTitle('Remove Shortsword').click()
  await expect(page.locator('.mimir-inv__item', { hasText: 'Shortsword' })).toHaveCount(0)

  // Coins.
  await page.getByLabel('GP').fill('77')
  await page.getByRole('button', { name: 'Save coins' }).click()
  const id = new URL(page.url()).pathname.split('/').pop()
  await expect
    .poll(async () => (await (await page.request.get(`/api/v1/characters/${id}`, { headers: auth })).json()).currency.gp)
    .toBe(77)

  // Spells: learn one from the list.
  await page.getByRole('tab', { name: 'Spells' }).click()
  const before = await page.locator('.mimir-sheet__list li').count()
  await page.getByLabel('Spell to learn').selectOption({ index: 1 })
  await page.getByRole('button', { name: 'Learn' }).click()
  await expect(page.locator('.mimir-sheet__list li')).toHaveCount(before + 1)

  // Level up through the dialog.
  await page.getByRole('button', { name: 'Level up' }).click()
  const dialog = page.getByRole('dialog')
  await expect(dialog).toBeVisible()
  for (let i = 0; i < 8; i++) {
    const next = dialog.getByRole('button', { name: 'Next' })
    if (!(await next.isVisible())) break
    const here = (await dialog.locator('.mimir-levelup__here').innerText()).trim()
    if (here === 'Subclass') await dialog.getByLabel('Subclass').selectOption({ index: 1 })
    if (here === 'Ability scores') await dialog.getByLabel('Ability', { exact: true }).selectOption('intelligence')
    await expect(next).toBeEnabled()
    await next.click()
  }
  await expect(dialog.locator('.mimir-levelup__here')).toHaveText('Review')
  await dialog.getByRole('button', { name: 'Level up' }).click()
  await expect(dialog.getByText('Level up done')).toBeVisible()
  await expect(dialog).toContainText(`Now level ${level + 1}`)
  await dialog.getByRole('button', { name: 'Done' }).click()
  await expect(page.locator('.cl-page-header').first()).toContainText(`Level ${level + 1}`)
})

test('a player edits their own sheet; the DM sees it live', async ({ page, browser }) => {
  await signIn(page)
  const campaign = await campaignId(page)
  const pcs = (await (await page.request.get(`/api/v1/campaigns/${campaign}/pcs`, { headers: auth })).json()) as {
    id: string
    name: string
  }[]
  const thorin = pcs.find((p) => p.name === 'Thorin Ironforge')!
  const link = await (await page.request.post(`/api/v1/characters/${thorin.id}/link`, { headers: auth })).json()

  // The DM has Thorin's sheet open.
  await page.goto(`/characters/${thorin.id}`)
  await page.getByRole('tab', { name: 'Equipment' }).click()
  await expect(page.locator('.mimir-inv__list')).toBeVisible()

  // The player opens their link and the Character tab.
  const device = await browser.newContext()
  const player = await device.newPage()
  await player.goto(link.path)
  await player.getByRole('tab', { name: 'Character' }).click()
  await expect(player.getByRole('heading', { name: 'Thorin Ironforge' })).toBeVisible()
  await player.getByRole('tab', { name: 'Equipment' }).click()
  await player.getByLabel('Item name').fill('greatsword')
  await player.getByRole('button', { name: 'Search' }).click()
  await player.locator('.mimir-inv__results li', { hasText: 'Greatsword' }).getByRole('button', { name: 'Add' }).click()
  await expect(player.locator('.mimir-inv__item', { hasText: 'Greatsword' })).toBeVisible()

  // Live on the DM's screen.
  await expect(page.locator('.mimir-inv__item', { hasText: 'Greatsword' })).toBeVisible()

  await page.request.delete(`/api/v1/characters/${thorin.id}/link`, { headers: auth })
  await device.close()
})
