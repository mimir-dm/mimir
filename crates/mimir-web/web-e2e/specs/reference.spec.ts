import { campaignId, expect, moduleId, signIn, test } from './session'

// The reference (MIMIR-T-0723): the catalog (search, filters, detail),
// cross-references opening over each other, the book reader (contents,
// headings, images, tooltips), adventures in the reader and homebrew files
// (MIMIR-T-0736), and the module page's inspector.

test('the catalog: search, filter, detail, cross-references', async ({ page }) => {
  await signIn(page)
  await page.goto('/reference')
  await expect(page.getByRole('link', { name: 'Spells' })).toHaveAttribute('aria-current', 'page')
  const results = page.getByRole('table', { name: 'Results' })
  await page.getByLabel('Name').fill('fireball')
  const row = results.locator('tbody tr', { hasText: 'Fireball' })
  await expect(row).toContainText('3rd')
  await expect(row).toContainText('Evocation')
  await row.getByRole('button', { name: 'Fireball' }).click()
  const drawer = page.getByRole('dialog', { name: 'Fireball' })
  await expect(drawer).toContainText('3rd-level evocation')
  await expect(drawer).toContainText('150 feet')
  await drawer.getByRole('button', { name: 'Close' }).click()

  // A filter: cantrips only.
  await page.getByLabel('Name').fill('')
  await page.getByLabel('Level').selectOption('0')
  await expect(results.locator('tbody tr').first()).toContainText('Cantrip')
  const levels = await results.locator('tbody tr td:nth-child(2)').allInnerTexts()
  expect(levels.every((l) => l === 'Cantrip')).toBe(true)

  // The bestiary: a stat block, and a cross-reference in it opens over it.
  await page.getByRole('link', { name: 'Bestiary' }).click()
  await page.getByLabel('Name').fill('goblin')
  await results.getByRole('button', { name: 'Goblin', exact: true }).click()
  const goblin = page.getByRole('dialog', { name: 'Goblin' })
  await expect(goblin).toContainText('CR 1/4')
  await expect(goblin.locator('.detail-abilities')).toBeVisible()
  const link = goblin.locator('a.cross-ref-link').first()
  const target = (await link.getAttribute('data-ref-name'))!
  await link.click()
  await expect(page.getByRole('dialog')).toContainText(target, { ignoreCase: true })
  await page.getByRole('dialog').getByRole('button', { name: '← Back' }).click()
  await expect(page.getByRole('dialog', { name: 'Goblin' })).toBeVisible()
})

test('the reader: contents, headings, images, tooltips', async ({ page }) => {
  await signIn(page)
  await page.goto('/reference?mode=reader')
  const contents = page.getByRole('navigation', { name: 'Contents' })
  await expect(contents.getByRole('link', { name: 'Introduction' })).toHaveAttribute('aria-current', 'page')
  const pageBody = page.locator('.mimir-book')
  await expect(pageBody.getByRole('heading', { name: 'Using This Book' })).toBeVisible()
  await expect(pageBody.locator('.inset')).toContainText('Quiet Camps')
  // The image loads with the token.
  await expect(pageBody.locator('.image-wrapper img')).toBeVisible()
  // A tooltip on a cross-reference.
  const fireball = pageBody.locator('a.spell-ref', { hasText: 'fireball' })
  await fireball.hover()
  await expect(fireball).toHaveAttribute('title', 'Fireball — 3rd-level evocation')
  // A cross-reference opens its detail.
  await pageBody.locator('a.creature-ref').click()
  await expect(page.getByRole('dialog')).toContainText('CR 1/4')
  await page.getByRole('dialog').getByRole('button', { name: 'Close' }).click()
  // A heading in the contents, and the next section.
  await contents.getByRole('link', { name: 'A Deeper Heading' }).click()
  await expect(page).toHaveURL(/at=s0-1-2/)
  await page.getByRole('link', { name: 'Next →' }).click()
  await expect(pageBody.getByRole('heading', { name: 'Random Encounters' })).toBeVisible()
  await expect(pageBody.locator('table')).toContainText('A lone wolf')
  await expect(pageBody.locator('.inset-readaloud')).toContainText('The road bends')
})

