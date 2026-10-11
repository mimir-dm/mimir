import { CAMPAIGN, TOKEN, campaignId, expect, signIn, test } from './session'
import { readFileSync } from 'node:fs'
import { gzipSync } from 'node:zlib'
import type { Download, Page } from '@playwright/test'

// Settings and admin (MIMIR-T-0724): a campaign archive round trip, a
// catalog import as a job and removing the source, PDFs, source books.

const auth = { Authorization: `Bearer ${TOKEN}` }

/** A minimal ustar archive of files (path, text), gzipped; a path ending
 * in "/" is a folder (the importer finds the top folder from one). */
function tarGz(files: [string, string][]): Buffer {
  const blocks: Buffer[] = []
  for (const [name, text] of files) {
    const data = Buffer.from(text)
    const h = Buffer.alloc(512)
    h.write(name, 0, 100)
    h.write('0000644\0', 100)
    h.write('0000000\0', 108)
    h.write('0000000\0', 116)
    h.write(data.length.toString(8).padStart(11, '0') + '\0', 124)
    h.write('00000000000\0', 136)
    h.write('        ', 148)
    h.write(name.endsWith('/') ? '5' : '0', 156)
    h.write('ustar\0', 257)
    h.write('00', 263)
    let sum = 0
    for (const b of h) sum += b
    h.write(sum.toString(8).padStart(6, '0') + '\0 ', 148)
    blocks.push(h, data, Buffer.alloc((512 - (data.length % 512)) % 512))
  }
  blocks.push(Buffer.alloc(1024))
  return gzipSync(Buffer.concat(blocks))
}

async function saved(d: Download): Promise<Buffer> {
  return readFileSync((await d.path())!)
}

async function pdfFrom(page: Page, click: () => Promise<void>) {
  const [d] = await Promise.all([page.waitForEvent('download'), click()])
  expect(d.suggestedFilename()).toMatch(/\.pdf$/)
  expect((await saved(d)).subarray(0, 4).toString()).toBe('%PDF')
}

test('a campaign archive: export, import as a copy, delete the copy', async ({ page }) => {
  await signIn(page)
  await page.goto('/settings')
  const row = page.getByRole('table', { name: 'Campaigns' }).locator('tr', { hasText: CAMPAIGN }).first()
  const [d] = await Promise.all([page.waitForEvent('download'), row.getByRole('button', { name: 'Export' }).click()])
  expect(d.suggestedFilename()).toMatch(/\.mimir-campaign\.tar\.gz$/)
  const archive = await saved(d)
  expect(archive.subarray(0, 2)).toEqual(Buffer.from([0x1f, 0x8b]))

  const section = page.getByRole('region', { name: 'Import a campaign' })
  await section.locator('input[type=file]').setInputFiles({ name: 'copy.mimir-campaign.tar.gz', mimeType: 'application/gzip', buffer: archive })
  await section.getByLabel(/Name/).fill('Phandelver (e2e copy)')
  await section.getByRole('button', { name: 'Import' }).click()
  await expect(section.getByRole('alert')).toContainText('Imported Phandelver (e2e copy)')
  const copy = page.getByRole('table', { name: 'Campaigns' }).locator('tr', { hasText: 'Phandelver (e2e copy)' })
  await expect(copy).toContainText('Active')

  // Its map image came along.
  const list = await (await page.request.get('/api/v1/campaigns', { headers: auth })).json()
  const id = list.find((c: { name: string }) => c.name === 'Phandelver (e2e copy)').id
  const maps = await (await page.request.get(`/api/v1/campaigns/${id}/maps`, { headers: auth })).json()
  expect((await page.request.get(`/api/v1/maps/${maps[0].id}/image`, { headers: auth })).status()).toBe(200)

  // Archive, then delete with the typed name.
  await copy.getByRole('button', { name: 'Archive' }).click()
  await expect(copy).toContainText('Archived')
  await copy.getByRole('button', { name: 'Delete…' }).click()
  const confirm = page.getByRole('dialog')
  await expect(confirm.getByRole('button', { name: 'Delete' })).toBeDisabled()
  await confirm.getByRole('textbox').fill('Phandelver (e2e copy)')
  await confirm.getByRole('button', { name: 'Delete' }).click()
  await expect(page.getByRole('table', { name: 'Campaigns' }).locator('tr', { hasText: 'Phandelver (e2e copy)' })).toHaveCount(0)
})

