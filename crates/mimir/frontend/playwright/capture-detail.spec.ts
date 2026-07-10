import { test } from '@playwright/test'

/**
 * Interaction-driven captures: panes that only render after a selection
 * (module detail, document viewer, character sheet sub-tabs, homebrew subtabs).
 */

const CAMPAIGN = 'cf8be92a-266e-4d40-b108-a42e2bb30f74'
const PC_MAJOR = 'cb5da3b1-73e2-4ad9-870e-1ce5633dcb7b'

async function shoot(page: import('@playwright/test').Page, name: string) {
  await page.waitForLoadState('networkidle')
  await page.waitForTimeout(600)
  await page.screenshot({ path: `playwright/screenshots/${name}.png`, fullPage: true })
}

test('module detail pane (M04 roster)', async ({ page }) => {
  await page.goto(`/campaigns/${CAMPAIGN}/dashboard/modules`)
  await page.waitForLoadState('networkidle')
  await page.getByText('Module 04').first().click()
  await shoot(page, 'module-detail-m04')
})

test('document viewer (Campaign Pitch)', async ({ page }) => {
  await page.goto(`/campaigns/${CAMPAIGN}/dashboard/campaign`)
  await page.waitForLoadState('networkidle')
  await page.getByText('Campaign Pitch').first().click()
  await shoot(page, 'document-viewer')
})

test('character equipment tab', async ({ page }) => {
  await page.goto(`/characters/${PC_MAJOR}`)
  await page.waitForLoadState('networkidle')
  await page.getByRole('button', { name: 'Equipment' }).or(page.getByText('Equipment', { exact: true })).first().click()
  await shoot(page, 'character-equipment')
})

test('character spells tab', async ({ page }) => {
  await page.goto(`/characters/${PC_MAJOR}`)
  await page.waitForLoadState('networkidle')
  await page.getByRole('button', { name: 'Spells' }).or(page.getByText('Spells', { exact: true })).first().click()
  await shoot(page, 'character-spells')
})

test('homebrew monsters subtab', async ({ page }) => {
  await page.goto(`/campaigns/${CAMPAIGN}/dashboard/homebrew`)
  await page.waitForLoadState('networkidle')
  await page.getByText('Monsters', { exact: true }).first().click()
  await page.waitForTimeout(400)
  // open the first monster if the list renders
  const first = page.getByText('Frost-Infected Bear').first()
  if (await first.isVisible().catch(() => false)) await first.click()
  await shoot(page, 'homebrew-monsters')
})
