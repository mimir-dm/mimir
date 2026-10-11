import { CAMPAIGN, MODULE, TOKEN, campaignId, expect, mapId, moduleId, signIn, test } from './session'

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

type Ids = { campaign: string; module: string; map: string }
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
    name: 'dm-map',
    path: (i) => `/maps/${i.map}`,
    ready: async (p) => {
      await expect(p.locator('.mimir-map__svg image').first()).toBeAttached()
      await expect(p.locator('.mimir-map__token[aria-label="Klarg"]')).toBeAttached()
      // Let the fit and token art settle.
      await p.waitForTimeout(400)
    },
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
        const ids = { campaign: await campaignId(page), module: '', map: await mapId(page) }
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

// The player display with the DM showing the module map: desktop and
// tablet, in each theme (the display is dark by design; the controls
// follow the theme).
for (const theme of themes) {
  for (const size of sizes) {
    test(`player-display ${size.name} ${theme}`, async ({ page }) => {
      await page.setViewportSize({ width: size.width, height: size.height })
      const campaign = await campaignId(page)
      const map = await mapId(page)
      await page.request.put(`/api/v1/campaigns/${campaign}/display`, {
        headers: { Authorization: `Bearer ${TOKEN}` },
        data: { map_id: map },
      })
      await signIn(page, theme)
      await page.goto(`/display/${campaign}`)
      await expect(page.locator('.mimir-display__svg image')).toBeAttached()
      await expect(page.locator('.mimir-map__token[aria-label="Klarg"]')).toBeAttached()
      await page.waitForTimeout(400)
      await page.evaluate(() => document.fonts.ready)
      await page.screenshot({ path: `${outDir}/player-display--${size.name}--${theme}.png`, animations: 'disabled', caret: 'hide' })
    })
  }
}

// The DM map with a combat in the tracker, and the display with the order.
async function combatUp(page: import('@playwright/test').Page) {
  const auth = { Authorization: `Bearer ${TOKEN}` }
  const map = await mapId(page)
  const detail = await (await page.request.get(`/api/v1/maps/${map}`, { headers: auth })).json()
  const combat = await (await page.request.post(`/api/v1/modules/${detail.module_id}/combat`, { headers: auth })).json()
  if (combat.entries.length === 0) {
    const monsters = (await (await page.request.get(`/api/v1/modules/${detail.module_id}/monsters`, { headers: auth })).json()) as {
      id: string
      name: string
    }[]
    const klarg = monsters.find((m) => m.name === 'Klarg')!
    const id = combat.session.id
    await page.request.post(`/api/v1/combat/${id}/entries`, { headers: auth, data: { kind: 'monster', module_monster_id: klarg.id } })
    await page.request.post(`/api/v1/combat/${id}/entries`, { headers: auth, data: { kind: 'custom', name: 'Robin', max_hp: 24, initiative: 15 } })
    await page.request.post(`/api/v1/combat/${id}/link-tokens`, { headers: auth, data: { map_id: map } })
    const state = await (await page.request.get(`/api/v1/combat/${id}`, { headers: auth })).json()
    const k = state.entries.find((e: { name: string }) => e.name === 'Klarg')
    await page.request.patch(`/api/v1/combat-entries/${k.id}`, { headers: auth, data: { initiative: 12 } })
    const robin = state.entries.find((e: { name: string }) => e.name === 'Robin')
    await page.request.post(`/api/v1/combat-entries/${robin.id}/damage`, { headers: auth, data: { amount: 9 } })
  }
  return map
}

for (const theme of themes) {
  for (const size of sizes) {
    test(`dm-map-combat ${size.name} ${theme}`, async ({ page }) => {
      await page.setViewportSize({ width: size.width, height: size.height })
      const map = await combatUp(page)
      await signIn(page, theme)
      await page.goto(`/maps/${map}`)
      await expect(page.locator('.mimir-tracker__row', { hasText: 'Robin' })).toBeVisible()
      await expect(page.locator('.mimir-map__token[aria-label="Klarg"]')).toBeAttached()
      await page.waitForTimeout(400)
      await page.evaluate(() => document.fonts.ready)
      await page.screenshot({ path: `${outDir}/dm-map-combat--${size.name}--${theme}.png`, animations: 'disabled', caret: 'hide' })
    })
  }
  test(`player-display-order desktop ${theme}`, async ({ page }) => {
    const map = await combatUp(page)
    const campaign = await campaignId(page)
    await page.request.put(`/api/v1/campaigns/${campaign}/display`, {
      headers: { Authorization: `Bearer ${TOKEN}` },
      data: { map_id: map, show_initiative: true },
    })
    await signIn(page, theme)
    await page.goto(`/display/${campaign}`)
    await expect(page.locator('.mimir-display__order')).toContainText('Robin')
    await page.waitForTimeout(400)
    await page.evaluate(() => document.fonts.ready)
    await page.screenshot({ path: `${outDir}/player-display-order--desktop--${theme}.png`, animations: 'disabled', caret: 'hide' })
    await page.request.put(`/api/v1/campaigns/${campaign}/display`, {
      headers: { Authorization: `Bearer ${TOKEN}` },
      data: { map_id: map, show_initiative: false },
    })
  })
}

