/**
 * Settings sections (MIMIR-T-0692): Manage Campaigns and Import Books are
 * panels in the right pane, and the URL holds the section.
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { flushPromises } from '@vue/test-utils'
import { defineComponent, h, reactive } from 'vue'
import { mountWithPlugins } from '@tests/helpers/mountHelpers'
import { setupInvokeMock, resetInvokeMock, setInvokeFallback } from '@tests/helpers/mockInvoke'

const route = reactive<{ query: Record<string, string> }>({ query: {} })
const replace = vi.fn((to: { query: Record<string, string> }) => {
  route.query = to.query
})
vi.mock('vue-router', () => ({ useRoute: () => route, useRouter: () => ({ replace }) }))
vi.mock('@tauri-apps/api/app', () => ({ getVersion: async () => '0.0.0' }))

import SettingsView from '@/views/SettingsView.vue'

const stubs = {
  MainLayout: defineComponent({ setup: (_, { slots }) => () => h('div', slots.default?.()) }),
  CampaignManagementPanel: defineComponent({ setup: () => () => h('section', { 'data-testid': 'manage-campaigns-panel' }) }),
  BookManagementPanel: defineComponent({ setup: () => () => h('section', { 'data-testid': 'import-books-panel' }) }),
  ThemeSelector: true,
}

async function mountSettings() {
  const wrapper = mountWithPlugins(SettingsView, { stubs })
  await flushPromises()
  return wrapper
}

const nav = (w: Awaited<ReturnType<typeof mountSettings>>, label: string) =>
  w.findAll('.nav-item').find((b) => b.text() === label)!

describe('SettingsView sections', () => {
  beforeEach(() => {
    setupInvokeMock()
    setInvokeFallback(() => ({ success: false }))
    route.query = {}
    replace.mockClear()
  })
  afterEach(() => resetInvokeMock())

  it('shows Manage Campaigns in the pane, not a modal, and records it in the URL', async () => {
    const wrapper = await mountSettings()
    await nav(wrapper, 'Manage Campaigns').trigger('click')
    await flushPromises()
    expect(wrapper.find('[data-testid="manage-campaigns-panel"]').exists()).toBe(true)
    expect(nav(wrapper, 'Manage Campaigns').classes()).toContain('active')
    expect(replace).toHaveBeenLastCalledWith({ query: { section: 'manage-campaigns' } })
    expect(document.querySelector('[role="dialog"]')).toBeNull()
  })

  it('opens the section named in the URL, and follows the URL', async () => {
    route.query = { section: 'import-books' }
    const wrapper = await mountSettings()
    expect(wrapper.find('[data-testid="import-books-panel"]').exists()).toBe(true)
    expect(nav(wrapper, 'Import Books').classes()).toContain('active')
    route.query = { section: 'theme' }
    await flushPromises()
    expect(wrapper.find('[data-testid="import-books-panel"]').exists()).toBe(false)
    expect(nav(wrapper, 'Theme').classes()).toContain('active')
  })

  it('falls back to Theme for an unknown section', async () => {
    route.query = { section: 'nope' }
    const wrapper = await mountSettings()
    expect(nav(wrapper, 'Theme').classes()).toContain('active')
  })
})
