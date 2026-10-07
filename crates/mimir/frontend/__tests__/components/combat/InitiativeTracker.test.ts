/**
 * InitiativeTracker drawer in the DM Map window (MIMIR-T-0677).
 */
import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { flushPromises } from '@vue/test-utils'
import { mountWithPlugins } from '@tests/helpers/mountHelpers'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
  mockCommandError,
  expectCommandCalledWith,
} from '@tests/helpers/mockInvoke'
import InitiativeTracker from '@/features/combat/components/InitiativeTracker.vue'
import type { CombatEntry, CombatState } from '@/features/combat/types'

function entry(id: string, name: string, initiative: number | null, hp: [number, number] | null = null): CombatEntry {
  return {
    id,
    source_kind: 'custom',
    source_id: null,
    token_id: null,
    display_name: name,
    initiative,
    dex_modifier: null,
    max_hp: hp ? hp[1] : null,
    current_hp: hp ? hp[0] : null,
    temp_hp: 0,
    is_concentrating: false,
    conditions: [],
    damage_log: [],
  }
}

const FIGHT: CombatState = {
  session: { id: 's1', module_id: 'm1', round: 2, turn_index: 1, status: 'active', created_at: '', updated_at: '' },
  entries: [entry('pc', 'Thorin', 18), entry('g1', 'Goblin 1', 12, [7, 7]), entry('g2', 'Goblin 2', null, [3, 7])],
  current_entry_id: 'g1',
}

async function mountTracker() {
  const wrapper = mountWithPlugins(InitiativeTracker, {
    props: { moduleId: 'm1', campaignId: 'c1' },
    stubs: { AppModal: false },
  })
  await flushPromises()
  return wrapper
}