// The player page (a player link), with the DM showing the module map.
for (const theme of themes) {
  for (const size of sizes) {
    test(`player-page ${size.name} ${theme}`, async ({ page }) => {
      await page.setViewportSize({ width: size.width, height: size.height })
      const auth = { Authorization: `Bearer ${TOKEN}` }
      const campaign = await campaignId(page)
      const map = await mapId(page)
      await page.request.put(`/api/v1/campaigns/${campaign}/display`, { headers: auth, data: { map_id: map } })
      const pcs = await (await page.request.get(`/api/v1/campaigns/${campaign}/pcs`, { headers: auth })).json()
      const link = await (await page.request.post(`/api/v1/characters/${pcs[0].id}/link`, { headers: auth })).json()
      await page.addInitScript(
        ([t, th]) => {
          localStorage.setItem('aurora-theme', th)
          if (!localStorage.getItem('mimir-dm-token')) localStorage.setItem('mimir-dm-token', t)
        },
        [link.token, theme] as const,
      )
      await page.goto(link.path)
      await expect(page.locator('.mimir-player__bar')).toContainText(pcs[0].name)
      await expect(page.locator('.mimir-display__svg image')).toBeAttached()
      await page.waitForTimeout(400)
      await page.evaluate(() => document.fonts.ready)
      await page.screenshot({ path: `${outDir}/player-page--${size.name}--${theme}.png`, animations: 'disabled', caret: 'hide' })
    })
  }
}

// The character sheet (Character tab at both sizes; Equipment and the
// level-up dialog at desktop size).
async function pcId(page: import('@playwright/test').Page, name: string) {
  const campaign = await campaignId(page)
  const pcs = (await (
    await page.request.get(`/api/v1/campaigns/${campaign}/pcs`, { headers: { Authorization: `Bearer ${TOKEN}` } })
  ).json()) as { id: string; name: string }[]
  return pcs.find((p) => p.name === name)!.id
}

for (const theme of themes) {
  for (const size of sizes) {
    test(`character-sheet ${size.name} ${theme}`, async ({ page }) => {
      await page.setViewportSize({ width: size.width, height: size.height })
      const id = await pcId(page, 'Thorin Ironforge')
      await signIn(page, theme)
      await page.goto(`/characters/${id}`)
      await expect(page.locator('.mimir-sheet__ability')).toHaveCount(6)
      await page.evaluate(() => document.fonts.ready)
      await page.screenshot({ path: `${outDir}/character-sheet--${size.name}--${theme}.png`, animations: 'disabled', caret: 'hide' })
    })
  }
  test(`character-equipment desktop ${theme}`, async ({ page }) => {
    const id = await pcId(page, 'Thorin Ironforge')
    await signIn(page, theme)
    await page.goto(`/characters/${id}`)
    await page.getByRole('tab', { name: 'Equipment' }).click()
    await expect(page.locator('.mimir-inv__list')).toBeVisible()
    await page.mouse.move(0, 0)
    await page.evaluate(() => document.fonts.ready)
    await page.screenshot({ path: `${outDir}/character-equipment--desktop--${theme}.png`, animations: 'disabled', caret: 'hide' })
  })
  test(`character-levelup desktop ${theme}`, async ({ page }) => {
    const id = await pcId(page, 'Thorin Ironforge')
    await signIn(page, theme)
    await page.goto(`/characters/${id}`)
    await page.getByRole('button', { name: 'Level up' }).click()
    const dialog = page.getByRole('dialog')
    await expect(dialog.locator('.mimir-levelup__steps')).toContainText('Hit points')
    await page.evaluate(() => document.fonts.ready)
    await page.screenshot({ path: `${outDir}/character-levelup--desktop--${theme}.png`, animations: 'disabled', caret: 'hide' })
  })
}

