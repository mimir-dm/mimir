import { test as base, expect, type Page } from '@playwright/test'

/**
 * `test` with an automatic check: no page error and no console error, but
 * the browser's own line for a 401 or 404 answer (a refused token, an
 * unknown campaign: the specs expect those).
 */
export const test = base.extend<{ consoleErrors: string[] }>({
  consoleErrors: [
    async ({ page }, use) => {
      const errors: string[] = []
      page.on('pageerror', (e) => errors.push(String(e)))
      page.on('console', (m) => {
        if (m.type() === 'error') errors.push(m.text())
      })
      await use(errors)
      expect(errors.filter((e) => !/status of (401|404)/.test(e))).toEqual([])
    },
    { auto: true },
  ],
})
export { expect }

/** The DM token of session.sh (a test value, not a secret). */
export const TOKEN = process.env.MIMIR_E2E_TOKEN ?? 'e2e-dm-token'
export const CAMPAIGN = 'The Lost Mine of Phandelver'
export const MODULE = 'Cragmaw Hideout'

/** Store the token and the theme before the app loads. */
export async function signIn(page: Page, theme?: 'light' | 'dark' | 'hyper') {
  await page.addInitScript(
    ([token, theme]) => {
      localStorage.setItem('mimir-dm-token', token)
      if (theme) localStorage.setItem('aurora-theme', theme)
    },
    [TOKEN, theme ?? ''] as const,
  )
}

/** The id of the fixture campaign, read from the API. */
export async function campaignId(page: Page): Promise<string> {
  const res = await page.request.get('/api/v1/campaigns', {
    headers: { Authorization: `Bearer ${TOKEN}` },
  })
  const list = (await res.json()) as { id: string; name: string }[]
  const c = list.find((x) => x.name === CAMPAIGN)
  if (!c) throw new Error('fixture campaign missing')
  return c.id
}

/** The id of the fixture module. */
export async function moduleId(page: Page, campaign: string): Promise<string> {
  const res = await page.request.get(`/api/v1/campaigns/${campaign}/modules`, {
    headers: { Authorization: `Bearer ${TOKEN}` },
  })
  const list = (await res.json()) as { id: string; name: string }[]
  const m = list.find((x) => x.name === MODULE)
  if (!m) throw new Error('fixture module missing')
  return m.id
}

/** The id of the fixture module map ("Goblin Hideout"). */
export async function mapId(page: Page): Promise<string> {
  const campaign = await campaignId(page)
  const module = await moduleId(page, campaign)
  const res = await page.request.get(`/api/v1/modules/${module}/maps`, {
    headers: { Authorization: `Bearer ${TOKEN}` },
  })
  const list = (await res.json()) as { id: string; name: string }[]
  return list[0].id
}
