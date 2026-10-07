/**
 * The tracker's map marks and the players' turn order (MIMIR-T-0680).
 */
import { describe, it, expect } from 'vitest'
import { HIDDEN_ORDER, combatMapState, playerInitiativeOrder } from '@/features/combat/mapLink'
import type { CombatEntry, CombatSourceKind, CombatState } from '@/features/combat/types'

function entry(id: string, kind: CombatSourceKind, tokenId: string | null, hp: number | null = 7): CombatEntry {
  return {
    id,
    source_kind: kind,
    source_id: kind === 'custom' ? null : `src-${id}`,
    token_id: tokenId,
    display_name: id,
    initiative: null,
    dex_modifier: null,
    max_hp: hp === null ? null : 7,
    current_hp: hp,
    temp_hp: 0,
    is_concentrating: false,
    conditions: [],
    damage_log: [],
  }
}

const STATE: CombatState = {
  session: { id: 's', module_id: 'm', round: 3, turn_index: 1, status: 'active', created_at: '', updated_at: '' },
  entries: [
    entry('Thorin', 'character', null, null),
    entry('Goblin 1', 'module_monster', 'tok-g1'),
    entry('Goblin 2', 'module_monster', 'tok-g2', 0),
    entry('Hidden Ogre', 'module_monster', 'tok-ogre'),
    entry('Unplaced Wolf', 'module_monster', null),
    entry('Sildar', 'module_npc', 'tok-sildar', 0),
    entry('Lair action', 'custom', null, null),
  ],
  current_entry_id: 'Goblin 1',
}

describe('combatMapState', () => {
  it('gives the token of the current turn and the tokens at 0 HP', () => {
    expect(combatMapState(STATE)).toEqual({
      currentTurnTokenId: 'tok-g1',
      downTokenIds: ['tok-g2', 'tok-sildar'],
    })
  })

  it('marks nothing without a fight or when the current entry has no token', () => {
    expect(combatMapState(null)).toEqual({ currentTurnTokenId: null, downTokenIds: [] })
    expect(combatMapState({ ...STATE, current_entry_id: 'Thorin' }).currentTurnTokenId).toBeNull()
  })
})

describe('playerInitiativeOrder', () => {
  it('names only; monsters and NPCs only with a token the players can see', () => {
    const order = playerInitiativeOrder(STATE, ['tok-g1', 'tok-g2', 'tok-sildar'])
    expect(order).toEqual({
      visible: true,
      round: 3,
      entries: [
        { name: 'Thorin', current: false },
        { name: 'Goblin 1', current: true },
        { name: 'Goblin 2', current: false },
        { name: 'Sildar', current: false },
        { name: 'Lair action', current: false },
      ],
    })
    // No HP or ids leak into the payload.
    expect(JSON.stringify(order)).not.toMatch(/hp|tok-/)
  })

  it('hides every map creature when no token is visible', () => {
    const names = playerInitiativeOrder(STATE, []).entries.map((e) => e.name)
    expect(names).toEqual(['Thorin', 'Lair action'])
    expect(HIDDEN_ORDER.visible).toBe(false)
  })
})