// Player notes and the party (MIMIR-T-0718): the DM reads a note on the
// module and the sheet; a player sees the party with one shared sheet.
for (const theme of themes) {
  test(`notes desktop ${theme}`, async ({ page }) => {
    const auth = { Authorization: `Bearer ${TOKEN}` }
    const campaign = await campaignId(page)
    const module = await moduleId(page, campaign)
    const id = await pcId(page, 'Thorin Ironforge')
    const link = await (await page.request.post(`/api/v1/characters/${id}/link`, { headers: auth })).json()
    const note = await (
      await page.request.post(`/api/v1/characters/${id}/notes`, {
        headers: { Authorization: `Bearer ${link.token}` },
        data: {
          title: 'Ambush at the cave',
          content: 'Goblins on the ridge.\n\n1. Two at the stream\n2. A **wolf** pen inside',
          module_id: module,
        },
      })
    ).json()
    await signIn(page, theme)
    await page.goto(`/modules/${module}?section=notes`)
    await expect(page.locator('.mimir-notes__note')).toContainText('Thorin Ironforge')
    await page.evaluate(() => document.fonts.ready)
    await page.screenshot({ path: `${outDir}/module-notes--desktop--${theme}.png`, animations: 'disabled', caret: 'hide' })
    await page.goto(`/characters/${id}`)
    await page.getByRole('tab', { name: 'Notes' }).click()
    await expect(page.locator('.mimir-notes__note')).toBeVisible()
    await page.mouse.move(0, 0)
    await page.evaluate(() => document.fonts.ready)
    await page.screenshot({ path: `${outDir}/character-notes--desktop--${theme}.png`, animations: 'disabled', caret: 'hide' })
    await page.request.delete(`/api/v1/notes/${note.id}`, { headers: { Authorization: `Bearer ${link.token}` } })
    await page.request.delete(`/api/v1/characters/${id}/link`, { headers: auth })
  })
  for (const size of sizes) {
    test(`player-party ${size.name} ${theme}`, async ({ page }) => {
      await page.setViewportSize({ width: size.width, height: size.height })
      const auth = { Authorization: `Bearer ${TOKEN}` }
      const campaign = await campaignId(page)
      const pcs = await (await page.request.get(`/api/v1/campaigns/${campaign}/pcs`, { headers: auth })).json()
      const link = await (await page.request.post(`/api/v1/characters/${pcs[0].id}/link`, { headers: auth })).json()
      await page.request.put(`/api/v1/characters/${pcs[1].id}/sharing`, { headers: auth, data: { shared: true } })
      await page.addInitScript((th) => localStorage.setItem('aurora-theme', th), theme)
      await page.goto(link.path)
      await page.getByRole('tab', { name: 'Party' }).click()
      await expect(page.locator('.mimir-party__member').getByRole('button', { name: 'View' })).toBeVisible()
      await page.mouse.move(0, 0)
      await page.evaluate(() => document.fonts.ready)
      await page.screenshot({ path: `${outDir}/player-party--${size.name}--${theme}.png`, animations: 'disabled', caret: 'hide' })
      await page.request.put(`/api/v1/characters/${pcs[1].id}/sharing`, { headers: auth, data: { shared: false } })
      await page.request.delete(`/api/v1/characters/${pcs[0].id}/link`, { headers: auth })
    })
  }
}

// The rich editor (MIMIR-T-0719): a player writing a note.
for (const theme of themes) {
  for (const size of sizes) {
    test(`note-editor ${size.name} ${theme}`, async ({ page }) => {
      await page.setViewportSize({ width: size.width, height: size.height })
      const auth = { Authorization: `Bearer ${TOKEN}` }
      const id = await pcId(page, 'Thorin Ironforge')
      const link = await (await page.request.post(`/api/v1/characters/${id}/link`, { headers: auth })).json()
      await page.addInitScript((th) => localStorage.setItem('aurora-theme', th), theme)
      await page.goto(link.path)
      await page.getByRole('tab', { name: 'Character' }).click()
      await page.getByRole('tab', { name: 'Notes' }).click()
      await page.getByRole('button', { name: 'New note' }).click()
      await page.getByLabel('Title').fill('Ambush at the cave')
      await page.getByRole('textbox', { name: 'Note' }).click()
      await page.keyboard.type('## Who was there')
      await page.keyboard.press('Enter')
      await page.keyboard.type('1. Goblins on the ridge')
      await page.keyboard.press('Enter')
      await page.keyboard.type('A **wolf** pen')
      await page.mouse.move(0, 0)
      await page.evaluate(() => document.fonts.ready)
      await page.screenshot({ path: `${outDir}/note-editor--${size.name}--${theme}.png`, animations: 'disabled', caret: 'hide' })
      await page.request.delete(`/api/v1/characters/${id}/link`, { headers: auth })
    })
  }
}

