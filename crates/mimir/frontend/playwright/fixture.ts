/**
 * The UI fixture the harness runs against (MIMIR-T-0663).
 *
 * Development machines hold no real campaign data. Each harness session's
 * bridge seeds a fresh scratch DB with the SRD catalog and the "Lost Mine of
 * Phandelver" dev campaign (mimir_core::seed::seed_ui_fixture). IDs are new
 * every session, so specs refer to fixture entities by NAME and resolve IDs
 * through the bridge with `resolveFixture()`.
 */

/** Names of fixture entities the specs use. Keep in sync with seed/dev.rs. */
export const FIXTURE = {
  campaign: 'The Lost Mine of Phandelver',
  module: 'Cragmaw Hideout',
  campaignDocument: 'Campaign Pitch',
  homebrewMonster: 'Cragmaw Mutant',
  /** Cleric 7: spells + inventory */
  spellcaster: 'Sister Helena',
  /** Fighter 5: full equipment */
  fighter: 'Thorin Ironforge',
} as const

export interface FixtureIds {
  campaign: string
  module: string
  spellcaster: string
  fighter: string
}

const BRIDGE = `http://127.0.0.1:${process.env.MIMIR_BRIDGE_PORT ?? '4175'}`

async function invoke<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  const res = await fetch(`${BRIDGE}/invoke/${cmd}`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(args),
  })
  const body = (await res.json()) as { success: boolean; data?: T; error?: string }
  if (!res.ok || !body.success || body.data === undefined) {
    throw new Error(`bridge ${cmd} failed: ${body.error ?? res.status}`)
  }
  return body.data
}

function byName<T extends { id: string; name: string }>(items: T[], name: string, what: string): string {
  const hit = items.find((i) => i.name === name)
  if (!hit) {
    throw new Error(
      `fixture ${what} '${name}' not found (have: ${items.map((i) => i.name).join(', ')}). ` +
        'Is the bridge running with MIMIR_BRIDGE_SEED=fixture?',
    )
  }
  return hit.id
}

/** Look up the fixture's IDs in the running bridge's DB. */
export async function resolveFixture(): Promise<FixtureIds> {
  type Named = { id: string; name: string }
  const campaign = byName(await invoke<Named[]>('list_campaigns'), FIXTURE.campaign, 'campaign')
  const modules = await invoke<Named[]>('list_modules', { campaignId: campaign })
  const characters = await invoke<Named[]>('list_characters', { campaignId: campaign })
  return {
    campaign,
    module: byName(modules, FIXTURE.module, 'module'),
    spellcaster: byName(characters, FIXTURE.spellcaster, 'character'),
    fighter: byName(characters, FIXTURE.fighter, 'character'),
  }
}
