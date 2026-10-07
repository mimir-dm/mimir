import type { Page } from '@playwright/test'
import { FIXTURE, type FixtureIds } from './fixture'

/**
 * Design-review screen catalog: every main screen, rendered with the UI
 * fixture campaign (see fixture.ts). `setup` performs the in-page selection
 * needed for panes that only render after interaction (module detail,
 * document viewer, sheet sub-tabs).
 *
 * Paths take the fixture IDs resolved for the running session.
 */

export interface Screen {
  name: string
  path: (ids: FixtureIds) => string
  setup?: (page: Page) => Promise<void>
}

export const SCREENS: Screen[] = [
  { name: 'home', path: () => '/' },
  { name: 'dashboard-campaign-docs', path: (ids) => `/campaigns/${ids.campaign}/dashboard/campaign` },
  {
    name: 'document-viewer',
    path: (ids) => `/campaigns/${ids.campaign}/dashboard/campaign`,
    setup: async (page) => {
      await page.getByText(FIXTURE.campaignDocument).first().click()
    },
  },
  { name: 'dashboard-modules', path: (ids) => `/campaigns/${ids.campaign}/dashboard/modules` },
  {
    name: 'module-detail',
    path: (ids) => `/campaigns/${ids.campaign}/dashboard/modules`,
    setup: async (page) => {
      await page.getByText(FIXTURE.module).first().click()
    },
  },
  { name: 'dashboard-npcs', path: (ids) => `/campaigns/${ids.campaign}/dashboard/npcs` },
  { name: 'dashboard-pcs', path: (ids) => `/campaigns/${ids.campaign}/dashboard/pcs` },
  { name: 'dashboard-homebrew', path: (ids) => `/campaigns/${ids.campaign}/dashboard/homebrew` },
  {
    name: 'homebrew-monster',
    path: (ids) => `/campaigns/${ids.campaign}/dashboard/homebrew`,
    setup: async (page) => {
      await page.getByText('Monsters', { exact: true }).first().click()
      await page.waitForTimeout(400)
      const first = page.getByText(FIXTURE.homebrewMonster).first()
      if (await first.isVisible().catch(() => false)) await first.click()
    },
  },
  { name: 'characters', path: () => '/characters' },
  { name: 'character-sheet', path: (ids) => `/characters/${ids.spellcaster}` },
  {
    name: 'character-equipment',
    path: (ids) => `/characters/${ids.fighter}`,
    setup: async (page) => {
      await page.getByRole('button', { name: 'Equipment' }).first().click()
    },
  },
  {
    name: 'character-spells',
    path: (ids) => `/characters/${ids.spellcaster}`,
    setup: async (page) => {
      await page.getByRole('button', { name: 'Spells' }).first().click()
    },
  },
  { name: 'settings', path: () => '/settings' },
  // Secondary window entries (multi-page Vite inputs, loadable directly)
  { name: 'map-view', path: (ids) => `/dm-map.html?moduleId=${ids.module}&campaignId=${ids.campaign}` },
  {
    // Initiative tracker drawer with a fight running (COLLIERY-I-0468).
    name: 'dm-map-combat',
    path: (ids) => `/dm-map.html?moduleId=${ids.module}&campaignId=${ids.campaign}`,
    setup: async (page) => {
      const start = page.getByTestId('start-combat')
      if (await start.isVisible().catch(() => false)) {
        await start.click()
        await page.getByTestId('add-monster-select').selectOption({ index: 4 })
        await page.getByTestId('add-monster').click()
        await page.getByTestId('add-pc-select').selectOption({ index: 1 })
        await page.getByTestId('add-pc').click()
        // Wait for the six goblins and the PC before editing.
        await page.getByTestId('combat-entry').nth(6).waitFor()
        const first = page.getByTestId('combat-entry').first().locator('input.entry-initiative')
        await first.fill('17')
        await first.press('Tab')
        await page.getByTestId('next-turn').click()
        // Goblin 2: concentrating and prone, then hit for 3 -> save prompt
        // (MIMIR-T-0678, MIMIR-T-0679).
        await page.getByTestId('combat-entry').nth(2).locator('.entry-name').click()
        await page.getByTestId('concentration-toggle').check()
        await page.getByTestId('condition-select').selectOption('prone')
        await page.getByTestId('add-condition').click()
        await page.getByTestId('hp-amount').fill('3')
        await page.getByTestId('hp-damage').click()
        await page.getByTestId('concentration-prompt').waitFor()
      }
    },
  },
  { name: 'reference-reader', path: () => '/sources.html' },
]

export const VIEWPORTS = [
  { name: 'desktop', width: 1400, height: 900 }, // app default window size
  { name: 'narrow', width: 768, height: 900 },
]
