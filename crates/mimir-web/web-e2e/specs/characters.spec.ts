import { CAMPAIGN, TOKEN, campaignId, expect, signIn, test } from './session'

// Character screens (MIMIR-T-0717): the DM's sheet, inventory, spells and
// level-up; a player's own sheet, and the DM seeing the player's edit live.

const auth = { Authorization: `Bearer ${TOKEN}` }

test('the DM works a sheet: inventory, spells, level-up', async ({ page }) => {
  await signIn(page)
  await page.goto('/')
  await page.getByText(CAMPAIGN).first().click()
  await page.getByRole('tab', { name: 'PCs', exact: true }).click()
  await page.getByRole('link', { name: 'Elara Moonwhisper', exact: true }).click()
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
  let swapped = false
  for (let i = 0; i < 8; i++) {
    const next = dialog.getByRole('button', { name: 'Next' })
    if (!(await next.isVisible())) break
    const here = (await dialog.locator('.mimir-levelup__here').innerText()).trim()
    if (here === 'Subclass') await dialog.getByLabel('Subclass').selectOption({ index: 1 })
    if (here === 'Ability scores') await dialog.getByLabel('Ability', { exact: true }).selectOption('intelligence')
    // Swap a known spell for another (MIMIR-T-0733).
    if (here === 'Spells' && (await dialog.getByText('Swap a spell (optional)').isVisible())) {
      await dialog.getByLabel('Forget').selectOption({ index: 1 })
      await dialog.getByLabel('Learn instead').selectOption({ index: 1 })
      swapped = true
    }
    await expect(next).toBeEnabled()
    await next.click()
  }
  await expect(dialog.locator('.mimir-levelup__here')).toHaveText('Review')
  expect(swapped).toBe(true)
  await expect(dialog).toContainText('Swap:')
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
  await expect(page.locator('.mimir-inv__list').first()).toBeVisible()

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

test('the sheet names its text; the lists and cards act on a character', async ({ page }) => {
  await signIn(page)
  const campaign = await campaignId(page)
  const pcs = (await (await page.request.get(`/api/v1/campaigns/${campaign}/pcs`, { headers: auth })).json()) as {
    id: string
    name: string
  }[]
  const elara = pcs.find((p) => p.name === 'Elara Moonwhisper')!

  // The dashboard's PCs are cards: View, PDF.
  await page.goto(`/campaigns/${campaign}/pcs`)
  const card = page.getByRole('listitem', { name: 'Elara Moonwhisper' })
  await expect(card).toContainText('Wizard')
  await card.getByRole('button', { name: 'PDF…' }).click()
  await expect(page.getByRole('dialog', { name: 'PDF of the character' })).toBeVisible()
  await page.keyboard.press('Escape')
  await card.getByRole('link', { name: 'View Elara Moonwhisper' }).click()
  await expect(page).toHaveURL(new RegExp(`/characters/${elara.id}$`))

  // Details: the background opens; the class features open to their text.
  await page.getByRole('tab', { name: 'Details' }).click()
  const sheet = await (await page.request.get(`/api/v1/characters/${elara.id}`, { headers: auth })).json()
  if (sheet.background_name) {
    await page.getByRole('button', { name: new RegExp(`^${sheet.background_name} \\(`) }).click()
    await expect(page.getByRole('dialog', { name: sheet.background_name })).toBeVisible()
    await page.keyboard.press('Escape')
  }
  const feature = page.locator('details.mimir-feature').first()
  await expect(feature).toBeVisible()
  await feature.locator('summary').click()
  await expect(feature.locator('.mimir-rendered')).toBeVisible()

  // Spells by level; a name opens the spell.
  await page.getByRole('tab', { name: 'Spells' }).click()
  await expect(page.getByRole('heading', { name: /^(Cantrips|1st level)/ }).first()).toBeVisible()
  const spell = page.locator('.mimir-sheet__list .mimir-link-button').first()
  const spellName = (await spell.innerText()).trim()
  await spell.click()
  await expect(page.getByRole('dialog', { name: spellName })).toBeVisible()
  await page.keyboard.press('Escape')

  // Equipment: Equipped apart; an item with notes; an item opens.
  await page.getByRole('tab', { name: 'Equipment' }).click()
  await expect(page.getByRole('heading', { name: /^Equipped/ })).toBeVisible()
  await page.getByLabel('Item name').fill('dagger')
  await page.getByLabel('Item notes').fill('From the goblin chief')
  await page.getByRole('button', { name: 'Search' }).click()
  await page.locator('.mimir-inv__results li', { hasText: 'Dagger' }).first().getByRole('button', { name: 'Add' }).click()
  const dagger = page.locator('.mimir-inv__item', { hasText: 'From the goblin chief' })
  await expect(dagger).toBeVisible()
  await dagger.locator('.mimir-link-button').click()
  await expect(page.getByRole('dialog', { name: 'Dagger' })).toBeVisible()
  await page.keyboard.press('Escape')
  await dagger.getByTitle('Remove Dagger').click()
  await expect(dagger).toHaveCount(0)

  // The characters page: Edit, PDF and Level up on each character.
  await page.goto('/characters')
  const row = page.locator('tr', { hasText: 'Elara Moonwhisper' })
  await row.getByRole('button', { name: 'PDF…' }).click()
  await expect(page.getByRole('dialog', { name: 'PDF of the character' })).toBeVisible()
  await page.keyboard.press('Escape')
  await row.getByRole('button', { name: 'Level up' }).click()
  const level = page.getByRole('dialog', { name: 'Level up Elara Moonwhisper' })
  await expect(level).toBeVisible()
  await level.getByRole('button', { name: /Cancel|Close/ }).first().click().catch(() => page.keyboard.press('Escape'))
  await page.keyboard.press('Escape')
  await row.getByRole('link', { name: 'Edit Elara Moonwhisper' }).click()
  await expect(page).toHaveURL(new RegExp(`/characters/${elara.id}$`))
})
