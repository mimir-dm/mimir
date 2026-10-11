import { CAMPAIGN, TOKEN, campaignId, expect, signIn, test } from './session'
import type { Locator, Page } from '@playwright/test'

// The DM's character management (MIMIR-T-0721): the wizard for a PC (the
// class at level 1) and an NPC, the characters page (move, delete), NPC
// fields on the sheet. Each test deletes what it made: other specs pick
// fixture PCs by their place in the list.

const auth = { Authorization: `Bearer ${TOKEN}` }
const made: string[] = []

test.afterEach(async ({ page }) => {
  const campaign = await campaignId(page)
  const lists = [`/api/v1/campaigns/${campaign}/pcs`, `/api/v1/campaigns/${campaign}/npcs`, '/api/v1/characters/unassigned']
  for (const l of lists) {
    for (const c of await (await page.request.get(l, { headers: auth })).json()) {
      if (made.includes(c.name)) await page.request.delete(`/api/v1/characters/${c.id}`, { headers: auth })
    }
  }
})

/** Pick the first option whose text starts with `text`. */
async function pick(select: Locator, text: string) {
  const value = await select.evaluate(
    (el, t) => [...(el as HTMLSelectElement).options].find((o) => o.text.startsWith(t))?.value ?? '',
    text,
  )
  expect(value, `an option starting with ${text}`).not.toBe('')
  await select.selectOption(value)
}

async function openTab(page: Page, tab: string) {
  await page.goto('/')
  await page.getByText(CAMPAIGN).first().click()
  await page.getByRole('tab', { name: tab, exact: true }).click()
}

test('a PC through the wizard starts at level 1', async ({ page }) => {
  made.push('Robin Brightwater')
  await signIn(page)
  await openTab(page, 'PCs')
  await page.getByRole('button', { name: 'New PC' }).click()
  const d = page.getByRole('dialog')
  const next = d.getByRole('button', { name: 'Next' })
  await expect(next).toBeDisabled()
  await d.getByLabel('Name').fill('Robin Brightwater')
  await d.getByLabel('Player').fill('Sam')
  await next.click()
  await pick(d.getByLabel('Race'), 'Human')
  await next.click()
  await pick(d.getByLabel('Class'), 'Fighter')
  await next.click()
  await pick(d.getByLabel('Background'), 'Acolyte')
  await next.click()

  // Point buy: 27 points on three 15s.
  await d.getByLabel('Method').selectOption('point-buy')
  await expect(next).toBeDisabled()
  for (const a of ['Strength', 'Dexterity', 'Constitution']) {
    for (let i = 0; i < 7; i++) await d.getByRole('button', { name: `Raise ${a}` }).click()
  }
  await expect(d.getByText('Points left: 0')).toBeVisible()
  await expect(d.getByRole('button', { name: 'Raise Wisdom' })).toBeDisabled()
  await next.click()

  // Skills: the background's, and two picks.
  const boxes = d.getByRole('group', { name: 'Class skills' }).getByRole('checkbox')
  await boxes.nth(0).check()
  await expect(next).toBeDisabled()
  await boxes.nth(1).check()
  await expect(boxes.nth(2)).toBeDisabled()
  await next.click()
  await expect(d.getByLabel('Review')).toContainText('Fighter')
  await d.getByRole('button', { name: 'Create' }).click()
  await expect(d).toHaveCount(0)

  const row = page.locator('tr', { hasText: 'Robin Brightwater' })
  await expect(row).toContainText('Fighter 1')
  await row.getByRole('link', { name: 'Robin Brightwater' }).click()
  await expect(page.locator('.cl-page-header').first()).toContainText('Level 1')
})

test('an NPC, its fields on the sheet, and the characters page', async ({ page }) => {
  made.push('Sister Garaele', 'Halia Thornton')
  await signIn(page)
  await openTab(page, 'NPCs')
  await page.getByRole('button', { name: 'New NPC' }).click()
  let d = page.getByRole('dialog')
  await d.getByLabel('Name').fill('Sister Garaele')
  await pick(d.getByLabel('Race'), 'Elf')
  await d.getByLabel('Role').fill('Priestess')
  await d.getByRole('button', { name: 'Create' }).click()
  const row = page.locator('tr', { hasText: 'Sister Garaele' })
  await expect(row).toContainText('Priestess')

  // The sheet's Details: the NPC fields.
  await row.getByRole('link', { name: 'Sister Garaele' }).click()
  await page.getByRole('tab', { name: 'Details' }).click()
  await page.getByLabel('Faction').fill('Harpers')
  await page.getByRole('button', { name: 'Save' }).click()
  await expect(page.getByText('Saved.')).toBeVisible()
  await page.reload()
  await page.getByRole('tab', { name: 'Details' }).click()
  await expect(page.getByLabel('Faction')).toHaveValue('Harpers')
  await expect(page.locator('.cl-page-header').first()).toContainText('Elf')

  // The characters page: one in no campaign, moved in, deleted.
  await page.goto('/characters')
  await page.getByRole('button', { name: 'New character' }).click()
  d = page.getByRole('dialog')
  await d.getByLabel('Kind').selectOption('npc')
  await d.getByLabel('Name').fill('Halia Thornton')
  await d.getByRole('button', { name: 'Create' }).click()
  const none = page.getByRole('region', { name: 'No campaign' })
  await expect(none.locator('tr', { hasText: 'Halia Thornton' })).toBeVisible()
  await none.getByLabel('Campaign of Halia Thornton').selectOption({ label: CAMPAIGN })
  const inCampaign = page.getByRole('region', { name: CAMPAIGN })
  await expect(inCampaign.locator('tr', { hasText: 'Halia Thornton' })).toBeVisible()
  await expect(none.locator('tr', { hasText: 'Halia Thornton' })).toHaveCount(0)
  await inCampaign.getByTitle('Delete Halia Thornton').click()
  await page.getByRole('dialog').getByRole('button', { name: 'Delete' }).click()
  await expect(page.locator('tr', { hasText: 'Halia Thornton' })).toHaveCount(0)
})
