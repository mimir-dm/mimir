/**
 * ModuleInspector: one details panel for monsters, traps and POIs (MIMIR-T-0693).
 */
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import ModuleInspector, { type InspectorSelection } from '@/features/modules/components/ModuleInspector.vue'

const body = (name: string, prop: string) =>
  defineComponent({
    name,
    props: [prop],
    setup: (props: Record<string, { name?: string; monster_name?: string }>) => () =>
      h('div', { 'data-testid': name }, props[prop]?.name ?? props[prop]?.monster_name),
  })

const stubs = {
  MonsterStatsPanel: body('MonsterStatsPanel', 'monster'),
  TrapDetailsPanel: body('TrapDetailsPanel', 'trap'),
  PoiDetailsPanel: body('PoiDetailsPanel', 'poi'),
}

const monster = { kind: 'monster', data: { id: 'm1', monster_name: 'Goblin' } } as unknown as InspectorSelection
const trap = { kind: 'trap', data: { id: 't1', name: 'Pit', source: 'DMG', count: 1 } } as InspectorSelection
const poi = {
  kind: 'poi',
  data: { id: 'p1', name: 'Altar', description: null, icon: 'pin', color: null, visible: 1, grid_x: 0, grid_y: 0, count: 1 },
} as InspectorSelection

function mountInspector(selection: InspectorSelection | null) {
  return mount(ModuleInspector, { props: { selection }, global: { stubs } })
}

describe('ModuleInspector', () => {
  it('renders nothing without a selection', () => {
    expect(mountInspector(null).find('[data-testid="module-inspector"]').exists()).toBe(false)
  })

  it('shows the body of the selected kind and replaces it on a new selection', async () => {
    const wrapper = mountInspector(monster)
    expect(wrapper.get('[data-testid="MonsterStatsPanel"]').text()).toBe('Goblin')
    expect(wrapper.get('[data-testid="module-inspector"]').attributes('aria-label')).toBe('Monster details')
    await wrapper.setProps({ selection: trap })
    expect(wrapper.find('[data-testid="MonsterStatsPanel"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="TrapDetailsPanel"]').text()).toBe('Pit')
    await wrapper.setProps({ selection: poi })
    expect(wrapper.get('[data-testid="PoiDetailsPanel"]').text()).toBe('Altar')
    // One frame, one close button.
    expect(wrapper.findAll('[data-testid="inspector-close"]')).toHaveLength(1)
  })

  it('closes from the button and from Esc inside it', async () => {
    const wrapper = mountInspector(trap)
    await wrapper.get('[data-testid="inspector-close"]').trigger('click')
    await wrapper.get('[data-testid="module-inspector"]').trigger('keydown', { key: 'Escape' })
    expect(wrapper.emitted('close')).toHaveLength(2)
  })

  it('collapses, and opens again for a new selection', async () => {
    const wrapper = mountInspector(monster)
    await wrapper.get('[data-testid="inspector-toggle"]').trigger('click')
    expect(wrapper.get('[data-testid="module-inspector"]').classes()).toContain('collapsed')
    expect(wrapper.find('[data-testid="MonsterStatsPanel"]').exists()).toBe(false)
    await wrapper.setProps({ selection: poi })
    expect(wrapper.get('[data-testid="module-inspector"]').classes()).not.toContain('collapsed')
  })
})
