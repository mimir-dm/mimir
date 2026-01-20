import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useThemeStore } from '../theme'
import { invoke } from '@tauri-apps/api/core'
import { emit, listen } from '@tauri-apps/api/event'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

vi.mock('@tauri-apps/api/event', () => ({
  emit: vi.fn(),
  listen: vi.fn()
}))

const mockInvoke = vi.mocked(invoke)
const mockEmit = vi.mocked(emit)
const mockListen = vi.mocked(listen)

describe('useThemeStore', () => {
  let localStorageMock: Record<string, string>

  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()

    // Mock localStorage
    localStorageMock = {}
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation((key) => localStorageMock[key] || null)
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation((key, value) => {
      localStorageMock[key] = value
    })

    // Mock document.body.classList
    document.body.classList.remove('theme-light', 'theme-dark', 'theme-hyper')
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('initial state', () => {
    it('has empty themes array', () => {
      const store = useThemeStore()
      expect(store.themes).toEqual([])
    })

    it('has light as default theme', () => {
      const store = useThemeStore()
      expect(store.currentTheme).toBe('light')
    })
  })

  describe('loadThemes', () => {
    it('loads themes from backend', async () => {
      const mockThemes = [
        { id: 'light', name: 'Light', description: 'Light theme' },
        { id: 'dark', name: 'Dark', description: 'Dark theme' }
      ]
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockThemes })

      const store = useThemeStore()
      await store.loadThemes()

      expect(mockInvoke).toHaveBeenCalledWith('get_themes')
      expect(store.themes).toEqual(mockThemes)
    })

    it('handles error silently', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Backend error'))

      const store = useThemeStore()
      await store.loadThemes()

      // Should not throw, themes should remain empty
      expect(store.themes).toEqual([])
    })

    it('handles unsuccessful response', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false, error: 'Failed' })

      const store = useThemeStore()
      await store.loadThemes()

      expect(store.themes).toEqual([])
    })
  })

  describe('applyTheme', () => {
    it('applies theme from localStorage', () => {
      localStorageMock['theme'] = 'dark'

      const store = useThemeStore()
      store.applyTheme()

      expect(store.currentTheme).toBe('dark')
      expect(document.body.classList.contains('theme-dark')).toBe(true)
    })

    it('defaults to light when no saved theme', () => {
      const store = useThemeStore()
      store.applyTheme()

      expect(store.currentTheme).toBe('light')
      expect(document.body.classList.contains('theme-light')).toBe(true)
    })

    it('removes existing theme classes', () => {
      document.body.classList.add('theme-dark')
      localStorageMock['theme'] = 'hyper'

      const store = useThemeStore()
      store.applyTheme()

      expect(document.body.classList.contains('theme-dark')).toBe(false)
      expect(document.body.classList.contains('theme-hyper')).toBe(true)
    })
  })

  describe('setTheme', () => {
    it('updates current theme', async () => {
      const store = useThemeStore()
      await store.setTheme('dark')

      expect(store.currentTheme).toBe('dark')
    })

    it('saves theme to localStorage', async () => {
      const store = useThemeStore()
      await store.setTheme('hyper')

      expect(localStorage.setItem).toHaveBeenCalledWith('theme', 'hyper')
    })

    it('applies theme class to body', async () => {
      const store = useThemeStore()
      await store.setTheme('dark')

      expect(document.body.classList.contains('theme-dark')).toBe(true)
    })

    it('removes previous theme class', async () => {
      document.body.classList.add('theme-light')

      const store = useThemeStore()
      await store.setTheme('dark')

      expect(document.body.classList.contains('theme-light')).toBe(false)
    })

    it('broadcasts theme change by default', async () => {
      const store = useThemeStore()
      await store.setTheme('dark')

      expect(mockEmit).toHaveBeenCalledWith('theme-changed', { theme: 'dark' })
    })

    it('does not broadcast when broadcast is false', async () => {
      const store = useThemeStore()
      await store.setTheme('dark', false)

      expect(mockEmit).not.toHaveBeenCalled()
    })

    it('handles emit error silently', async () => {
      mockEmit.mockRejectedValueOnce(new Error('Emit failed'))

      const store = useThemeStore()
      // Should not throw
      await store.setTheme('dark')

      expect(store.currentTheme).toBe('dark')
    })
  })

  describe('initThemeSync', () => {
    it('sets up listener for theme-changed event', async () => {
      const mockUnlisten = vi.fn()
      mockListen.mockResolvedValueOnce(mockUnlisten)

      const store = useThemeStore()
      await store.initThemeSync()

      expect(mockListen).toHaveBeenCalledWith('theme-changed', expect.any(Function))
    })

    it('updates theme when event is received', async () => {
      let eventCallback: ((event: { payload: { theme: string } }) => void) | null = null
      mockListen.mockImplementation(async (_event, callback) => {
        eventCallback = callback as typeof eventCallback
        return vi.fn()
      })

      const store = useThemeStore()
      await store.initThemeSync()

      // Simulate receiving theme change event
      if (eventCallback) {
        eventCallback({ payload: { theme: 'hyper' } })
      }

      expect(store.currentTheme).toBe('hyper')
    })
  })

  describe('cleanup', () => {
    it('calls unlisten function when cleaning up', async () => {
      const mockUnlisten = vi.fn()
      mockListen.mockResolvedValueOnce(mockUnlisten)

      const store = useThemeStore()
      await store.initThemeSync()
      store.cleanup()

      expect(mockUnlisten).toHaveBeenCalled()
    })

    it('handles cleanup when no listener was set', () => {
      const store = useThemeStore()
      // Should not throw
      store.cleanup()
    })
  })

  describe('theme values', () => {
    it('supports light theme', async () => {
      const store = useThemeStore()
      await store.setTheme('light')

      expect(store.currentTheme).toBe('light')
      expect(document.body.classList.contains('theme-light')).toBe(true)
    })

    it('supports dark theme', async () => {
      const store = useThemeStore()
      await store.setTheme('dark')

      expect(store.currentTheme).toBe('dark')
      expect(document.body.classList.contains('theme-dark')).toBe(true)
    })

    it('supports hyper theme', async () => {
      const store = useThemeStore()
      await store.setTheme('hyper')

      expect(store.currentTheme).toBe('hyper')
      expect(document.body.classList.contains('theme-hyper')).toBe(true)
    })
  })
})
