/**
 * Tests for CreateModuleModal.vue
 *
 * Tests the module creation form modal including:
 * - Form fields rendering
 * - Validation (name required)
 * - Create button disabled state
 * - Form submission emits correct data
 * - Form reset on close
 */

import { describe, it, expect, vi } from 'vitest'
import { nextTick } from 'vue'
import { mountWithPlugins } from '@tests/helpers/mountHelpers'
import CreateModuleModal from '@/features/campaigns/components/StageLanding/CreateModuleModal.vue'

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('CreateModuleModal', () => {
  function mountModal(show = true) {
    return mountWithPlugins(CreateModuleModal, {
      props: { show },
      stubs: { AppModal: false },
    })
  }

  describe('rendering', () => {
    it('displays Create New Module title', () => {
      const wrapper = mountModal()
      expect(wrapper.text()).toContain('Create New Module')
    })

    it('shows module name input', () => {
      const wrapper = mountModal()
      expect(wrapper.find('#module-name').exists()).toBe(true)
    })

    it('shows module type select', () => {
      const wrapper = mountModal()
      const select = wrapper.find('#module-type')
      expect(select.exists()).toBe(true)
    })

    it('shows module type options', () => {
      const wrapper = mountModal()
      const options = wrapper.findAll('#module-type option')
      expect(options.length).toBeGreaterThanOrEqual(6)
      const optionTexts = options.map(o => o.text())
      expect(optionTexts).toContain('Standard Adventure')
      expect(optionTexts).toContain('Mystery')
      expect(optionTexts).toContain('Dungeon Crawl')
    })

    it('shows description textarea', () => {
      const wrapper = mountModal()
      expect(wrapper.find('#module-description').exists()).toBe(true)
    })

    it('shows Cancel and Create Module buttons', () => {
      const wrapper = mountModal()
      const buttons = wrapper.findAll('button')
      expect(buttons.some(b => b.text() === 'Cancel')).toBe(true)
      expect(buttons.some(b => b.text() === 'Create Module')).toBe(true)
    })
  })

  describe('validation', () => {
    it('disables Create Module button when name is empty', () => {
      const wrapper = mountModal()
      const createBtn = wrapper.findAll('button').find(b => b.text() === 'Create Module')
      expect((createBtn!.element as HTMLButtonElement).disabled).toBe(true)
    })

    it('enables Create Module button when name is filled', async () => {
      const wrapper = mountModal()
      await wrapper.find('#module-name').setValue('My Module')
      await nextTick()
      const createBtn = wrapper.findAll('button').find(b => b.text() === 'Create Module')
      expect((createBtn!.element as HTMLButtonElement).disabled).toBe(false)
    })

    it('trims whitespace-only names as invalid', async () => {
      const wrapper = mountModal()
      await wrapper.find('#module-name').setValue('   ')
      await nextTick()
      const createBtn = wrapper.findAll('button').find(b => b.text() === 'Create Module')
      expect((createBtn!.element as HTMLButtonElement).disabled).toBe(true)
    })
  })

  describe('events', () => {
    it('emits close when Cancel is clicked', async () => {
      const wrapper = mountModal()
      const cancelBtn = wrapper.findAll('button').find(b => b.text() === 'Cancel')
      await cancelBtn!.trigger('click')
      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('emits create with form data when Create Module is clicked', async () => {
      const wrapper = mountModal()
      await wrapper.find('#module-name').setValue('The Goblin Hideout')
      await wrapper.find('#module-type').setValue('dungeon')
      await wrapper.find('#module-description').setValue('A goblin cave')
      await nextTick()

      const createBtn = wrapper.findAll('button').find(b => b.text() === 'Create Module')
      await createBtn!.trigger('click')

      const emitted = wrapper.emitted('create')
      expect(emitted).toBeTruthy()
      expect(emitted![0][0]).toEqual({
        name: 'The Goblin Hideout',
        type: 'dungeon',
        description: 'A goblin cave',
      })
    })

    it('omits description when empty', async () => {
      const wrapper = mountModal()
      await wrapper.find('#module-name').setValue('Quick Module')
      await nextTick()

      const createBtn = wrapper.findAll('button').find(b => b.text() === 'Create Module')
      await createBtn!.trigger('click')

      const emitted = wrapper.emitted('create')
      expect(emitted![0][0]).toEqual({
        name: 'Quick Module',
        type: 'general',
        description: undefined,
      })
    })

    it('does not emit create when name is empty', async () => {
      const wrapper = mountModal()
      const createBtn = wrapper.findAll('button').find(b => b.text() === 'Create Module')
      await createBtn!.trigger('click')
      expect(wrapper.emitted('create')).toBeFalsy()
    })
  })
})
