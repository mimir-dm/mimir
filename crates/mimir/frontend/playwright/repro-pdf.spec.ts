import { test } from '@playwright/test'

const PC_MAJOR = 'cb5da3b1-73e2-4ad9-870e-1ce5633dcb7b'

test('reproduce PDF print failure', async ({ page }) => {
  const logs: string[] = []
  page.on('console', (msg) => logs.push(`[${msg.type()}] ${msg.text()}`))
  page.on('pageerror', (err) => logs.push(`[pageerror] ${err}`))

  await page.goto(`/characters/${PC_MAJOR}`)
  await page.waitForLoadState('networkidle')

  await page.getByRole('button', { name: 'Print PDF' }).click()
  await page.waitForTimeout(500)
  await page.screenshot({ path: 'playwright/screenshots/pdf-step1-options.png' })

  // CharacterPrintDialog -> generate
  const exportBtn = page.getByRole('button', { name: /Export|Generate/ }).first()
  if (await exportBtn.isVisible().catch(() => false)) {
    await exportBtn.click()
    logs.push('--- clicked Export ---')
  }
  // wait for the preview to finish generating (PDF takes a few seconds)
  await page.waitForTimeout(8000)
  await page.screenshot({ path: 'playwright/screenshots/pdf-step2-preview.png' })

  const saveBtn = page.getByRole('button', { name: 'Save PDF' }).first()
  if (await saveBtn.isVisible().catch(() => false)) {
    const downloadPromise = page.waitForEvent('download', { timeout: 10_000 })
    await saveBtn.click()
    logs.push('--- clicked Save PDF ---')
    const download = await downloadPromise
    logs.push(`--- download received: ${download.suggestedFilename()} ---`)
  }
  await page.waitForTimeout(1000)
  await page.screenshot({ path: 'playwright/screenshots/pdf-step3-after-save.png' })

  console.log(logs.join('\n'))
})