test('an adventure in the reader: levels, boxes, maps, area links, homebrew files', async ({ page }) => {
  await signIn(page)
  // The fixture adventure (MIMIR-T-0736) is listed under Adventures.
  await page.goto('/reference?mode=reader')
  const pick = page.getByLabel('Book')
  await expect(pick.locator('optgroup[label="Adventures"] option', { hasText: 'The Lantern of Low Fen' })).toHaveCount(1)
  await pick.selectOption('MFA')
  await expect(page.locator('.mimir-reader__levels')).toHaveText('Levels 1–3 · Mimir fixtures')
  await expect(page.getByRole('button', { name: 'Remove this adventure…' })).toBeVisible()
  const contents = page.getByRole('navigation', { name: 'Contents' })
  await expect(contents.getByRole('link', { name: 'The Old Causeway' })).toBeVisible()
  const pageBody = page.locator('.mimir-book')
  // The introduction: a sidebar, and an area link to the next-but-one chapter.
  await expect(pageBody.locator('.inset')).toContainText('Running This Adventure')
  await pageBody.locator('a.area-ref', { hasText: 'the gatehouse' }).click()
  await expect(page).toHaveURL(/section=2&at=mfa-g1/)
  await expect(pageBody.getByRole('heading', { name: '1. Gatehouse' })).toBeVisible()
  await expect(pageBody.locator('.inset-readaloud')).toContainText('rusted portcullis')
  // The map says it is a map, with its grid; the adventure's own monster links.
  await expect(pageBody.locator('.image-caption')).toContainText('(map, square grid)')
  await expect(pageBody.locator('a.creature-ref', { hasText: 'Lantern Wisp' }).first()).toBeVisible()

  // A homebrew file from Settings: its adventure joins the reader.
  const code = `E2E${Date.now() % 100000}`
  const file = {
    _meta: { sources: [{ json: code, abbreviation: code, full: `Homebrew ${code}` }] },
    adventure: [{ name: `Bog Road ${code}`, id: code, source: code, level: { start: 2, end: 2 }, contents: [{ name: 'The Road' }] }],
    adventureData: [{ id: code, source: code, data: [{ type: 'section', name: 'The Road', entries: ['Mud to the knees.'] }] }],
  }
  await page.goto('/settings?section=books')
  const hb = page.getByRole('region', { name: 'Import a homebrew file' })
  await hb.locator('input[type=file]').setInputFiles({ name: 'bog.json', mimeType: 'application/json', buffer: Buffer.from(JSON.stringify(file)) })
  await hb.getByRole('button', { name: 'Import' }).click()
  await expect(hb.getByRole('status')).toHaveText(`Imported ${code}: 1 adventure, 0 entries.`)
  await page.goto(`/reference?mode=reader&book=${code}&section=0`)
  await expect(page.locator('.mimir-reader__levels')).toHaveText('Level 2')
  await expect(page.locator('.mimir-book')).toContainText('Mud to the knees.')
})

test('an adventure becomes a campaign: plan, choices, the campaign', async ({ page }) => {
  await signIn(page)
  await page.goto('/reference?mode=reader&book=MFA&section=0')
  await page.getByRole('button', { name: 'Make a campaign…' }).click()
  const dialog = page.getByRole('dialog', { name: 'Make a campaign' })
  await expect(dialog.getByLabel('Campaign name')).toHaveValue('The Lantern of Low Fen')
  const name = `Fen ${Date.now() % 100000}`
  await dialog.getByLabel('Campaign name').fill(name)
  // The plan: the two chapters are modules; leave out the creatures.
  await expect(dialog.getByLabel('The Old Causeway becomes')).toHaveValue('module')
  await expect(dialog.getByRole('row', { name: /The Old Causeway/ })).toContainText('3 documents · 0 maps')
  await expect(dialog.getByRole('row', { name: /The Lantern Tower/ })).toContainText('1 document · 1 map')
  await dialog.getByLabel('Creatures becomes').selectOption('skip')
  await dialog.getByLabel('The Old Causeway in one document').check()
  await dialog.getByRole('button', { name: 'Make the campaign' }).click()
  await expect(dialog.getByRole('status')).toHaveText('Made 2 modules, 3 documents, 1 map and 5 monsters.')
  await dialog.getByRole('link', { name: 'Open the campaign' }).click()
  await expect(page.getByRole('heading', { level: 1, name })).toBeVisible()
  await expect(page.locator('.mimir-doclist a', { hasText: 'Introduction' })).toBeVisible()
  await page.getByRole('tab', { name: 'Modules', exact: true }).click()
  await expect(page.getByRole('link', { name: 'The Old Causeway' })).toBeVisible()
  await expect(page.getByRole('link', { name: 'The Lantern Tower' })).toBeVisible()
})

test('the module inspector opens a monster', async ({ page }) => {
  await signIn(page)
  const campaign = await campaignId(page)
  const module = await moduleId(page, campaign)
  await page.goto(`/modules/${module}?section=dangers`)
  await page.getByRole('table', { name: 'Monsters' }).getByRole('button', { name: 'Goblin' }).first().click()
  await expect(page.getByRole('dialog', { name: 'Goblin' })).toContainText('CR 1/4')
})
