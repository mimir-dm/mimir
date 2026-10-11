import { MODULE, TOKEN, campaignId, expect, mapId, moduleId, signIn, test } from './session'
import type { Browser, Page } from '@playwright/test'

// Player notes and party sharing (MIMIR-T-0718): a player writes a note
// during an encounter, the DM reads it on the sheet and the module; a
// player shares their sheet, another player reads it, then it is hidden.

const auth = { Authorization: `Bearer ${TOKEN}` }

async function pcs(page: Page, campaign: string) {
  return (await (await page.request.get(`/api/v1/campaigns/${campaign}/pcs`, { headers: auth })).json()) as {
    id: string
    name: string
  }[]
}

/** A player's device: a fresh context on the character's new link. */
async function playerDevice(page: Page, browser: Browser, id: string) {
  const link = await (await page.request.post(`/api/v1/characters/${id}/link`, { headers: auth })).json()
  const device = await browser.newContext()
  const player = await device.newPage()
  await player.goto(link.path)
  return { device, player, token: link.token as string }
}

// Whatever a test did: no links, nothing on the display.
test.afterEach(async ({ page }) => {
  const campaign = await campaignId(page)
  for (const pc of await pcs(page, campaign)) {
    await page.request.delete(`/api/v1/characters/${pc.id}/link`, { headers: auth })
  }
  await page.request.put(`/api/v1/campaigns/${campaign}/display`, {
    headers: auth,
    data: { map_id: null, blackout: false },
  })
})

test('a player writes a note; the DM reads it on the sheet and the module', async ({ page, browser }) => {
  await signIn(page)
  const campaign = await campaignId(page)
  const thorin = (await pcs(page, campaign)).find((p) => p.name === 'Thorin Ironforge')!
  // The module's map is on the display: the encounter.
  await page.request.put(`/api/v1/campaigns/${campaign}/display`, {
    headers: auth,
    data: { map_id: await mapId(page), blackout: false },
  })

  const { device, player } = await playerDevice(page, browser, thorin.id)
  await player.getByRole('tab', { name: 'Character' }).click()
  await player.getByRole('tab', { name: 'Notes' }).click()
  await player.getByRole('button', { name: 'New note' }).click()
  await player.getByLabel('Title').fill('Ambush at the cave')
  await player.getByLabel('Note (markdown)').fill('1. Goblins on the ridge\n2. A **wolf** pen')
  await player.getByRole('button', { name: 'Save note' }).click()
  const note = player.locator('.mimir-notes__note', { hasText: 'Ambush at the cave' })
  await expect(note).toBeVisible()
  await expect(note).toContainText(MODULE)
  await expect(note.locator('strong', { hasText: 'wolf' })).toBeVisible()

  // The DM: the sheet's Notes tab, read-only.
  await page.goto(`/characters/${thorin.id}`)
  await page.getByRole('tab', { name: 'Notes' }).click()
  await expect(page.locator('.mimir-notes__note', { hasText: 'Ambush at the cave' })).toBeVisible()
  await expect(page.getByRole('button', { name: 'New note' })).toHaveCount(0)

  // The DM: the module's player notes name the character.
  const module = await moduleId(page, campaign)
  await page.goto(`/modules/${module}?section=notes`)
  const about = page.locator('.mimir-notes__note', { hasText: 'Ambush at the cave' })
  await expect(about).toContainText('Thorin Ironforge')

  // The player deletes it.
  await note.getByTitle('Delete the note').click()
  await expect(note).toHaveCount(0)

  await device.close()
})

test('a shared sheet opens read-only for the party, until unshared', async ({ page, browser }) => {
  await signIn(page)
  const campaign = await campaignId(page)
  const party = await pcs(page, campaign)
  const thorin = party.find((p) => p.name === 'Thorin Ironforge')!
  const other = party.find((p) => p.name !== 'Thorin Ironforge')!

  const a = await playerDevice(page, browser, thorin.id)
  const b = await playerDevice(page, browser, other.id)

  // The other player: Thorin is in the party, not shared.
  await b.player.getByRole('tab', { name: 'Party' }).click()
  const row = b.player.locator('.mimir-party__member', { hasText: 'Thorin Ironforge' })
  await expect(row).toContainText('Not shared')
  await expect(row.getByRole('button', { name: 'View' })).toHaveCount(0)
  await expect(b.player.locator('.mimir-party__member', { hasText: other.name })).toContainText('You')

  // Thorin shares: the other player's list changes live.
  await a.player.getByRole('tab', { name: 'Character' }).click()
  await a.player.getByRole('tab', { name: 'Details' }).click()
  await a.player.getByText('Share this sheet with the party (read-only)').click()
  await expect(row).toContainText('Shared')
  await row.getByRole('button', { name: 'View' }).click()
  await expect(b.player.getByRole('heading', { name: 'Thorin Ironforge' })).toBeVisible()
  await expect(b.player.getByRole('button', { name: 'Level up' })).toHaveCount(0)
  await expect(b.player.getByRole('tab', { name: 'Notes' })).toHaveCount(0)
  await b.player.getByRole('tab', { name: 'Equipment' }).click()
  await expect(b.player.getByRole('button', { name: 'Save coins' })).toHaveCount(0)
  await expect(b.player.getByLabel('Item name')).toHaveCount(0)

  // Thorin stops sharing: the sheet closes on the other device.
  await a.player.getByText('Share this sheet with the party (read-only)').click()
  await expect(b.player.getByText('Thorin Ironforge does not share their sheet now.')).toBeVisible()
  const shared = await page.request.get(`/api/v1/characters/${thorin.id}`, {
    headers: { Authorization: `Bearer ${b.token}` },
  })
  expect(shared.status()).toBe(403)

  await a.device.close()
  await b.device.close()
})
