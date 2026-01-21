import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import LoadingSpinner from '../LoadingSpinner.vue'

describe('LoadingSpinner', () => {
  describe('rendering', () => {
    it('renders the spinner element', () => {
      const wrapper = mount(LoadingSpinner)

      expect(wrapper.find('.loading-spinner').exists()).toBe(true)
    })

    it('renders within a loading container', () => {
      const wrapper = mount(LoadingSpinner)

      expect(wrapper.find('.loading-container').exists()).toBe(true)
    })
  })

  describe('message prop', () => {
    it('does not render message when not provided', () => {
      const wrapper = mount(LoadingSpinner)

      expect(wrapper.find('.loading-message').exists()).toBe(false)
    })

    it('renders message when provided', () => {
      const wrapper = mount(LoadingSpinner, {
        props: {
          message: 'Loading data...'
        }
      })

      expect(wrapper.find('.loading-message').exists()).toBe(true)
      expect(wrapper.find('.loading-message').text()).toBe('Loading data...')
    })

    it('renders different messages correctly', () => {
      const messages = [
        'Please wait...',
        'Fetching campaigns...',
        'Saving changes...',
        'Processing...'
      ]

      messages.forEach(message => {
        const wrapper = mount(LoadingSpinner, {
          props: { message }
        })

        expect(wrapper.find('.loading-message').text()).toBe(message)
      })
    })

    it('handles empty string message by not rendering', () => {
      const wrapper = mount(LoadingSpinner, {
        props: {
          message: ''
        }
      })

      // Empty string is falsy, so message should not render
      expect(wrapper.find('.loading-message').exists()).toBe(false)
    })
  })

  describe('structure', () => {
    it('spinner appears before message in DOM order', () => {
      const wrapper = mount(LoadingSpinner, {
        props: {
          message: 'Loading...'
        }
      })

      const container = wrapper.find('.loading-container')
      const children = container.element.children

      expect(children[0].classList.contains('loading-spinner')).toBe(true)
      expect(children[1].classList.contains('loading-message')).toBe(true)
    })
  })
})
