import { campaignId, expect, moduleId, signIn, test } from './session'

// The reference (MIMIR-T-0723): the catalog (search, filters, detail),
// cross-references opening over each other, the book reader (contents,
// headings, images, tooltips), and the module page's inspector.

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

test('the module inspector opens a monster', async ({ page }) => {
  await signIn(page)
  const campaign = await campaignId(page)
  const module = await moduleId(page, campaign)
  await page.goto(`/modules/${module}?section=dangers`)
  await page.getByRole('table', { name: 'Monsters' }).getByRole('button', { name: 'Goblin' }).first().click()
  await expect(page.getByRole('dialog', { name: 'Goblin' })).toContainText('CR 1/4')
})
