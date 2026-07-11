import { test } from '@playwright/test'

const CAMPAIGN = 'cf8be92a-266e-4d40-b108-a42e2bb30f74'

test('PCs tab PDF flow with all options', async ({ page }) => {
  const logs: string[] = []
  page.on('console', (m) => logs.push(`[${m.type()}] ${m.text()}`))
  page.on('pageerror', (e) => logs.push(`[pageerror] ${e}`))

  await page.goto(`/campaigns/${CAMPAIGN}/dashboard/pcs`)
  await page.waitForLoadState('networkidle')

  // Major's card PDF button (4th card, but target by card containing name)
  const card = page.locator('.character-card, [class*=card]').filter({ hasText: 'Major' }).first()
  await card.getByText('PDF', { exact: true }).click()
  await page.waitForTimeout(500)
  await page.screenshot({ path: 'playwright/screenshots/pcs-pdf-1-options.png' })

  // select ALL checkboxes
  const boxes = page.locator('input[type=checkbox]')
  const n = await boxes.count()
  for (let i = 0; i < n; i++) {
    if (!(await boxes.nth(i).isChecked())) await boxes.nth(i).check()
  }
  await page.screenshot({ path: 'playwright/screenshots/pcs-pdf-2-all-checked.png' })

  await page.getByRole('button', { name: 'Export PDF' }).click()
  logs.push('--- clicked Export ---')
  await page.waitForTimeout(12000)
  await page.screenshot({ path: 'playwright/screenshots/pcs-pdf-3-after-export.png' })

  console.log(logs.filter((l) => /error|export|pdf|---/i.test(l)).join('\n'))
})
