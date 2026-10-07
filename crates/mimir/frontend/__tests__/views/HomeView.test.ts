/**
 * Home actions (MIMIR-T-0688): Continue the last campaign, create, import.
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { flushPromises } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import { mountWithPlugins } from '@tests/helpers/mountHelpers'
import { setupInvokeMock, resetInvokeMock, mockCommand } from '@tests/helpers/mockInvoke'

const push = vi.fn()
vi.mock('vue-router', () => ({ useRouter: () => ({ push }) }))

import HomeView from '@/views/HomeView.vue'

const campaign = (id: string, name: string, archived = false) => ({
  id,
  name,
  description: null,
  archived_at: archived ? '2026-01-01T00:00:00Z' : null,
  created_at: '',
  updated_at: '',
})

// The layout and the dialog are not under test; the router-link stub keeps `to`.
const stubs = {
  MainLayout: defineComponent({ setup: (_, { slots }) => () => h('div', slots.default?.()) }),
  CampaignArchiveImportDialog: true,
  RouterLink: defineComponent({
    props: { to: { type: String, required: true } },
    setup: (props, { slots }) => () => h('a', { to: props.to }, slots.default?.()),
  }),
}

async function mountHome() {
  const wrapper = mountWithPlugins(HomeView, { stubs })
  await flushPromises()
  return wrapper
}

describe('HomeView actions', () => {
  beforeEach(() => {
    setupInvokeMock()
    localStorage.clear()
    push.mockClear()
  })
  afterEach(() => resetInvokeMock())

  it('offers to continue the last campaign; create and import are secondary', async () => {
    mockCommand('list_campaigns', [campaign('c1', 'Lost Mine'), campaign('c2', 'Curse')])
    localStorage.setItem('selectedCampaignId', 'c2')
    const wrapper = await mountHome()
    const cont = wrapper.get('[data-testid="home-continue"]')
    expect(cont.text()).toContain('Continue Curse')
    expect(cont.attributes('to')).toBe('/campaigns/c2/dashboard')
    expect(wrapper.get('[data-testid="home-create"]').classes()).toContain('btn-secondary')
    expect(wrapper.get('[data-testid="home-create"]').attributes('to')).toBe('/campaigns/new')
    expect(wrapper.find('[data-testid="home-import"]').exists()).toBe(true)
  })

  it('has no Continue when the stored campaign is gone or archived; Create is primary', async () => {
    mockCommand('list_campaigns', [campaign('c1', 'Lost Mine', true)])
    localStorage.setItem('selectedCampaignId', 'c1')
    const wrapper = await mountHome()
    expect(wrapper.find('[data-testid="home-continue"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="home-create"]').classes()).toContain('btn-primary')
  })

  it('opens an imported campaign and remembers it', async () => {
    mockCommand('list_campaigns', [])
    const wrapper = await mountHome()
    await wrapper.get('[data-testid="home-import"]').trigger('click')
    const dialog = wrapper.findComponent({ name: 'CampaignArchiveImportDialog' })
    expect(dialog.props('visible')).toBe(true)
    dialog.vm.$emit('imported', campaign('c9', 'Imported'))
    await flushPromises()
    expect(localStorage.getItem('selectedCampaignId')).toBe('c9')
    expect(push).toHaveBeenCalledWith('/campaigns/c9/dashboard')
  })
})
