/**
 * useCombatSession: loads, starts and drives a module's combat through the
 * combat commands (MIMIR-T-0677).
 */
import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
  mockCommandError,
  expectCommandCalledWith,
} from '@tests/helpers/mockInvoke'
import { useCombatSession } from '@/features/combat/composables/useCombatSession'
import type { CombatState, CombatEntry } from '@/features/combat/types'

function entry(id: string, name: string, initiative: number | null = null): CombatEntry {
  return {
    id,
    source_kind: 'custom',
    source_id: null,
    token_id: null,
    display_name: name,
    initiative,
    dex_modifier: null,
    max_hp: null,
    current_hp: null,
    temp_hp: 0,
    is_concentrating: false,
    conditions: [],
    damage_log: [],
  }
}

function state(round = 1, entries: CombatEntry[] = [], current: string | null = null): CombatState {
  return {
    session: {
      id: 's1',
      module_id: 'm1',
      round,
      turn_index: 0,
      status: 'active',
      created_at: '',
      updated_at: '',
    },
    entries,
    current_entry_id: current,
  }
}

describe('useCombatSession', () => {
  beforeEach(() => setupInvokeMock())
  afterEach(() => resetInvokeMock())

  it('loads the active combat, or none', async () => {
    mockCommand('get_active_combat', null)
    const combat = useCombatSession('m1')
    await combat.load()
    expectCommandCalledWith('get_active_combat', { moduleId: 'm1' })
    expect(combat.state.value).toBeNull()

    mockCommand('get_active_combat', state(2, [entry('a', 'A')], 'a'))
    await combat.load()
    expect(combat.round.value).toBe(2)
    expect(combat.currentEntry.value?.display_name).toBe('A')
  })

  it('starts a combat for the module', async () => {
    mockCommand('start_combat', state())
    const combat = useCombatSession('m1')
    await combat.start()
    expectCommandCalledWith('start_combat', { moduleId: 'm1' })
    expect(combat.state.value?.session.id).toBe('s1')
  })

  it('advances and rewinds turns from the returned state', async () => {
    mockCommand('start_combat', state())
    mockCommand('combat_next_turn', state(2, [entry('a', 'A')], 'a'))
    mockCommand('combat_previous_turn', state(1, [entry('a', 'A')], 'a'))
    const combat = useCombatSession('m1')
    await combat.start()
    await combat.nextTurn()
    expectCommandCalledWith('combat_next_turn', { sessionId: 's1' })
    expect(combat.round.value).toBe(2)
    await combat.previousTurn()
    expect(combat.round.value).toBe(1)
  })

  it('adds entries and refreshes the order', async () => {
    mockCommand('start_combat', state())
    mockCommand('add_combat_module_monster', [entry('g1', 'Goblin 1')])
    mockCommand('add_combat_character', entry('pc', 'Thorin'))
    mockCommand('add_combat_custom', entry('c', 'Lair action', 20))
    mockCommand('get_combat', state(1, [entry('c', 'Lair action', 20), entry('g1', 'Goblin 1')]))
    const combat = useCombatSession('m1')
    await combat.start()

    await combat.addMonsterGroup('mm-1')
    expectCommandCalledWith('add_combat_module_monster', { sessionId: 's1', moduleMonsterId: 'mm-1', count: null })
    await combat.addCharacter('pc-1')
    expectCommandCalledWith('add_combat_character', { sessionId: 's1', characterId: 'pc-1' })
    await combat.addCustom('Lair action', null, 20)
    expectCommandCalledWith('add_combat_custom', { sessionId: 's1', name: 'Lair action', maxHp: null, initiative: 20 })
    expectCommandCalledWith('get_combat', { sessionId: 's1' })
    expect(combat.state.value?.entries.map((e) => e.display_name)).toEqual(['Lair action', 'Goblin 1'])
  })

  it('sets initiative (empty clears it) and removes entries', async () => {
    mockCommand('start_combat', state())
    mockCommand('set_combat_initiative', entry('a', 'A', 15))
    mockCommand('get_combat', state(1, [entry('a', 'A', 15)]))
    mockCommand('remove_combat_entry', state(1, []))
    const combat = useCombatSession('m1')
    await combat.start()

    await combat.setInitiative('a', 15)
    expectCommandCalledWith('set_combat_initiative', { entryId: 'a', initiative: 15 })
    await combat.setInitiative('a', null)
    expectCommandCalledWith('set_combat_initiative', { entryId: 'a', initiative: null })
    await combat.removeEntry('a')
    expect(combat.state.value?.entries).toEqual([])
  })

  it('ends the combat', async () => {
    mockCommand('start_combat', state())
    mockCommand('end_combat', null)
    const combat = useCombatSession('m1')
    await combat.start()
    await combat.end()
    expectCommandCalledWith('end_combat', { sessionId: 's1' })
    expect(combat.state.value).toBeNull()
  })

  it('keeps the state and reports the error when a command fails', async () => {
    mockCommand('start_combat', state(3))
    mockCommandError('combat_next_turn', 'this combat has ended')
    const combat = useCombatSession('m1')
    await combat.start()
    await combat.nextTurn()
    expect(combat.error.value).toContain('this combat has ended')
    expect(combat.round.value).toBe(3)
  })
})
