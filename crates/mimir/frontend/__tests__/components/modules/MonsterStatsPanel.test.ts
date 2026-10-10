/**
 * MonsterStatsPanel title and alias (MIMIR-T-0700).
 */
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import MonsterStatsPanel from '@/features/modules/components/MonsterStatsPanel.vue'
import type { MonsterWithData } from '@/features/modules/composables/useModuleMonsters'

function monster(over: Partial<MonsterWithData>): MonsterWithData {
  return {
    id: 'm1',
    module_id: 'mod',
    monster_name: null,
    monster_source: null,
    homebrew_monster_id: null,
    quantity: 1,
    encounter_tag: null,
    display_name: null,
    notes: null,
    monster_data: null,
    ...over,
  } as MonsterWithData
}

const title = (w: ReturnType<typeof mount>) => w.get('h2').text()
const alias = (w: ReturnType<typeof mount>) => w.find('[data-testid="monster-alias"]')

describe('MonsterStatsPanel title', () => {
  it('shows the catalog name as the alias of a display name', () => {
    const w = mount(MonsterStatsPanel, { props: { monster: monster({ display_name: 'Klarg', monster_name: 'Bugbear' }) } })
    expect(title(w)).toBe('Klarg')
    expect(alias(w).text()).toBe('(Bugbear)')
  })

  it('shows no empty "()" for a homebrew monster with a display name', () => {
    const w = mount(MonsterStatsPanel, {
      props: { monster: monster({ display_name: 'Cragmaw Mutants', homebrew_monster_id: 'h1', monster_data: { name: 'Cragmaw Mutant' } }) },
    })
    expect(title(w)).toBe('Cragmaw Mutants')
    expect(alias(w).text()).toBe('(Cragmaw Mutant)')
    const bare = mount(MonsterStatsPanel, { props: { monster: monster({ display_name: 'Thing', homebrew_monster_id: 'h2' }) } })
    expect(alias(bare).exists()).toBe(false)
  })

  it('has no alias without a display name, and names a homebrew monster from its stat block', () => {
    const w = mount(MonsterStatsPanel, { props: { monster: monster({ monster_name: 'Goblin' }) } })
    expect(title(w)).toBe('Goblin')
    expect(alias(w).exists()).toBe(false)
    const hb = mount(MonsterStatsPanel, { props: { monster: monster({ homebrew_monster_id: 'h1', monster_data: { name: 'Mutant' } }) } })
    expect(title(hb)).toBe('Mutant')
  })
})
