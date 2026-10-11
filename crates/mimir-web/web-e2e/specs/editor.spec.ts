import { TOKEN, campaignId, expect, moduleId, signIn, test } from './session'
import type { Page } from '@playwright/test'

// The rich editor (MIMIR-T-0719): TipTap through wasm-bindgen.
//
// Round trip: the markdown TipTap writes is not byte for byte what it read
// (it pads tables, escapes brackets, puts blank lines around blocks), as
// in the desktop app. So a save must keep the document's structure and
// words, and a second save must change nothing. A document is written only
// when someone edits it.

const auth = { Authorization: `Bearer ${TOKEN}` }

type Trip = { markdown: string; json: string }
const trip = (page: Page, md: string) =>
  page.evaluate((m) => (window as unknown as { MimirEditor: { roundTrip(m: string): Trip } }).MimirEditor.roundTrip(m), md)
const words = (md: string) => md.match(/[\p{L}\p{N}]+/gu)?.filter((w) => w !== 'amp') ?? []

test('the fixture documents keep their structure and words through the editor', async ({ page }) => {
  await signIn(page)
  await page.goto('/')
  await page.waitForFunction(() => 'MimirEditor' in window)
  const campaign = await campaignId(page)
  const module = await moduleId(page, campaign)
  const docs: { id: string; title: string }[] = []
  for (const l of [`/api/v1/campaigns/${campaign}/documents`, `/api/v1/modules/${module}/documents`])
    docs.push(...(await (await page.request.get(l, { headers: auth })).json()))
  expect(docs.length).toBeGreaterThan(5)
  for (const d of docs) {
    const doc = await (await page.request.get(`/api/v1/documents/${d.id}`, { headers: auth })).json()
    const original: string = doc.content
    const once = await trip(page, original)
    const twice = await trip(page, once.markdown)
    expect.soft(twice.json, `${d.title}: structure`).toBe(once.json)
    expect.soft(twice.markdown, `${d.title}: a second save changes nothing`).toBe(once.markdown)
    expect.soft(words(once.markdown), `${d.title}: words`).toEqual(words(original))
    if (original.includes('&')) expect.soft(once.markdown, `${d.title}: &`).not.toContain('&amp;')
    if (/- \[ \]/.test(original)) expect.soft(once.markdown, `${d.title}: task list`).toMatch(/- \[ \]/)
  }
})

test('typing a numbered list continues it; the toolbar formats', async ({ page, browser }) => {
  await signIn(page)
  const campaign = await campaignId(page)
  const pcs = (await (await page.request.get(`/api/v1/campaigns/${campaign}/pcs`, { headers: auth })).json()) as {
    id: string
    name: string
  }[]
  const thorin = pcs.find((p) => p.name === 'Thorin Ironforge')!
  const link = await (await page.request.post(`/api/v1/characters/${thorin.id}/link`, { headers: auth })).json()
  const device = await browser.newContext()
  const player = await device.newPage()
  await player.goto(link.path)
  await player.getByRole('tab', { name: 'Character' }).click()
  await player.getByRole('tab', { name: 'Notes' }).click()
  await player.getByRole('button', { name: 'New note' }).click()
  await player.getByLabel('Title').fill('Watch order')
  const editor = player.getByRole('textbox', { name: 'Note' })
  await editor.click()
  await player.keyboard.type('1. Thorin first')
  await player.keyboard.press('Enter')
  await player.keyboard.type('Elara second')
  await player.keyboard.press('Enter')
  await player.keyboard.press('Enter')
  const bold = player.getByRole('button', { name: 'Bold' })
  await bold.click()
  await expect(bold).toHaveAttribute('aria-pressed', 'true')
  await player.keyboard.type('Wake on noise')
  await expect(editor.locator('ol > li')).toHaveCount(2)
  await player.getByRole('button', { name: 'Save note' }).click()
  const note = player.locator('.mimir-notes__note', { hasText: 'Watch order' })
  await expect(note.locator('ol li')).toHaveCount(2)

  const notes = await (
    await page.request.get(`/api/v1/characters/${thorin.id}/notes`, { headers: auth })
  ).json()
  const saved = notes.find((n: { title: string }) => n.title === 'Watch order')
  expect(saved.content).toBe('1. Thorin first\n2. Elara second\n\n**Wake on noise**')
  await note.getByTitle('Delete the note').click()
  await expect(note).toHaveCount(0)

  await page.request.delete(`/api/v1/characters/${thorin.id}/link`, { headers: auth })
  await device.close()
})

