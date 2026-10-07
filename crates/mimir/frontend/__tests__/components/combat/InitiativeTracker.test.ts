/**
 * InitiativeTracker drawer in the DM Map window (MIMIR-T-0677).
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'

const eventMock = vi.hoisted(() => ({
  emit: vi.fn(async () => {}),
  listeners: new Map<string, (event: { payload: unknown }) => void>(),
}))
vi.mock('@tauri-apps/api/event', () => ({
  emit: eventMock.emit,
  listen: vi.fn(async (name: string, handler: (event: { payload: unknown }) => void) => {
    eventMock.listeners.set(name, handler)
    return () => eventMock.listeners.delete(name)
  }),
}))
import { flushPromises } from '@vue/test-utils'
import { mountWithPlugins } from '@tests/helpers/mountHelpers'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
  mockCommandError,
  expectCommandCalledWith,
  getInvokeMock,
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

async function mountTracker(extraProps: Record<string, unknown> = {}) {
  const wrapper = mountWithPlugins(InitiativeTracker, {
    props: { moduleId: 'm1', campaignId: 'c1', ...extraProps },
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

  describe('HP panel (MIMIR-T-0678)', () => {
    async function expandGoblin2() {
      mockCommand('get_active_combat', FIGHT)
      const wrapper = await mountTracker()
      await wrapper.findAll('[data-testid="combat-entry"]')[2].get('.entry-name').trigger('click')
      return wrapper
    }

    it('expands one entry at a time', async () => {
      const wrapper = await expandGoblin2()
      expect(wrapper.findAll('[data-testid="entry-details"]')).toHaveLength(1)
      await wrapper.findAll('[data-testid="combat-entry"]')[1].get('.entry-name').trigger('click')
      const details = wrapper.findAll('[data-testid="entry-details"]')
      expect(details).toHaveLength(1)
      expect(wrapper.findAll('[data-testid="combat-entry"]')[1].find('[data-testid="entry-details"]').exists()).toBe(true)
      // Clicking the open entry closes it.
      await wrapper.findAll('[data-testid="combat-entry"]')[1].get('.entry-name').trigger('click')
      expect(wrapper.find('[data-testid="entry-details"]').exists()).toBe(false)
    })

    it('shows an HP bar sized to current/max', async () => {
      const wrapper = await expandGoblin2()
      const bar = wrapper.get('[data-testid="hp-bar-fill"]')
      expect(bar.attributes('style')).toContain('width: 43%') // 3 of 7
    })

    it('applies damage from the button and from Enter, and heals', async () => {
      mockCommand('combat_damage', { entry: { ...FIGHT.entries[2], current_hp: 1 }, concentration_dc: null })
      mockCommand('combat_heal', { ...FIGHT.entries[2], current_hp: 5 })
      const wrapper = await expandGoblin2()
      const amount = wrapper.get('[data-testid="hp-amount"]')

      await amount.setValue('2')
      await wrapper.get('[data-testid="hp-damage"]').trigger('click')
      await flushPromises()
      expectCommandCalledWith('combat_damage', { entryId: 'g2', amount: 2 })
      expect(wrapper.findAll('[data-testid="combat-entry"]')[2].get('.entry-hp').text()).toBe('1/7')
      expect((amount.element as HTMLInputElement).value).toBe('')

      await amount.setValue('3')
      await amount.trigger('keydown', { key: 'Enter' })
      await flushPromises()
      expectCommandCalledWith('combat_damage', { entryId: 'g2', amount: 3 })

      await amount.setValue('4')
      await wrapper.get('[data-testid="hp-heal"]').trigger('click')
      await flushPromises()
      expectCommandCalledWith('combat_heal', { entryId: 'g2', amount: 4 })
    })

    it('ignores empty or invalid amounts', async () => {
      const wrapper = await expandGoblin2()
      await wrapper.get('[data-testid="hp-amount"]').setValue('')
      expect(wrapper.get('[data-testid="hp-damage"]').attributes('disabled')).toBeDefined()
      expect(wrapper.get('[data-testid="hp-heal"]').attributes('disabled')).toBeDefined()
    })

    it('sets temporary HP', async () => {
      mockCommand('set_combat_temp_hp', { ...FIGHT.entries[2], temp_hp: 5 })
      const wrapper = await expandGoblin2()
      const temp = wrapper.get('[data-testid="temp-hp"]')
      await temp.setValue('5')
      await temp.trigger('change')
      await flushPromises()
      expectCommandCalledWith('set_combat_temp_hp', { entryId: 'g2', amount: 5 })
      expect(wrapper.findAll('[data-testid="combat-entry"]')[2].get('.entry-hp').text()).toBe('3/7 +5')
    })

    it('lets an entry without HP get max HP', async () => {
      mockCommand('get_active_combat', FIGHT)
      mockCommand('set_combat_max_hp', { ...FIGHT.entries[0], max_hp: 44, current_hp: 44 })
      const wrapper = await mountTracker()
      await wrapper.findAll('[data-testid="combat-entry"]')[0].get('.entry-name').trigger('click')
      expect(wrapper.find('[data-testid="hp-damage"]').exists()).toBe(false)
      await wrapper.get('[data-testid="set-max-hp"]').setValue('44')
      await wrapper.get('[data-testid="save-max-hp"]').trigger('click')
      await flushPromises()
      expectCommandCalledWith('set_combat_max_hp', { entryId: 'pc', maxHp: 44 })
      expect(wrapper.findAll('[data-testid="combat-entry"]')[0].get('.entry-hp').text()).toBe('44/44')
    })

    it('lists the recent HP changes, newest first', async () => {
      const hurt: CombatState = {
        ...FIGHT,
        entries: FIGHT.entries.map((e) =>
          e.id === 'g2'
            ? { ...e, damage_log: [{ kind: 'damage', amount: 6, round: 1 }, { kind: 'heal', amount: 2, round: 2 }] }
            : e,
        ),
      }
      mockCommand('get_active_combat', hurt)
      const wrapper = await mountTracker()
      await wrapper.findAll('[data-testid="combat-entry"]')[2].get('.entry-name').trigger('click')
      const items = wrapper.findAll('[data-testid="hp-log"] li').map((li) => li.text())
      expect(items).toEqual(['+2 (round 2)', '−6 (round 1)'])
    })
  })

  describe('conditions and concentration (MIMIR-T-0679)', () => {
    const AFFECTED: CombatState = {
      ...FIGHT,
      session: { ...FIGHT.session, round: 2 },
      entries: FIGHT.entries.map((e) =>
        e.id === 'g1'
          ? {
              ...e,
              is_concentrating: true,
              conditions: [
                { name: 'prone', expires_round: null },
                { name: 'frightened', expires_round: 3 },
              ],
            }
          : e,
      ),
    }

    async function mountAffected() {
      mockCommand('get_active_combat', AFFECTED)
      return mountTracker()
    }

    function row(wrapper: Awaited<ReturnType<typeof mountTracker>>, i: number) {
      return wrapper.findAll('[data-testid="combat-entry"]')[i]
    }

    it('shows condition pills and a concentration badge on the row', async () => {
      const wrapper = await mountAffected()
      const pills = row(wrapper, 1).findAll('[data-testid="condition-pill"]')
      expect(pills.map((p) => p.text())).toEqual(['prone', 'frightened'])
      expect(pills[1].attributes('title')).toContain('until the end of round 3')
      expect(row(wrapper, 1).find('[data-testid="concentration-badge"]').exists()).toBe(true)
      expect(row(wrapper, 0).find('[data-testid="concentration-badge"]').exists()).toBe(false)
    })

    it('removes a condition by clicking its pill', async () => {
      mockCommand('remove_combat_condition', { ...AFFECTED.entries[1], conditions: [{ name: 'frightened', expires_round: 3 }] })
      const wrapper = await mountAffected()
      await row(wrapper, 1).findAll('[data-testid="condition-pill"]')[0].trigger('click')
      await flushPromises()
      expectCommandCalledWith('remove_combat_condition', { entryId: 'g1', name: 'prone' })
      expect(row(wrapper, 1).findAll('[data-testid="condition-pill"]')).toHaveLength(1)
    })

    it('adds a condition with an optional duration', async () => {
      mockCommand('add_combat_condition', { ...FIGHT.entries[2], conditions: [{ name: 'poisoned', expires_round: 3 }] })
      mockCommand('get_active_combat', FIGHT)
      const wrapper = await mountTracker()
      await row(wrapper, 2).get('.entry-name').trigger('click')
      expect(wrapper.get('[data-testid="condition-select"]').findAll('option').length).toBe(16) // placeholder + 15
      await wrapper.get('[data-testid="condition-select"]').setValue('poisoned')
      await wrapper.get('[data-testid="condition-duration"]').setValue('2')
      await wrapper.get('[data-testid="add-condition"]').trigger('click')
      await flushPromises()
      expectCommandCalledWith('add_combat_condition', { entryId: 'g2', name: 'poisoned', durationRounds: 2 })

      await wrapper.get('[data-testid="condition-select"]').setValue('prone')
      await wrapper.get('[data-testid="condition-duration"]').setValue('')
      await wrapper.get('[data-testid="add-condition"]').trigger('click')
      await flushPromises()
      expectCommandCalledWith('add_combat_condition', { entryId: 'g2', name: 'prone', durationRounds: null })
    })

    it('conditions work on entries without HP', async () => {
      mockCommand('get_active_combat', FIGHT)
      const wrapper = await mountTracker()
      await row(wrapper, 0).get('.entry-name').trigger('click')
      expect(wrapper.find('[data-testid="condition-select"]').exists()).toBe(true)
    })

    it('toggles concentration', async () => {
      mockCommand('set_combat_concentration', { ...FIGHT.entries[2], is_concentrating: true })
      mockCommand('get_active_combat', FIGHT)
      const wrapper = await mountTracker()
      await row(wrapper, 2).get('.entry-name').trigger('click')
      await wrapper.get('[data-testid="concentration-toggle"]').setValue(true)
      await flushPromises()
      expectCommandCalledWith('set_combat_concentration', { entryId: 'g2', concentrating: true })
      expect(row(wrapper, 2).find('[data-testid="concentration-badge"]').exists()).toBe(true)
    })

    it('asks for a concentration save after damage; Lost ends concentration', async () => {
      mockCommand('combat_damage', {
        entry: { ...AFFECTED.entries[1], current_hp: 2 },
        concentration_dc: 12,
      })
      mockCommand('set_combat_concentration', { ...AFFECTED.entries[1], current_hp: 2, is_concentrating: false })
      const wrapper = await mountAffected()
      await row(wrapper, 1).get('.entry-name').trigger('click')
      await wrapper.get('[data-testid="hp-amount"]').setValue('5')
      await wrapper.get('[data-testid="hp-damage"]').trigger('click')
      await flushPromises()
      expect(wrapper.get('[data-testid="concentration-prompt"]').text()).toContain('Concentration save DC 12')

      await wrapper.get('[data-testid="concentration-lost"]').trigger('click')
      await flushPromises()
      expectCommandCalledWith('set_combat_concentration', { entryId: 'g1', concentrating: false })
      expect(wrapper.find('[data-testid="concentration-prompt"]').exists()).toBe(false)
      expect(row(wrapper, 1).find('[data-testid="concentration-badge"]').exists()).toBe(false)
    })

    it('Kept closes the prompt and keeps concentration', async () => {
      mockCommand('combat_damage', { entry: { ...AFFECTED.entries[1], current_hp: 2 }, concentration_dc: 10 })
      const wrapper = await mountAffected()
      await row(wrapper, 1).get('.entry-name').trigger('click')
      await wrapper.get('[data-testid="hp-amount"]').setValue('2')
      await wrapper.get('[data-testid="hp-damage"]').trigger('click')
      await flushPromises()
      await wrapper.get('[data-testid="concentration-kept"]').trigger('click')
      expect(wrapper.find('[data-testid="concentration-prompt"]').exists()).toBe(false)
      expect(row(wrapper, 1).find('[data-testid="concentration-badge"]').exists()).toBe(true)
    })
  })

  describe('map link (MIMIR-T-0680)', () => {
    function monster(id: string, name: string, tokenId: string | null, hp: [number, number] = [7, 7]): CombatEntry {
      return { ...entry(id, name, 10, hp), source_kind: 'module_monster', source_id: 'mm-1', token_id: tokenId }
    }

    const LINKED: CombatState = {
      ...FIGHT,
      entries: [entry('pc', 'Thorin', 18), monster('g1', 'Goblin 1', 'tok-1'), monster('g2', 'Goblin 2', 'tok-2', [0, 7])],
      current_entry_id: 'g1',
    }

    beforeEach(() => {
      eventMock.emit.mockClear()
      eventMock.listeners.clear()
    })

    it('links unlinked monsters to tokens on the map, once per set of unlinked entries', async () => {
      const unlinked = { ...LINKED, entries: [monster('g1', 'Goblin 1', null), monster('g2', 'Goblin 2', null)] }
      mockCommand('get_active_combat', unlinked)
      mockCommand('link_combat_tokens', { ...unlinked, entries: [monster('g1', 'Goblin 1', 'tok-1'), monster('g2', 'Goblin 2', null)] })
      const wrapper = await mountTracker({ mapId: 'map-1' })
      await flushPromises()
      expectCommandCalledWith('link_combat_tokens', { sessionId: 's1', mapId: 'map-1' })
      const calls = () => getInvokeMock().mock.calls.filter(([c]) => c === 'link_combat_tokens').length
      // Goblin 2 has no token: one more try for the smaller set, then no more.
      expect(calls()).toBe(2)
      await wrapper.setProps({ visibleTokenIds: ['tok-1'] })
      await flushPromises()
      expect(calls()).toBe(2)
      expect(wrapper.findAll('.linked')).toHaveLength(1)
    })

    it('does not link without a map', async () => {
      mockCommand('get_active_combat', { ...LINKED, entries: [monster('g1', 'Goblin 1', null)] })
      await mountTracker()
      expect(getInvokeMock().mock.calls.some(([c]) => c === 'link_combat_tokens')).toBe(false)
    })

    it('reports the current-turn and down tokens to the map', async () => {
      mockCommand('get_active_combat', LINKED)
      const wrapper = await mountTracker({ mapId: 'map-1' })
      const states = wrapper.emitted('map-state')!
      expect(states[states.length - 1]).toEqual([{ currentTurnTokenId: 'tok-1', downTokenIds: ['tok-2'] }])
    })

    it('highlights the entry of the token selected on the map and selects the token of an opened entry', async () => {
      mockCommand('get_active_combat', LINKED)
      const wrapper = await mountTracker({ mapId: 'map-1', selectedTokenId: 'tok-2' })
      const rows = wrapper.findAll('[data-testid="combat-entry"]')
      expect(rows[2].classes()).toContain('selected')
      expect(rows[1].classes()).not.toContain('selected')

      await rows[1].get('.entry-name').trigger('click')
      expect(wrapper.emitted('select-token')).toEqual([['tok-1']])
      await rows[1].get('.entry-name').trigger('click')
      expect(wrapper.emitted('select-token')).toEqual([['tok-1'], [null]])
      // An entry with no token selects nothing.
      await rows[0].get('.entry-name').trigger('click')
      expect(wrapper.emitted('select-token')).toHaveLength(2)
    })

    it('shows the order to players on request: names only, visible tokens only', async () => {
      mockCommand('get_active_combat', LINKED)
      const wrapper = await mountTracker({ mapId: 'map-1', visibleTokenIds: ['tok-1'] })
      expect(eventMock.emit).not.toHaveBeenCalled()

      await wrapper.get('[data-testid="show-order-to-players"]').setValue(true)
      await flushPromises()
      expect(eventMock.emit).toHaveBeenLastCalledWith('player-display:initiative-update', {
        visible: true,
        round: 2,
        entries: [
          { name: 'Thorin', current: false },
          { name: 'Goblin 1', current: true },
        ],
      })

      // The player display asks again when it loads a map.
      eventMock.emit.mockClear()
      eventMock.listeners.get('player-display:request-state')!({ payload: { mapId: 'map-1' } })
      expect(eventMock.emit).toHaveBeenCalledTimes(1)

      await wrapper.get('[data-testid="show-order-to-players"]').setValue(false)
      await flushPromises()
      expect(eventMock.emit).toHaveBeenLastCalledWith('player-display:initiative-update', {
        visible: false,
        round: null,
        entries: [],
      })
    })
  })
})