// Authoring (MIMIR-T-0720): the new campaign dialog, a document in the
// editor, the map upload dialog, a module's monsters with a search.
for (const theme of themes) {
  test(`campaign-new desktop ${theme}`, async ({ page }) => {
    await signIn(page, theme)
    await page.goto('/')
    await page.getByRole('button', { name: 'New campaign' }).click()
    await page.getByRole('dialog').getByLabel('Name').fill('Storm King')
    await page.evaluate(() => document.fonts.ready)
    await page.screenshot({ path: `${outDir}/campaign-new--desktop--${theme}.png`, animations: 'disabled', caret: 'hide' })
  })
  for (const size of sizes) {
    test(`document-edit ${size.name} ${theme}`, async ({ page }) => {
      await page.setViewportSize({ width: size.width, height: size.height })
      const campaign = await campaignId(page)
      await signIn(page, theme)
      await page.goto(`/campaigns/${campaign}`)
      await page.getByRole('button', { name: 'Edit', exact: true }).click()
      await expect(page.getByRole('textbox', { name: 'Document' })).toBeVisible()
      await page.mouse.move(0, 0)
      await page.evaluate(() => document.fonts.ready)
      await page.screenshot({ path: `${outDir}/document-edit--${size.name}--${theme}.png`, animations: 'disabled', caret: 'hide' })
    })
  }
  test(`map-upload desktop ${theme}`, async ({ page }) => {
    const campaign = await campaignId(page)
    const module = await moduleId(page, campaign)
    await signIn(page, theme)
    await page.goto(`/modules/${module}?section=maps`)
    await page.getByRole('button', { name: 'Upload a map' }).click()
    await expect(page.getByRole('dialog')).toBeVisible()
    await page.evaluate(() => document.fonts.ready)
    await page.screenshot({ path: `${outDir}/map-upload--desktop--${theme}.png`, animations: 'disabled', caret: 'hide' })
  })
  test(`module-monsters-search desktop ${theme}`, async ({ page }) => {
    const campaign = await campaignId(page)
    const module = await moduleId(page, campaign)
    await signIn(page, theme)
    await page.goto(`/modules/${module}?section=dangers`)
    await page.getByLabel('Monster name').fill('goblin')
    await page.getByRole('button', { name: 'Search' }).click()
    await expect(page.getByRole('list', { name: 'Monsters found' })).toBeVisible()
    await page.mouse.move(0, 0)
    await page.evaluate(() => document.fonts.ready)
    await page.screenshot({ path: `${outDir}/module-monsters-search--desktop--${theme}.png`, animations: 'disabled', caret: 'hide' })
  })
}

// Character management (MIMIR-T-0721): the characters page, the wizard's
// ability scores (point buy).
for (const theme of themes) {
  for (const size of sizes) {
    test(`characters ${size.name} ${theme}`, async ({ page }) => {
      await page.setViewportSize({ width: size.width, height: size.height })
      await signIn(page, theme)
      await page.goto('/characters')
      await expect(page.getByRole('region', { name: CAMPAIGN })).toBeVisible()
      await page.evaluate(() => document.fonts.ready)
      await page.screenshot({ path: `${outDir}/characters--${size.name}--${theme}.png`, animations: 'disabled', caret: 'hide' })
    })
  }
  test(`wizard-abilities desktop ${theme}`, async ({ page }) => {
    await signIn(page, theme)
    await page.goto('/characters')
    await page.getByRole('button', { name: 'New character' }).click()
    const d = page.getByRole('dialog')
    await d.getByLabel('Name').fill('Robin Brightwater')
    await d.getByLabel('Player').fill('Sam')
    for (const label of ['', 'Race', 'Class', 'Background']) {
      if (label) {
        const select = d.getByLabel(label)
        await select.selectOption({ index: 1 })
      }
      await d.getByRole('button', { name: 'Next' }).click()
    }
    await d.getByLabel('Method').selectOption('point-buy')
    for (let i = 0; i < 5; i++) await d.getByRole('button', { name: 'Raise Strength' }).click()
    await page.evaluate(() => document.fonts.ready)
    await page.screenshot({ path: `${outDir}/wizard-abilities--desktop--${theme}.png`, animations: 'disabled', caret: 'hide' })
  })
}

