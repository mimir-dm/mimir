import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import ThemeSelector from '../ThemeSelector.vue'

// Mock the theme store
const mockSetTheme = vi.fn()
const mockThemeStore = {
  currentTheme: 'light',
  themes: [],
  setTheme: mockSetTheme
}

vi.mock('../../../../stores/theme', () => ({
  useThemeStore: () => mockThemeStore
}))

describe('ThemeSelector', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    mockThemeStore.currentTheme = 'light'
    mockThemeStore.themes = []
  })

  describe('rendering', () => {
    it('renders theme label', () => {
      const wrapper = mount(ThemeSelector)

      expect(wrapper.find('label').text()).toBe('Theme')
    })

    it('renders select element', () => {
      const wrapper = mount(ThemeSelector)

      expect(wrapper.find('select').exists()).toBe(true)
    })

    it('renders default themes when store has no themes', () => {
      const wrapper = mount(ThemeSelector)

      const options = wrapper.findAll('option')
      expect(options).toHaveLength(3)
      expect(options[0].text()).toBe('Light')
      expect(options[1].text()).toBe('Dark')
      expect(options[2].text()).toBe('Hyper')
    })

    it('renders themes from store when available', () => {
      mockThemeStore.themes = [
        { id: 'custom1', name: 'Custom Theme 1', description: 'Custom' },
        { id: 'custom2', name: 'Custom Theme 2', description: 'Custom' }
      ]

      const wrapper = mount(ThemeSelector)

      const options = wrapper.findAll('option')
      expect(options).toHaveLength(2)
      expect(options[0].text()).toBe('Custom Theme 1')
      expect(options[1].text()).toBe('Custom Theme 2')
    })
  })

  describe('theme selection', () => {
    it('displays current theme as selected', () => {
      mockThemeStore.currentTheme = 'dark'

      const wrapper = mount(ThemeSelector)

      const select = wrapper.find('select').element as HTMLSelectElement
      expect(select.value).toBe('dark')
    })

    it('calls setTheme when selection changes', async () => {
      const wrapper = mount(ThemeSelector)

      const select = wrapper.find('select')
      await select.setValue('dark')
      await select.trigger('change')

      expect(mockSetTheme).toHaveBeenCalledWith('dark')
    })

    it('updates selection when different theme is chosen', async () => {
      const wrapper = mount(ThemeSelector)

      const select = wrapper.find('select')
      await select.setValue('hyper')

      expect((select.element as HTMLSelectElement).value).toBe('hyper')
    })
  })

  describe('accessibility', () => {
    it('has label associated with select via id', () => {
      const wrapper = mount(ThemeSelector)

      const label = wrapper.find('label')
      const select = wrapper.find('select')

      expect(label.attributes('for')).toBe('theme-select')
      expect(select.attributes('id')).toBe('theme-select')
    })

    it('each option has a unique value', () => {
      const wrapper = mount(ThemeSelector)

      const options = wrapper.findAll('option')
      const values = options.map(opt => opt.attributes('value'))

      expect(new Set(values).size).toBe(values.length)
    })
  })

  describe('default themes', () => {
    it('includes light theme option', () => {
      const wrapper = mount(ThemeSelector)

      const lightOption = wrapper.find('option[value="light"]')
      expect(lightOption.exists()).toBe(true)
      expect(lightOption.text()).toBe('Light')
    })

    it('includes dark theme option', () => {
      const wrapper = mount(ThemeSelector)

      const darkOption = wrapper.find('option[value="dark"]')
      expect(darkOption.exists()).toBe(true)
      expect(darkOption.text()).toBe('Dark')
    })

    it('includes hyper theme option', () => {
      const wrapper = mount(ThemeSelector)

      const hyperOption = wrapper.find('option[value="hyper"]')
      expect(hyperOption.exists()).toBe(true)
      expect(hyperOption.text()).toBe('Hyper')
    })
  })
})