describe('InitiativeTracker', () => {
  beforeEach(() => {
    setupInvokeMock()
    localStorage.clear()
    mockCommand('list_module_monsters_with_data', [
      { id: 'mm-1', module_id: 'm1', monster_name: 'Goblin', monster_source: 'MM', homebrew_monster_id: null, quantity: 6, encounter_tag: null, display_name: null, notes: null, monster_data: null },
    ])
    mockCommand('list_pcs', [{ id: 'pc-1', name: 'Thorin' }])
  })
  afterEach(() => resetInvokeMock())

  it('offers to start combat when the module has none', async () => {
    mockCommand('get_active_combat', null)
    mockCommand('start_combat', FIGHT)
    const wrapper = await mountTracker()
    expectCommandCalledWith('get_active_combat', { moduleId: 'm1' })
    const start = wrapper.get('[data-testid="start-combat"]')
    await start.trigger('click')
    await flushPromises()
    expectCommandCalledWith('start_combat', { moduleId: 'm1' })
    expect(wrapper.findAll('[data-testid="combat-entry"]')).toHaveLength(3)
  })

  it('lists entries in turn order with the round and the current turn', async () => {
    mockCommand('get_active_combat', FIGHT)
    const wrapper = await mountTracker()
    const rows = wrapper.findAll('[data-testid="combat-entry"]')
    expect(rows.map((r) => r.get('.entry-name').text())).toEqual(['Thorin', 'Goblin 1', 'Goblin 2'])
    expect(rows[1].classes()).toContain('current')
    expect(rows[0].classes()).not.toContain('current')
    expect(wrapper.get('[data-testid="combat-round"]').text()).toContain('2')
    expect(rows[2].get('.entry-hp').text()).toBe('3/7')
    expect(rows[0].get('.entry-hp').text()).toBe('—')
  })

  it('advances and rewinds turns', async () => {
    mockCommand('get_active_combat', FIGHT)
    mockCommand('combat_next_turn', { ...FIGHT, current_entry_id: 'g2' })
    mockCommand('combat_previous_turn', FIGHT)
    const wrapper = await mountTracker()
    await wrapper.get('[data-testid="next-turn"]').trigger('click')
    await flushPromises()
    expectCommandCalledWith('combat_next_turn', { sessionId: 's1' })
    expect(wrapper.findAll('[data-testid="combat-entry"]')[2].classes()).toContain('current')
    await wrapper.get('[data-testid="previous-turn"]').trigger('click')
    await flushPromises()
    expectCommandCalledWith('combat_previous_turn', { sessionId: 's1' })
  })

  it('saves initiative edits; an empty field clears it', async () => {
    mockCommand('get_active_combat', FIGHT)
    mockCommand('set_combat_initiative', entry('g2', 'Goblin 2', 9))
    mockCommand('get_combat', FIGHT)
    const wrapper = await mountTracker()
    const input = wrapper.findAll('[data-testid="combat-entry"]')[2].get('input.entry-initiative')
    await input.setValue('9')
    await input.trigger('change')
    await flushPromises()
    expectCommandCalledWith('set_combat_initiative', { entryId: 'g2', initiative: 9 })
    await input.setValue('')
    await input.trigger('change')
    await flushPromises()
    expectCommandCalledWith('set_combat_initiative', { entryId: 'g2', initiative: null })
  })

  it('adds a monster group, a PC and a custom entry', async () => {
    mockCommand('get_active_combat', FIGHT)
    mockCommand('add_combat_module_monster', [])
    mockCommand('add_combat_character', entry('x', 'Thorin', null))
    mockCommand('add_combat_custom', entry('y', 'Lair action', null))
    mockCommand('get_combat', FIGHT)
    const wrapper = await mountTracker()

    const monsterSelect = wrapper.get('[data-testid="add-monster-select"]')
    expect(monsterSelect.text()).toContain('Goblin ×6')
    await monsterSelect.setValue('mm-1')
    await wrapper.get('[data-testid="add-monster"]').trigger('click')
    await flushPromises()
    expectCommandCalledWith('add_combat_module_monster', { sessionId: 's1', moduleMonsterId: 'mm-1', count: null })

    await wrapper.get('[data-testid="add-pc-select"]').setValue('pc-1')
    await wrapper.get('[data-testid="add-pc"]').trigger('click')
    await flushPromises()
    expectCommandCalledWith('add_combat_character', { sessionId: 's1', characterId: 'pc-1' })

    await wrapper.get('[data-testid="add-custom-name"]').setValue('Lair action')
    await wrapper.get('[data-testid="add-custom"]').trigger('click')
    await flushPromises()
    expectCommandCalledWith('add_combat_custom', { sessionId: 's1', name: 'Lair action', maxHp: null, initiative: null })
  })

  it('removes an entry', async () => {
    mockCommand('get_active_combat', FIGHT)
    mockCommand('remove_combat_entry', { ...FIGHT, entries: FIGHT.entries.slice(0, 2) })
    const wrapper = await mountTracker()
    await wrapper.findAll('[data-testid="combat-entry"]')[2].get('[data-testid="remove-entry"]').trigger('click')
    await flushPromises()
    expectCommandCalledWith('remove_combat_entry', { entryId: 'g2' })
    expect(wrapper.findAll('[data-testid="combat-entry"]')).toHaveLength(2)
  })

  it('asks before ending the combat', async () => {
    mockCommand('get_active_combat', FIGHT)
    mockCommand('end_combat', null)
    const wrapper = await mountTracker()
    await wrapper.get('[data-testid="end-combat"]').trigger('click')
    await flushPromises()
    expect(wrapper.text()).toContain('End combat?')
    await wrapper.get('[data-testid="confirm-end-combat"]').trigger('click')
    await flushPromises()
    expectCommandCalledWith('end_combat', { sessionId: 's1' })
    expect(wrapper.find('[data-testid="start-combat"]').exists()).toBe(true)
  })

  it('collapses and remembers it', async () => {
    mockCommand('get_active_combat', FIGHT)
    const wrapper = await mountTracker()
    await wrapper.get('[data-testid="toggle-tracker"]').trigger('click')
    expect(wrapper.find('[data-testid="combat-entry"]').exists()).toBe(false)
    expect(localStorage.getItem('mimir.initiativeTracker.collapsed')).toBe('true')

    const again = await mountTracker()
    expect(again.find('[data-testid="combat-entry"]').exists()).toBe(false)
  })

  it('shows backend errors', async () => {
    mockCommand('get_active_combat', FIGHT)
    mockCommandError('combat_next_turn', 'this combat has ended')
    const wrapper = await mountTracker()
    await wrapper.get('[data-testid="next-turn"]').trigger('click')
    await flushPromises()
    expect(wrapper.get('[data-testid="combat-error"]').text()).toContain('this combat has ended')
  })
})