test('the toolbar: headings, tables with rows and columns, rule, undo', async ({ page, browser }) => {
  await signIn(page)
  const campaign = await campaignId(page)
  const pcs = (await (await page.request.get(`/api/v1/campaigns/${campaign}/pcs`, { headers: auth })).json()) as {
    id: string
    name: string
  }[]
  const thorin = pcs.find((p) => p.name === 'Thorin Ironforge')!
  const link = await (await page.request.post(`/api/v1/characters/${thorin.id}/link`, { headers: auth })).json()
  const device = await browser.newContext()
  const player = await device.newPage()
  await player.goto(link.path)
  await player.getByRole('tab', { name: 'Character' }).click()
  await player.getByRole('tab', { name: 'Notes' }).click()
  await player.getByRole('button', { name: 'New note' }).click()
  const editor = player.getByRole('textbox', { name: 'Note' })
  const tool = (name: string) => player.getByRole('button', { name, exact: true })
  await editor.click()

  await tool('Heading 2').click()
  await player.keyboard.type('Loot')
  await expect(editor.locator('h2')).toHaveText('Loot')
  await expect(tool('Heading 2')).toHaveAttribute('aria-pressed', 'true')
  await player.keyboard.press('Enter')

  // Table actions are off outside a table.
  await expect(tool('Add a row')).toBeDisabled()
  await tool('Insert a table').click()
  const table = editor.locator('table')
  await expect(table.locator('tr')).toHaveCount(3)
  await expect(table.locator('tr').first().locator('th')).toHaveCount(3)
  await tool('Add a row').click()
  await expect(table.locator('tr')).toHaveCount(4)
  await tool('Add a column').click()
  await expect(table.locator('tr').first().locator('th, td')).toHaveCount(4)
  await tool('Delete the column').click()
  await expect(table.locator('tr').first().locator('th, td')).toHaveCount(3)
  await tool('Delete the row').click()
  await expect(table.locator('tr')).toHaveCount(3)
  await player.keyboard.type('Gold')
  await tool('Delete the table').click()
  await expect(table).toHaveCount(0)
  await tool('Undo').click()
  await expect(editor.locator('table')).toHaveCount(1)
  await tool('Redo').click()
  await expect(editor.locator('table')).toHaveCount(0)

  await tool('Horizontal rule').click()
  await expect(editor.locator('hr')).toHaveCount(1)
  await tool('Quote').click()
  await player.keyboard.type('Split it evenly')
  await expect(editor.locator('blockquote')).toHaveText('Split it evenly')

  await player.getByLabel('Title').fill('Toolbar')
  await player.getByRole('button', { name: 'Save note' }).click()
  const note = player.locator('.mimir-notes__note', { hasText: 'Toolbar' })
  await expect(note.locator('blockquote')).toBeVisible()
  const notes = await (await page.request.get(`/api/v1/characters/${thorin.id}/notes`, { headers: auth })).json()
  const saved = notes.find((n: { title: string }) => n.title === 'Toolbar')
  expect(saved.content).toBe('## Loot\n\n---\n\n> Split it evenly')
  await note.getByTitle('Delete the note').click()
  await expect(note).toHaveCount(0)

  await page.request.delete(`/api/v1/characters/${thorin.id}/link`, { headers: auth })
  await device.close()
})
