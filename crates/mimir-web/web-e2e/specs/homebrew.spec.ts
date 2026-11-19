import { TOKEN, campaignId, expect, moduleId, signIn, test } from './session'
import type { Page } from '@playwright/test'

// Homebrew (MIMIR-T-0722): an item through the form and the JSON (the
// form keeps what it does not model), a monster copied from the catalog
// and used by a module (delete says so), a spell from the template.

const auth = { Authorization: `Bearer ${TOKEN}` }
const made = ['Ember Ring', 'Goblin Boss', 'Ember Lance']

test.afterEach(async ({ page }) => {
  const campaign = await campaignId(page)
  for (const kind of ['items', 'monsters', 'spells']) {
    const list = await (await page.request.get(`/api/v1/campaigns/${campaign}/homebrew/${kind}`, { headers: auth })).json()
    for (const h of list) {
      if (made.includes(h.name)) await page.request.delete(`/api/v1/homebrew/${kind}/${h.id}`, { headers: auth })
    }
  }
})

async function open(page: Page, kind: string) {
  const campaign = await campaignId(page)
  await page.goto(`/campaigns/${campaign}/homebrew?kind=${kind}`)
}

test('an item: the form, the JSON, the preview', async ({ page }) => {
  await signIn(page)
  await open(page, 'items')
  await page.getByRole('button', { name: 'New' }).click()
  const d = page.getByRole('dialog')
  await d.getByLabel('Name').fill('Ember Ring')
  await d.getByLabel('Kind').selectOption('ring')
  await d.getByLabel('Rarity').selectOption('rare')
  await d.getByText('Requires attunement').click()
  await d.getByLabel(/Description/).fill('Warm to the touch.')
  const preview = d.getByLabel('Preview')
  await expect(preview).toContainText('Ring, rare')
  await expect(preview).toContainText('Requires attunement')
  // A field the form does not model, in the JSON; then the form again.
  await d.getByRole('tab', { name: 'JSON' }).click()
  const json = d.getByLabel('Data (JSON)')
  const data = JSON.parse(await json.inputValue())
  data.charges = 3
  await json.fill(JSON.stringify(data, null, 2))
  await expect(preview).toContainText('Charges')
  await json.fill('{ not json')
  await expect(d.getByText('Not valid JSON')).toBeVisible()
  await expect(d.getByRole('button', { name: 'Save' })).toBeDisabled()
  await json.fill(JSON.stringify(data, null, 2))
  await d.getByRole('tab', { name: 'Form' }).click()
  await d.getByLabel('Weight (lb.)').fill('1')
  await d.getByRole('button', { name: 'Save' }).click()
  await expect(d).toHaveCount(0)

  const row = page.getByRole('navigation', { name: 'Homebrew' }).getByRole('link', { name: /Ember Ring/ })
  await expect(row).toContainText('Ring')
  const main = page.locator('.mimir-split__main')
  await expect(main).toContainText('Charges')
  await expect(main).toContainText('1 lb.')
  await expect(main).toContainText('Source: HB')
})

test('a monster copied from the catalog, used by a module, deleted', async ({ page }) => {
  await signIn(page)
  await open(page, 'monsters')
  await page.getByRole('button', { name: 'Copy from the catalog' }).click()
  const d = page.getByRole('dialog')
  await d.getByLabel('Catalog name').fill('goblin')
  await d.getByRole('button', { name: 'Search' }).click()
  await d.getByRole('list', { name: 'Catalog entries' }).getByRole('listitem').filter({ hasText: /^Goblin/ }).first().getByRole('button', { name: 'Pick' }).click()
  await d.getByLabel('Name', { exact: true }).fill('Goblin Boss')
  await d.getByRole('button', { name: 'Copy' }).click()
  await expect(page.locator('.mimir-split__main')).toContainText('Copy of Goblin')
  await expect(page.locator('.mimir-split__main')).toContainText('CR 1/4')

  // The module adds it from the search (HB).
  const campaign = await campaignId(page)
  const module = await moduleId(page, campaign)
  await page.goto(`/modules/${module}?section=dangers`)
  await page.getByLabel('Monster name').fill('boss')
  await page.getByRole('button', { name: 'Search' }).click()
  const hit = page.getByRole('list', { name: 'Monsters found' }).getByRole('listitem').filter({ hasText: 'Goblin Boss' })
  await expect(hit).toContainText('Homebrew')
  await hit.getByRole('button', { name: 'Add' }).click()
  await expect(page.getByRole('table', { name: 'Monsters' })).toContainText('Goblin Boss')

  // Delete: the module is named, and loses it.
  await open(page, 'monsters')
  await page.getByRole('navigation', { name: 'Homebrew' }).getByRole('link', { name: /Goblin Boss/ }).click()
  await page.getByTitle('Delete Goblin Boss').click()
  const confirm = page.getByRole('dialog')
  await expect(confirm).toContainText('Goblin Boss')
  await expect(confirm).toContainText('Module')
  await confirm.getByRole('button', { name: 'Delete' }).click()
  await page.goto(`/modules/${module}?section=dangers`)
  await expect(page.getByRole('table', { name: 'Monsters' })).not.toContainText('Goblin Boss')
})

test('a spell from the template', async ({ page }) => {
  await signIn(page)
  await open(page, 'spells')
  await page.getByRole('button', { name: 'New' }).click()
  const d = page.getByRole('dialog')
  await d.getByLabel('Name').fill('Ember Lance')
  await expect(d.getByLabel('Preview')).toContainText('1st-level evocation')
  await d.getByRole('button', { name: 'Save' }).click()
  await expect(page.getByRole('navigation', { name: 'Homebrew' }).getByRole('link', { name: /Ember Lance/ })).toContainText('1st-level evocation')
})
