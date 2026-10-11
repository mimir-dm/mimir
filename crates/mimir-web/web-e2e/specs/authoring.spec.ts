import { TOKEN, expect, signIn, test } from './session'
import type { Page } from '@playwright/test'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'

// Authoring (MIMIR-T-0720): the DM makes a campaign, writes documents,
// makes a module with monsters and maps, then archives and deletes it.
// Each test works in a campaign of its own and removes it at the end.

const auth = { Authorization: `Bearer ${TOKEN}` }
const UVTT = fileURLToPath(new URL('../../../mimir-core/src/seed/assets/test-dungeon.dd2vtt', import.meta.url))

async function newCampaign(page: Page, name: string): Promise<string> {
  const c = await (await page.request.post('/api/v1/campaigns', { headers: auth, data: { name } })).json()
  return c.id
}

async function removeCampaign(page: Page, id: string) {
  await page.request.put(`/api/v1/campaigns/${id}/archive`, { headers: auth })
  await page.request.delete(`/api/v1/campaigns/${id}`, { headers: auth })
}

test('a campaign: create, edit, archive, restore, delete', async ({ page }) => {
  await signIn(page)
  await page.goto('/')
  await page.getByRole('button', { name: 'New campaign' }).click()
  const dialog = page.getByRole('dialog')
  await dialog.getByLabel('Name').fill('Storm King')
  await dialog.getByLabel('Description').fill('Giants walk again.')
  await dialog.getByRole('button', { name: 'Create' }).click()
  await expect(page.getByRole('heading', { name: 'Storm King', level: 1 })).toBeVisible()
  // The planning documents came with it.
  await expect(page.getByRole('navigation', { name: 'Documents' }).getByRole('link')).not.toHaveCount(0)

  await page.getByRole('button', { name: 'Edit campaign' }).click()
  await page.getByRole('dialog').getByLabel('Name').fill("Storm King's Thunder")
  await page.getByRole('dialog').getByRole('button', { name: 'Save' }).click()
  await expect(page.getByRole('heading', { name: "Storm King's Thunder", level: 1 })).toBeVisible()

  await page.getByRole('button', { name: 'More campaign actions' }).click()
  await page.getByRole('menuitem', { name: 'Archive', exact: true }).click()
  await expect(page.locator('.mimir-dash__titlebar')).toContainText('Archived')
  await page.getByRole('button', { name: 'More campaign actions' }).click()
  await page.getByRole('menuitem', { name: 'Restore' }).click()
  await expect(page.locator('.mimir-dash__titlebar')).not.toContainText('Archived')
  await page.getByRole('button', { name: 'More campaign actions' }).click()
  await page.getByRole('menuitem', { name: 'Archive', exact: true }).click()
  await page.getByRole('button', { name: 'More campaign actions' }).click()
  await page.getByRole('menuitem', { name: 'Delete…' }).click()
  const confirm = page.getByRole('dialog')
  const del = confirm.getByRole('button', { name: 'Delete' })
  await expect(del).toBeDisabled()
  await confirm.getByRole('textbox').fill("Storm King's Thunder")
  await del.click()
  await expect(page).toHaveURL(/\/$/)
  const list = await (await page.request.get('/api/v1/campaigns?archived=true', { headers: auth })).json()
  expect(list.some((c: { name: string }) => c.name === "Storm King's Thunder")).toBe(false)
})

test('documents: write with autosave, rename, reorder, delete', async ({ page }) => {
  await signIn(page)
  const id = await newCampaign(page, 'Doc Test')
  await page.goto(`/campaigns/${id}`)
  for (const title of ['Rumours', 'Calendar']) {
    await page.getByRole('button', { name: 'New document' }).click()
    await page.getByRole('dialog').getByLabel('Title').fill(title)
    await page.getByRole('dialog').getByRole('button', { name: 'Create' }).click()
    await expect(page.getByRole('heading', { name: title, level: 2 })).toBeVisible()
  }
  const nav = page.getByRole('navigation', { name: 'Documents' })
  const own = nav.locator('.mimir-doclist__row:has(.mimir-order)')
  await expect(own).toHaveText(['Rumours↑↓', 'Calendar↑↓'])

  // Write in Calendar: saved a second after the last change.
  await page.getByRole('button', { name: 'Edit', exact: true }).click()
  const editor = page.getByRole('textbox', { name: 'Document' })
  await editor.click()
  await page.keyboard.type('1. First frost')
  await page.keyboard.press('Enter')
  await page.keyboard.type('Midwinter feast')
  await expect(page.getByRole('status')).toHaveText('Saved', { timeout: 5000 })
  await page.getByLabel('Title').fill('Calendar of Harptos')
  await expect(page.getByRole('status')).toHaveText('Saved', { timeout: 5000 })
  await page.getByRole('button', { name: 'Done' }).click()
  await page.reload()
  await expect(page.getByRole('heading', { name: 'Calendar of Harptos', level: 2 })).toBeVisible()
  await expect(page.locator('.mimir-doc ol li')).toHaveText(['First frost', 'Midwinter feast'])

  // Move it up, above Rumours.
  await page.getByRole('button', { name: 'Move Calendar of Harptos up' }).click()
  await expect(own).toHaveText(['Calendar of Harptos↑↓', 'Rumours↑↓'])
  await expect(page.getByRole('button', { name: 'Move Calendar of Harptos up' })).toBeDisabled()

  // Delete it.
  await page.getByTitle('Delete the document').click()
  await page.getByRole('dialog').getByRole('button', { name: 'Delete' }).click()
  await expect(own).toHaveText(['Rumours↑↓'])

  await removeCampaign(page, id)
})