test('a catalog import job, then the source removed', async ({ page }) => {
  await signIn(page)
  await page.goto('/settings?section=books')
  const data = tarGz([
    ['5etools-e2e/', ''],
    ['5etools-e2e/data/books.json', JSON.stringify({ book: [{ id: 'EZE', name: 'E2E Rules', source: 'EZE', group: 'supplement', contents: [{ name: 'One' }] }] })],
    ['5etools-e2e/data/book/book-eze.json', JSON.stringify({ data: [{ type: 'section', name: 'One', entries: ['Hello.'] }] })],
    ['5etools-e2e/data/spells/spells-eze.json', JSON.stringify({ spell: [{ name: 'E2E Spark', source: 'EZE', level: 0, school: 'V', time: [{ number: 1, unit: 'action' }], range: { type: 'point', distance: { type: 'feet', amount: 30 } }, components: { v: true }, duration: [{ type: 'instant' }], entries: ['A spark.'] }] })],
  ])
  const section = page.getByRole('region', { name: 'Import 5etools data' })
  await section.locator('input[type=file]').setInputFiles({ name: '5etools-e2e.tar.gz', mimeType: 'application/gzip', buffer: data })
  await section.getByRole('button', { name: 'Import' }).click()
  await expect(section.getByRole('status')).toContainText('Imported 1 sources', { timeout: 20000 })
  const row = page.getByRole('table', { name: 'Sources' }).locator('tr', { hasText: 'EZE' })
  await expect(row).toContainText('E2E Rules')
  await expect(row).toContainText('Yes')
  await row.getByTitle('Remove EZE').click()
  await page.getByRole('dialog').getByRole('button', { name: 'Remove' }).click()
  await expect(page.getByRole('table', { name: 'Sources' }).locator('tr', { hasText: 'EZE' })).toHaveCount(0)
})

test('PDFs and source books', async ({ page }) => {
  test.setTimeout(120_000)
  await signIn(page)
  const campaign = await campaignId(page)
  await page.goto(`/campaigns/${campaign}`)
  // Source books: PHB only, then all again.
  await page.getByRole('button', { name: 'More campaign actions' }).click()
  await page.getByRole('menuitem', { name: 'Source books…' }).click()
  let d = page.getByRole('dialog')
  await d.getByRole('group', { name: 'Source books' }).getByText(/\(PHB\)/).click()
  await d.getByRole('button', { name: 'Save' }).click()
  await expect(d).toHaveCount(0)
  const s = await (await page.request.get(`/api/v1/campaigns/${campaign}/sources`, { headers: auth })).json()
  expect(s.codes).toEqual(['PHB'])
  await page.request.put(`/api/v1/campaigns/${campaign}/sources`, { headers: auth, data: { codes: [] } })

  // A campaign PDF (the documents only, to keep it quick).
  await page.getByRole('button', { name: 'More campaign actions' }).click()
  await page.getByRole('menuitem', { name: 'PDF…' }).click()
  d = page.getByRole('dialog')
  await d.getByText('Module content').click()
  await d.getByText('NPCs').click()
  await pdfFrom(page, () => d.getByRole('button', { name: 'Download PDF' }).click())

  // A character's PDF.
  const pcs = await (await page.request.get(`/api/v1/campaigns/${campaign}/pcs`, { headers: auth })).json()
  await page.goto(`/characters/${pcs[0].id}`)
  await page.getByRole('button', { name: 'PDF…' }).click()
  await pdfFrom(page, () => page.getByRole('dialog').getByRole('button', { name: 'Download PDF' }).click())
})