// The reference (MIMIR-T-0723): the catalog, a stat block, a book.
for (const theme of themes) {
  for (const size of sizes) {
    test(`reference-catalog ${size.name} ${theme}`, async ({ page }) => {
      await page.setViewportSize({ width: size.width, height: size.height })
      await signIn(page, theme)
      await page.goto('/reference?mode=catalog&category=monsters')
      await expect(page.getByRole('table', { name: 'Results' }).locator('tbody tr').first()).toBeVisible()
      await page.evaluate(() => document.fonts.ready)
      await page.screenshot({ path: `${outDir}/reference-catalog--${size.name}--${theme}.png`, animations: 'disabled', caret: 'hide' })
    })
    test(`reference-book ${size.name} ${theme}`, async ({ page }) => {
      await page.setViewportSize({ width: size.width, height: size.height })
      await signIn(page, theme)
      await page.goto('/reference?mode=reader&book=DMG&section=0')
      await expect(page.locator('.mimir-book .image-wrapper img')).toBeVisible()
      await page.evaluate(() => document.fonts.ready)
      await page.screenshot({ path: `${outDir}/reference-book--${size.name}--${theme}.png`, animations: 'disabled', caret: 'hide' })
    })
  }
  test(`reference-statblock desktop ${theme}`, async ({ page }) => {
    await signIn(page, theme)
    await page.goto('/reference?mode=catalog&category=monsters')
    await page.getByLabel('Name').fill('goblin')
    await page.getByRole('table', { name: 'Results' }).getByRole('button', { name: 'Goblin', exact: true }).click()
    await expect(page.getByRole('dialog', { name: 'Goblin' })).toContainText('CR 1/4')
    await page.mouse.move(0, 0)
    await page.evaluate(() => document.fonts.ready)
    await page.screenshot({ path: `${outDir}/reference-statblock--desktop--${theme}.png`, animations: 'disabled', caret: 'hide' })
  })
}

// Homebrew (MIMIR-T-0722): the tab with a monster, the item editor.
for (const theme of themes) {
  test(`homebrew desktop ${theme}`, async ({ page }) => {
    const campaign = await campaignId(page)
    await signIn(page, theme)
    await page.goto(`/campaigns/${campaign}/homebrew?kind=monsters`)
    await page.getByRole('navigation', { name: 'Homebrew' }).getByRole('link').first().click()
    await expect(page.locator('.mimir-split__main .detail')).toBeVisible()
    await page.mouse.move(0, 0)
    await page.evaluate(() => document.fonts.ready)
    await page.screenshot({ path: `${outDir}/homebrew--desktop--${theme}.png`, animations: 'disabled', caret: 'hide' })
  })
  test(`homebrew-editor desktop ${theme}`, async ({ page }) => {
    const campaign = await campaignId(page)
    await signIn(page, theme)
    await page.goto(`/campaigns/${campaign}/homebrew?kind=items`)
    await page.getByRole('button', { name: 'New' }).click()
    const d = page.getByRole('dialog')
    await d.getByLabel('Name').fill('Flame Tongue Shortsword')
    await d.getByLabel('Kind').selectOption('weapon')
    await d.getByLabel('Rarity').selectOption('rare')
    await d.getByLabel('Damage', { exact: true }).fill('1d6')
    await d.getByLabel('Damage type').selectOption('P')
    await page.mouse.move(0, 0)
    await page.evaluate(() => document.fonts.ready)
    await page.screenshot({ path: `${outDir}/homebrew-editor--desktop--${theme}.png`, animations: 'disabled', caret: 'hide' })
  })
}

// Settings (MIMIR-T-0724): campaigns, books.
for (const theme of themes) {
  for (const section of ['campaigns', 'books']) {
    test(`settings-${section} desktop ${theme}`, async ({ page }) => {
      await signIn(page, theme)
      await page.goto(`/settings?section=${section}`)
      await expect(page.getByRole('table').first()).toBeVisible()
      await page.evaluate(() => document.fonts.ready)
      await page.screenshot({ path: `${outDir}/settings-${section}--desktop--${theme}.png`, animations: 'disabled', caret: 'hide' })
    })
  }
}