test('a module: create, monsters, maps, edit, delete', async ({ page }) => {
  await signIn(page)
  const id = await newCampaign(page, 'Module Test')
  await page.goto(`/campaigns/${id}/modules`)
  await page.getByRole('button', { name: 'New module' }).click()
  const dialog = page.getByRole('dialog')
  await dialog.getByLabel('Name').fill('Wave Echo Cave')
  await dialog.getByLabel('Type').selectOption('dungeon')
  await dialog.getByRole('button', { name: 'Create' }).click()
  await expect(page.getByRole('heading', { name: 'Wave Echo Cave' })).toBeVisible()
  await expect(page.getByRole('navigation', { name: 'Documents' }).getByRole('link')).not.toHaveCount(0)

  // Monsters from the catalog.
  await page.getByRole('tab', { name: 'Dangers' }).click()
  await page.getByLabel('Monster name').fill('goblin')
  await page.getByRole('button', { name: 'Search' }).click()
  await page.getByRole('list', { name: 'Monsters found' }).getByRole('listitem').first().getByRole('button', { name: 'Add' }).click()
  const row = page.getByRole('table', { name: 'Monsters' }).locator('tbody tr').first()
  await expect(row).toContainText(/goblin/i)
  await row.getByRole('button', { name: 'Edit' }).click()
  await page.getByRole('dialog').getByLabel('Count').fill('3')
  await page.getByRole('dialog').getByLabel('DM notes').fill('On the ridge')
  await page.getByRole('dialog').getByRole('button', { name: 'Save' }).click()
  await expect(row).toContainText('3')
  await expect(row).toContainText('On the ridge')

  // Maps: a UVTT file, and an image with its grid.
  await page.getByRole('tab', { name: 'Maps' }).click()
  await page.getByRole('button', { name: 'Upload a map' }).click()
  let upload = page.getByRole('dialog')
  await upload.locator('input[type=file]').setInputFiles(UVTT)
  await expect(upload.getByLabel('Name')).toHaveValue('Test Dungeon')
  await expect(upload.getByLabel('Grid: pixels a square')).toHaveCount(0)
  await upload.getByRole('button', { name: 'Upload' }).click()
  const maps = page.getByRole('table', { name: 'Maps' })
  await expect(maps.getByRole('link', { name: 'Test Dungeon' })).toBeVisible()

  const png = Buffer.from(JSON.parse(readFileSync(UVTT, 'utf8')).image, 'base64')
  await page.getByRole('button', { name: 'Upload a map' }).click()
  upload = page.getByRole('dialog')
  await upload.locator('input[type=file]').setInputFiles({ name: 'cave-entrance.png', mimeType: 'image/png', buffer: png })
  await expect(upload.getByLabel('Grid: pixels a square')).toBeVisible()
  await upload.getByLabel('Lighting').selectOption('dark')
  await upload.getByRole('button', { name: 'Upload' }).click()
  await expect(maps.getByRole('link', { name: 'Cave Entrance' })).toBeVisible()
  await expect(maps.locator('tr', { hasText: 'Cave Entrance' })).toContainText('dark')

  // Drop a file on the upload: the same as choosing it (MIMIR-T-0731).
  await page.getByRole('button', { name: 'Upload a map' }).click()
  upload = page.getByRole('dialog')
  const dropped = await page.evaluateHandle((b64) => {
    const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0))
    const dt = new DataTransfer()
    dt.items.add(new File([bytes], 'dropped-cave.png', { type: 'image/png' }))
    return dt
  }, png.toString('base64'))
  await upload.locator('.mimir-drop').dispatchEvent('drop', { dataTransfer: dropped })
  await expect(upload.getByLabel('Name')).toHaveValue('Dropped Cave')
  await upload.getByRole('button', { name: 'Upload' }).click()
  await expect(maps.getByRole('link', { name: 'Dropped Cave' })).toBeVisible()

  // The preview: the image, its size and grid, a PDF with the page count.
  await maps.locator('tr', { hasText: 'Cave Entrance' }).getByRole('button', { name: 'Preview' }).click()
  const preview = page.getByRole('region', { name: 'Map preview' })
  await expect(preview.locator('img')).toBeVisible()
  await expect(preview).toContainText(/[\d.]+ × [\d.]+ squares · 70 px a square/)
  await preview.getByRole('button', { name: 'PDF…' }).click()
  const pdfDialog = page.getByRole('dialog', { name: 'PDF of the map' })
  await expect(pdfDialog).not.toContainText('letter pages')
  await pdfDialog.getByText('To play on (tiled, actual size)').click()
  await expect(pdfDialog).toContainText(/about \d+ letter pages at 1 inch a square/)
  await pdfDialog.getByRole('button', { name: 'Cancel' }).click()
  await preview.locator('.cl-btn', { hasText: 'Close' }).click()

  // A file that is not a map: refused with the server's reason.
  await page.getByRole('button', { name: 'Upload a map' }).click()
  upload = page.getByRole('dialog')
  await upload.locator('input[type=file]').setInputFiles({ name: 'notes.png', mimeType: 'image/png', buffer: Buffer.from('not a png') })
  await upload.getByRole('button', { name: 'Upload' }).click()
  await expect(upload.getByRole('alert')).toContainText('notes.png')
  await upload.getByRole('button', { name: 'Cancel' }).click()

  // The map opens on the DM map page.
  await maps.getByRole('link', { name: 'Cave Entrance' }).click()
  await expect(page.locator('svg image').first()).toBeAttached()
  await page.goBack()

  // Delete a map.
  await page.getByRole('tab', { name: 'Maps' }).click()
  await page.getByTitle('Delete Test Dungeon').click()
  await page.getByRole('dialog').getByRole('button', { name: 'Delete' }).click()
  await expect(maps.getByRole('link', { name: 'Test Dungeon' })).toHaveCount(0)

  // Edit, then delete the module.
  await page.getByRole('button', { name: 'Edit module' }).click()
  await page.getByRole('dialog').getByLabel('Name').fill('Wave Echo')
  await page.getByRole('dialog').getByRole('button', { name: 'Save' }).click()
  await expect(page.getByRole('heading', { name: 'Wave Echo', exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'More module actions' }).click()
  await page.getByRole('menuitem', { name: 'Delete module…' }).click()
  await page.getByRole('dialog').getByRole('button', { name: 'Delete' }).click()
  await expect(page).toHaveURL(new RegExp(`/campaigns/${id}/modules$`))
  await expect(page.getByText('No modules yet.')).toBeVisible()

  await removeCampaign(page, id)
})

