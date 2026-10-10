import { CAMPAIGN, MODULE, campaignId, expect, moduleId, signIn, test } from './session'

// Captures for design review: each screen × desktop and tablet × the themes
// in THEMES (default light,dark). Run with `angreal web screenshots`; it sets
// SCREENSHOT_DIR to a new run directory. Compare runs with
// `angreal web screenshots-diff`.

const outDir = process.env.SCREENSHOT_DIR ?? 'screenshots/latest'
const themes = (process.env.THEMES ?? 'light,dark').split(',').map((t) => t.trim()) as (
  | 'light'
  | 'dark'
  | 'hyper'
)[]
const sizes = [
  { name: 'desktop', width: 1280, height: 800 },
  { name: 'tablet', width: 820, height: 1100 },
]

type Ids = { campaign: string; module: string }
type Screen = { name: string; path: (ids: Ids) => string; ready: (page: import('@playwright/test').Page) => Promise<void>; signedOut?: boolean }

const screens: Screen[] = [
  {
    name: 'sign-in',
    signedOut: true,
    path: () => '/',
    ready: async (p) => expect(p.getByLabel('Token')).toBeVisible(),
  },
  {
    name: 'campaigns',
    path: () => '/',
    ready: async (p) => expect(p.getByText(CAMPAIGN).first()).toBeVisible(),
  },
  {
    name: 'dashboard-campaign',
    path: (i) => `/campaigns/${i.campaign}`,
    ready: async (p) => expect(p.locator('.mimir-doc__title')).toHaveText('Campaign Bible'),
  },
  {
    name: 'dashboard-modules',
    path: (i) => `/campaigns/${i.campaign}/modules`,
    ready: async (p) => expect(p.getByRole('link', { name: MODULE })).toBeVisible(),
  },
  {
    name: 'dashboard-npcs',
    path: (i) => `/campaigns/${i.campaign}/npcs`,
    ready: async (p) => expect(p.getByRole('cell', { name: 'Gundren Rockseeker' })).toBeVisible(),
  },
  {
    name: 'dashboard-pcs',
    path: (i) => `/campaigns/${i.campaign}/pcs`,
    ready: async (p) => expect(p.getByRole('cell', { name: 'Wizard 5' })).toBeVisible(),
  },
  {
    name: 'module-documents',
    path: (i) => `/modules/${i.module}`,
    ready: async (p) => expect(p.locator('.mimir-doc__title')).toBeVisible(),
  },
]

for (const theme of themes) {
  for (const size of sizes) {
    for (const screen of screens) {
      test(`${screen.name} ${size.name} ${theme}`, async ({ page }) => {
        await page.setViewportSize({ width: size.width, height: size.height })
        const ids = { campaign: await campaignId(page), module: '' }
        ids.module = await moduleId(page, ids.campaign)
        if (screen.signedOut) {
          await page.addInitScript((t) => localStorage.setItem('aurora-theme', t), theme)
        } else {
          await signIn(page, theme)
        }
        await page.goto(screen.path(ids))
        await screen.ready(page)
        await page.evaluate(() => document.fonts.ready)
        await page.screenshot({ path: `${outDir}/${screen.name}--${size.name}--${theme}.png`, animations: 'disabled', caret: 'hide' })
      })
    }
  }
}

// The other sections of the module page (in the address), desktop only.
for (const theme of themes) {
  for (const [tab, cell] of [
    ['NPCs', 'Sildar Hallwinter'],
    ['Maps', 'Goblin Hideout'],
    ['Dangers', 'Cragmaw Mutants'],
  ] as const) {
    test(`module-${tab.toLowerCase()} desktop ${theme}`, async ({ page }) => {
      const campaign = await campaignId(page)
      const module = await moduleId(page, campaign)
      await signIn(page, theme)
      await page.goto(`/modules/${module}?section=${tab.toLowerCase()}`)
      await expect(page.getByRole('cell', { name: cell })).toBeVisible()
      await page.evaluate(() => document.fonts.ready)
      await page.screenshot({ path: `${outDir}/module-${tab.toLowerCase()}--desktop--${theme}.png`, animations: 'disabled', caret: 'hide' })
    })
  }
}
