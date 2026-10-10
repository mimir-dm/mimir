import { CAMPAIGN, MODULE, TOKEN, expect, signIn, test } from './session'

// Smoke: boot, sign-in, campaign list, dashboard, module page. Runs in CI.

test('the server reports token mode', async ({ request }) => {
  const config = await (await request.get('/api/config')).json()
  expect(config.auth).toBe('token')
})

test('sign in: a wrong token is refused, the right one opens the app', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByRole('heading', { name: 'Mimir' })).toBeVisible()
  await page.getByLabel('Token').fill('wrong')
  await page.getByRole('button', { name: 'Sign in' }).click()
  await expect(page.getByText('The server refused this token.')).toBeVisible()
  await page.getByLabel('Token').fill(TOKEN)
  await page.getByRole('button', { name: 'Sign in' }).click()
  await expect(page.getByRole('heading', { name: 'Campaigns' })).toBeVisible()
  await page.reload()
  await expect(page.getByRole('heading', { name: 'Campaigns' })).toBeVisible()
  await page.getByRole('button', { name: 'Sign out' }).click()
  await expect(page.getByLabel('Token')).toBeVisible()
})

test('campaign list to dashboard tabs', async ({ page }) => {
  await signIn(page)
  await page.goto('/')
  await page.getByText(CAMPAIGN).first().click()
  await expect(page.getByRole('heading', { level: 1, name: CAMPAIGN })).toBeVisible()
  await expect(page.locator('.mimir-doc__title')).toHaveText('Campaign Bible')

  await page.locator('.mimir-doclist a', { hasText: 'House Rules' }).click()
  await expect(page.locator('.mimir-doc__title')).toHaveText('House Rules')
  await expect(page).toHaveURL(/\?doc=/)

  await page.getByRole('tab', { name: 'Modules', exact: true }).click()
  await expect(page.getByRole('link', { name: MODULE })).toBeVisible()
  await page.getByRole('tab', { name: 'NPCs', exact: true }).click()
  await expect(page.getByRole('cell', { name: 'Gundren Rockseeker' })).toBeVisible()
  await page.getByRole('tab', { name: 'PCs', exact: true }).click()
  await expect(page.getByRole('cell', { name: 'Wizard 5' })).toBeVisible()
  await expect(page.getByRole('tab', { name: 'PCs', exact: true })).toHaveAttribute('aria-selected', 'true')
})

test('module page sections', async ({ page }) => {
  await signIn(page)
  await page.goto('/')
  await page.getByText(CAMPAIGN).first().click()
  await page.getByRole('tab', { name: 'Modules', exact: true }).click()
  await page.getByRole('link', { name: MODULE }).click()
  await expect(page.getByRole('heading', { name: MODULE })).toBeVisible()
  await expect(page.locator('.mimir-doc__title')).toBeVisible()
  await page.getByRole('tab', { name: 'NPCs' }).click()
  await expect(page.getByRole('cell', { name: 'Sildar Hallwinter' })).toBeVisible()
  await page.getByRole('tab', { name: 'Maps' }).click()
  await expect(page.getByRole('cell', { name: 'Goblin Hideout' })).toBeVisible()
  await page.getByRole('tab', { name: 'Dangers' }).click()
  await expect(page.getByRole('cell', { name: 'Cragmaw Mutants' })).toBeVisible()
  await expect(page).toHaveURL(/\?section=dangers$/)
  await page.goBack()
  await expect(page.getByRole('cell', { name: 'Goblin Hideout' })).toBeVisible()
  await expect(page.getByRole('tab', { name: 'Maps' })).toHaveAttribute('aria-selected', 'true')
})

test('an unknown address and an unknown campaign', async ({ page }) => {
  await signIn(page)
  await page.goto('/no/such/page')
  await expect(page.getByRole('heading', { name: 'Page not found' })).toBeVisible()
  await page.goto('/campaigns/no-such-campaign')
  await expect(page.getByRole('alert')).toContainText('Campaign not found')
})