test('images: a new document from an image, view it, delete it', async ({ page }) => {
  await signIn(page)
  const list = await (await page.request.get('/api/v1/campaigns', { headers: { Authorization: `Bearer ${TOKEN}` } })).json()
  const campaign = list[0].id
  const png = Buffer.from(JSON.parse(readFileSync(UVTT, 'utf8')).image, 'base64')
  await page.goto(`/campaigns/${campaign}`)
  await page.getByRole('button', { name: 'New document' }).click()
  const dialog = page.getByRole('dialog', { name: 'New document' })
  await dialog.locator('input[type=file]').setInputFiles({ name: 'wanted-poster.png', mimeType: 'image/png', buffer: png })
  await expect(dialog.locator('img.mimir-image-preview')).toBeVisible()
  await expect(dialog.getByLabel('Title')).toHaveValue('wanted poster')
  await dialog.getByLabel('Title').fill('Wanted poster')
  await dialog.getByRole('button', { name: 'Create' }).click()

  // It opens big, and lists under Images.
  const viewer = page.getByRole('dialog', { name: 'Wanted poster' })
  await expect(viewer.locator('img.mimir-image-view')).toBeVisible()
  await page.keyboard.press('Escape')
  await expect(viewer).toHaveCount(0)
  const images = page.getByRole('navigation', { name: 'Images' })
  await images.getByRole('button', { name: 'Wanted poster' }).click()
  page.once('dialog', (d) => d.accept())
  await page.getByRole('dialog', { name: 'Wanted poster' }).getByRole('button', { name: 'Delete image' }).click()
  await expect(page.getByRole('button', { name: 'Wanted poster' })).toHaveCount(0)
})
