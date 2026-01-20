import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import MultiSelectFilter from '../MultiSelectFilter.vue'

describe('MultiSelectFilter', () => {
  const defaultProps = {
    label: 'Categories',
    options: ['Fantasy', 'Sci-Fi', 'Horror', 'Mystery'],
    modelValue: []
  }

  beforeEach(() => {
    // Reset any document event listeners between tests
    vi.clearAllMocks()
  })

  afterEach(() => {
    // Clean up any remaining event listeners
    document.body.innerHTML = ''
  })

  describe('rendering', () => {
    it('renders button with label', () => {
      const wrapper = mount(MultiSelectFilter, { props: defaultProps })

      expect(wrapper.find('.btn-filter').text()).toContain('Categories')
    })

    it('shows active count when items are selected', () => {
      const wrapper = mount(MultiSelectFilter, {
        props: {
          ...defaultProps,
          modelValue: ['Fantasy', 'Horror']
        }
      })

      expect(wrapper.find('.btn-filter__count').text()).toBe('2')
    })

    it('does not show count badge when no items selected', () => {
      const wrapper = mount(MultiSelectFilter, { props: defaultProps })

      expect(wrapper.find('.btn-filter__count').exists()).toBe(false)
    })

    it('applies active class when items are selected', () => {
      const wrapper = mount(MultiSelectFilter, {
        props: {
          ...defaultProps,
          modelValue: ['Fantasy']
        }
      })

      expect(wrapper.find('.btn-filter').classes()).toContain('btn-filter--active')
    })
  })

  describe('dropdown behavior', () => {
    it('opens dropdown on button click', async () => {
      const wrapper = mount(MultiSelectFilter, { props: defaultProps })

      expect(wrapper.find('.form-select-custom__dropdown').exists()).toBe(false)

      await wrapper.find('.btn-filter').trigger('click')

      expect(wrapper.find('.form-select-custom__dropdown').exists()).toBe(true)
    })

    it('closes dropdown on second button click', async () => {
      const wrapper = mount(MultiSelectFilter, { props: defaultProps })

      await wrapper.find('.btn-filter').trigger('click')
      expect(wrapper.find('.form-select-custom__dropdown').exists()).toBe(true)

      await wrapper.find('.btn-filter').trigger('click')
      expect(wrapper.find('.form-select-custom__dropdown').exists()).toBe(false)
    })

    it('rotates chevron when dropdown is open', async () => {
      const wrapper = mount(MultiSelectFilter, { props: defaultProps })

      expect(wrapper.find('.form-select-custom__chevron').classes()).not.toContain('rotated')

      await wrapper.find('.btn-filter').trigger('click')

      expect(wrapper.find('.form-select-custom__chevron').classes()).toContain('rotated')
    })
  })

  describe('options', () => {
    it('renders all options when dropdown is open', async () => {
      const wrapper = mount(MultiSelectFilter, { props: defaultProps })

      await wrapper.find('.btn-filter').trigger('click')

      const options = wrapper.findAll('.form-checkbox')
      expect(options).toHaveLength(4)
    })

    it('shows search input when more than 10 options', async () => {
      const manyOptions = Array.from({ length: 15 }, (_, i) => `Option ${i + 1}`)
      const wrapper = mount(MultiSelectFilter, {
        props: {
          ...defaultProps,
          options: manyOptions
        }
      })

      await wrapper.find('.btn-filter').trigger('click')

      expect(wrapper.find('.form-select-custom__search input').exists()).toBe(true)
    })

    it('does not show search input for 10 or fewer options', async () => {
      const wrapper = mount(MultiSelectFilter, { props: defaultProps })

      await wrapper.find('.btn-filter').trigger('click')

      expect(wrapper.find('.form-select-custom__search').exists()).toBe(false)
    })
  })

  describe('selection', () => {
    it('emits update:modelValue when option is toggled on', async () => {
      const wrapper = mount(MultiSelectFilter, { props: defaultProps })

      await wrapper.find('.btn-filter').trigger('click')

      const firstCheckbox = wrapper.find('.form-checkbox__input')
      await firstCheckbox.setValue(true)

      expect(wrapper.emitted('update:modelValue')).toBeTruthy()
      expect(wrapper.emitted('update:modelValue')![0]).toEqual([['Fantasy']])
    })

    it('emits update:modelValue with item removed when toggled off', async () => {
      const wrapper = mount(MultiSelectFilter, {
        props: {
          ...defaultProps,
          modelValue: ['Fantasy', 'Horror']
        }
      })

      await wrapper.find('.btn-filter').trigger('click')

      // Find and toggle the first checkbox (Fantasy)
      const firstCheckbox = wrapper.find('.form-checkbox__input')
      await firstCheckbox.setValue(false)

      expect(wrapper.emitted('update:modelValue')).toBeTruthy()
      expect(wrapper.emitted('update:modelValue')![0]).toEqual([['Horror']])
    })

    it('shows checkbox as checked for selected items', async () => {
      const wrapper = mount(MultiSelectFilter, {
        props: {
          ...defaultProps,
          modelValue: ['Fantasy']
        }
      })

      await wrapper.find('.btn-filter').trigger('click')

      const checkboxes = wrapper.findAll('.form-checkbox__input')
      const fantasyCheckbox = checkboxes[0].element as HTMLInputElement

      expect(fantasyCheckbox.checked).toBe(true)
    })
  })

  describe('clear all / select all', () => {
    it('clears all selections when Clear All is clicked', async () => {
      const wrapper = mount(MultiSelectFilter, {
        props: {
          ...defaultProps,
          modelValue: ['Fantasy', 'Horror']
        }
      })

      await wrapper.find('.btn-filter').trigger('click')

      const clearButton = wrapper.findAll('.form-select-custom__action-btn')[0]
      await clearButton.trigger('click')

      expect(wrapper.emitted('update:modelValue')).toBeTruthy()
      expect(wrapper.emitted('update:modelValue')![0]).toEqual([[]])
    })

    it('selects all options when Select All is clicked', async () => {
      const wrapper = mount(MultiSelectFilter, { props: defaultProps })

      await wrapper.find('.btn-filter').trigger('click')

      const selectAllButton = wrapper.findAll('.form-select-custom__action-btn')[1]
      await selectAllButton.trigger('click')

      expect(wrapper.emitted('update:modelValue')).toBeTruthy()
      expect(wrapper.emitted('update:modelValue')![0]).toEqual([defaultProps.options])
    })
  })

  describe('search filtering', () => {
    it('filters options based on search term', async () => {
      const manyOptions = ['Apple', 'Banana', 'Cherry', 'Apricot', 'Blueberry',
                          'Cranberry', 'Date', 'Elderberry', 'Fig', 'Grape', 'Honeydew']
      const wrapper = mount(MultiSelectFilter, {
        props: {
          ...defaultProps,
          options: manyOptions
        }
      })

      await wrapper.find('.btn-filter').trigger('click')

      const searchInput = wrapper.find('.form-select-custom__search input')
      await searchInput.setValue('Ap')

      const visibleOptions = wrapper.findAll('.form-checkbox')
      // Apple, Apricot, and Grape (contains 'ap')
      expect(visibleOptions).toHaveLength(3)
    })

    it('search is case insensitive', async () => {
      const manyOptions = Array.from({ length: 15 }, (_, i) => `Option ${i + 1}`)
      const wrapper = mount(MultiSelectFilter, {
        props: {
          ...defaultProps,
          options: manyOptions
        }
      })

      await wrapper.find('.btn-filter').trigger('click')

      const searchInput = wrapper.find('.form-select-custom__search input')
      await searchInput.setValue('OPTION 1')

      const visibleOptions = wrapper.findAll('.form-checkbox')
      // Should match Option 1, Option 10, Option 11, etc.
      expect(visibleOptions.length).toBeGreaterThan(0)
    })
  })
})
