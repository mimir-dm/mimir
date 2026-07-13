import type { Page } from '@playwright/test'

/**
 * Design-review screen catalog: every main screen with real Frost Architect
 * data. `setup` performs the in-page selection needed for panes that only
 * render after interaction (module detail, document viewer, sheet sub-tabs).
 *
 * IDs reference The Frost Architect campaign in the session's DB snapshot.
 */

export const CAMPAIGN = 'cf8be92a-266e-4d40-b108-a42e2bb30f74'
export const MODULE_M04 = 'b2003fd9-e0ce-4447-b0f6-0767dbbaf68c' // Caverns of Echo (roster)
export const MODULE_M05 = 'd39397f4-a23e-4d98-8df2-1c1d339eb7da' // Dwarven Fortress (has map)
export const PC_MATRIM = 'ac345851-d6c3-4344-ae97-1fb15d3f073e'
export const PC_MAJOR = 'cb5da3b1-73e2-4ad9-870e-1ce5633dcb7b' // cleric-ish: spells + inventory

export interface Screen {
  name: string
  path: string
  setup?: (page: Page) => Promise<void>
}

export const SCREENS: Screen[] = [
  { name: 'home', path: '/' },
  { name: 'dashboard-campaign-docs', path: `/campaigns/${CAMPAIGN}/dashboard/campaign` },
  {
    name: 'document-viewer',
    path: `/campaigns/${CAMPAIGN}/dashboard/campaign`,
    setup: async (page) => {
      await page.getByText('Campaign Pitch').first().click()
    },
  },
  { name: 'dashboard-modules', path: `/campaigns/${CAMPAIGN}/dashboard/modules` },
  {
    name: 'module-detail',
    path: `/campaigns/${CAMPAIGN}/dashboard/modules`,
    setup: async (page) => {
      await page.getByText('Module 04').first().click()
    },
  },
  { name: 'dashboard-npcs', path: `/campaigns/${CAMPAIGN}/dashboard/npcs` },
  { name: 'dashboard-pcs', path: `/campaigns/${CAMPAIGN}/dashboard/pcs` },
  { name: 'dashboard-homebrew', path: `/campaigns/${CAMPAIGN}/dashboard/homebrew` },
  {
    name: 'homebrew-monster',
    path: `/campaigns/${CAMPAIGN}/dashboard/homebrew`,
    setup: async (page) => {
      await page.getByText('Monsters', { exact: true }).first().click()
      await page.waitForTimeout(400)
      const first = page.getByText('Frost-Infected Bear').first()
      if (await first.isVisible().catch(() => false)) await first.click()
    },
  },
  { name: 'characters', path: '/characters' },
  { name: 'character-sheet', path: `/characters/${PC_MAJOR}` },
  {
    name: 'character-equipment',
    path: `/characters/${PC_MATRIM}`,
    setup: async (page) => {
      await page.getByRole('button', { name: 'Equipment' }).first().click()
    },
  },
  {
    name: 'character-spells',
    path: `/characters/${PC_MAJOR}`,
    setup: async (page) => {
      await page.getByRole('button', { name: 'Spells' }).first().click()
    },
  },
  { name: 'settings', path: '/settings' },
  // Secondary window entries (multi-page Vite inputs, loadable directly)
  { name: 'map-view', path: `/dm-map.html?moduleId=${MODULE_M05}&campaignId=${CAMPAIGN}` },
  { name: 'reference-reader', path: '/sources.html' },
]

export const VIEWPORTS = [
  { name: 'desktop', width: 1400, height: 900 }, // app default window size
  { name: 'narrow', width: 768, height: 900 },
]
