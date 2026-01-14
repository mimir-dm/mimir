import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import EmptyState from '../EmptyState.vue'

describe('EmptyState', () => {
  describe('rendering', () => {
    it('renders title correctly', () => {
      const wrapper = mount(EmptyState, {
        props: {
          title: 'No items found'
        }
      })

      expect(wrapper.find('.empty-state-title').text()).toBe('No items found')
    })

    it('renders description when provided', () => {
      const wrapper = mount(EmptyState, {
        props: {
          title: 'No items',
          description: 'Try adding some items'
        }
      })

      expect(wrapper.find('.empty-state-description').text()).toBe('Try adding some items')
    })

    it('does not render description when not provided', () => {
      const wrapper = mount(EmptyState, {
        props: {
          title: 'No items'
        }
      })

      expect(wrapper.find('.empty-state-description').exists()).toBe(false)
    })
  })

  describe('variants', () => {
    const variants = ['users', 'characters', 'campaigns', 'books', 'search', 'generic'] as const

    it.each(variants)('renders %s variant with appropriate icon', (variant) => {
      const wrapper = mount(EmptyState, {
        props: {
          title: 'Test',
          variant
        }
      })

      expect(wrapper.find('.empty-state-icon svg').exists()).toBe(true)
    })

    it('renders default/generic icon when no variant specified', () => {
      const wrapper = mount(EmptyState, {
        props: {
          title: 'Test'
        }
      })

      expect(wrapper.find('.empty-state-icon svg').exists()).toBe(true)
    })
  })

  describe('action slot', () => {
    it('renders action slot content when provided', () => {
      const wrapper = mount(EmptyState, {
        props: {
          title: 'No items'
        },
        slots: {
          action: '<button class="test-action">Add Item</button>'
        }
      })

      expect(wrapper.find('.empty-state-action').exists()).toBe(true)
      expect(wrapper.find('.test-action').text()).toBe('Add Item')
    })

    it('does not render action container when slot is empty', () => {
      const wrapper = mount(EmptyState, {
        props: {
          title: 'No items'
        }
      })

      expect(wrapper.find('.empty-state-action').exists()).toBe(false)
    })
  })
})
