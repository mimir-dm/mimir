import { CAMPAIGN, TOKEN, campaignId, expect, mapId, signIn, test } from './session'

// Player links (MIMIR-T-0716): the DM makes a link on a PC, another device
// opens it and gets the player page (the display, no DM pages), and loses
// it when the DM revokes or replaces the link. Runs in CI.

const auth = { Authorization: `Bearer ${TOKEN}` }

test('a player link: make, open on another device, revoke', async ({ page, browser }) => {
  await signIn(page)
  const campaign = await campaignId(page)
  const map = await mapId(page)
  await page.request.put(`/api/v1/campaigns/${campaign}/display`, { headers: auth, data: { map_id: null } })

  // The DM makes a link on the first PC.
  await page.goto('/')
  await page.getByText(CAMPAIGN).first().click()
  await page.getByRole('tab', { name: 'PCs', exact: true }).click()
  const first = page.locator('tbody tr').first()
  const name = (await first.locator('td').first().innerText()).trim()
  await first.getByRole('button', { name: 'Player link' }).click()
  await expect(page.getByText('No link yet.')).toBeVisible()
  await page.getByRole('button', { name: 'Make a link' }).click()
  const url = (await page.locator('.mimir-link__url').innerText()).trim()
  expect(url).toMatch(/\/play\/[0-9a-f]{64}$/)
  await expect(page.locator('.mimir-link__qr svg')).toBeVisible()
  await expect(page.getByText(/A link is active/)).toBeVisible()

  // Another device opens it: the player page, with no DM pages.
  const device = await browser.newContext()
  const player = await device.newPage()
  const errors: string[] = []
  player.on('pageerror', (e) => errors.push(String(e)))
  await player.goto(new URL(url).pathname)
  await expect(player).toHaveURL(/\/player$/)
  await expect(player.locator('.mimir-player__bar')).toContainText(name)
  await expect(player.getByText('Waiting for the DM to show a map.')).toBeVisible()
  await expect(player.locator('.cl-appshell')).toHaveCount(0)

  // The DM shows a map: the player sees it.
  await page.request.put(`/api/v1/campaigns/${campaign}/display`, { headers: auth, data: { map_id: map } })
  await expect(player.locator('.mimir-display__svg image')).toBeAttached()
  // The player token cannot read DM data.
  const stored = await player.evaluate(() => localStorage.getItem('mimir-dm-token'))
  const dm = await player.request.get('/api/v1/campaigns', { headers: { Authorization: `Bearer ${stored}` } })
  expect(dm.status()).toBe(403)

  // The DM revokes: the device is signed out at once.
  await page.getByRole('button', { name: 'Revoke' }).click()
  await expect(player.getByText('This player link no longer works.', { exact: false })).toBeVisible()
  await expect(page.getByText('No link yet.')).toBeVisible()
  // The old address no longer works either.
  await player.goto(new URL(url).pathname)
  await expect(player.getByText('This player link no longer works.', { exact: false })).toBeVisible()

  // A new link works; making another ends it.
  await page.getByRole('button', { name: 'Make a link' }).click()
  const second = (await page.locator('.mimir-link__url').innerText()).trim()
  await player.goto(new URL(second).pathname)
  await expect(player.locator('.mimir-player__bar')).toContainText(name)
  page.once('dialog', (d) => d.accept())
  await page.getByRole('button', { name: 'New link' }).click()
  await expect(player.getByText('This player link no longer works.', { exact: false })).toBeVisible()
  expect(errors).toEqual([])

  await page.request.delete(`/api/v1/characters/${(await (await page.request.get(`/api/v1/campaigns/${campaign}/pcs`, { headers: auth })).json())[0].id}/link`, { headers: auth })
  await page.request.put(`/api/v1/campaigns/${campaign}/display`, { headers: auth, data: { map_id: null } })
  await device.close()
})
