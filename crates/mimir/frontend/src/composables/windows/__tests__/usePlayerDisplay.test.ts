/**
 * Tests for usePlayerDisplay composable.
 *
 * Tests window open/close/toggle, blackout mode, sending maps,
 * fullscreen toggle, and error handling. All invoke calls are mocked.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const mockInvoke = vi.mocked(invoke)

// Must re-import usePlayerDisplay fresh per test since it uses module-level state.
async function getUsePlayerDisplay() {
  const mod = await import('../usePlayerDisplay')
  return mod.usePlayerDisplay()
}

describe('usePlayerDisplay', () => {
  beforeEach(() => {
    vi.resetModules()
    vi.clearAllMocks()
    // Re-mock after resetModules
    vi.mock('@tauri-apps/api/core', () => ({
      invoke: vi.fn(),
    }))
  })

  describe('initial state', () => {
    it('starts with display closed', async () => {
      const pd = await getUsePlayerDisplay()
      expect(pd.isDisplayOpen.value).toBe(false)
    })

    it('starts with no current map', async () => {
      const pd = await getUsePlayerDisplay()
      expect(pd.currentMapId.value).toBeNull()
    })

    it('starts with blackout off', async () => {
      const pd = await getUsePlayerDisplay()
      expect(pd.isBlackout.value).toBe(false)
    })
  })

  describe('checkDisplayOpen', () => {
    it('updates state based on invoke result', async () => {
      const { invoke: freshInvoke } = await import('@tauri-apps/api/core')
      vi.mocked(freshInvoke).mockResolvedValueOnce(true)

      const pd = await getUsePlayerDisplay()
      const result = await pd.checkDisplayOpen()

      expect(result).toBe(true)
      expect(pd.isDisplayOpen.value).toBe(true)
    })

    it('returns false on error', async () => {
      const { invoke: freshInvoke } = await import('@tauri-apps/api/core')
      vi.mocked(freshInvoke).mockRejectedValueOnce(new Error('window error'))

      const pd = await getUsePlayerDisplay()
      const result = await pd.checkDisplayOpen()

      expect(result).toBe(false)
    })
  })

  describe('openDisplay', () => {
    it('calls invoke and sets state to open', async () => {
      const { invoke: freshInvoke } = await import('@tauri-apps/api/core')
      vi.mocked(freshInvoke).mockResolvedValueOnce(undefined)

      const pd = await getUsePlayerDisplay()
      await pd.openDisplay()

      expect(pd.isDisplayOpen.value).toBe(true)
      expect(freshInvoke).toHaveBeenCalledWith('open_player_display_window')
    })

    it('throws on error', async () => {
      const { invoke: freshInvoke } = await import('@tauri-apps/api/core')
      vi.mocked(freshInvoke).mockRejectedValueOnce(new Error('failed'))

      const pd = await getUsePlayerDisplay()
      await expect(pd.openDisplay()).rejects.toThrow('failed')
    })
  })

  describe('closeDisplay', () => {
    it('calls invoke and resets state', async () => {
      const { invoke: freshInvoke } = await import('@tauri-apps/api/core')
      vi.mocked(freshInvoke)
        .mockResolvedValueOnce(undefined)  // open
        .mockResolvedValueOnce(undefined)  // close

      const pd = await getUsePlayerDisplay()
      await pd.openDisplay()
      expect(pd.isDisplayOpen.value).toBe(true)

      await pd.closeDisplay()
      expect(pd.isDisplayOpen.value).toBe(false)
      expect(pd.currentMapId.value).toBeNull()
    })
  })

  describe('toggleDisplay', () => {
    it('opens when closed', async () => {
      const { invoke: freshInvoke } = await import('@tauri-apps/api/core')
      vi.mocked(freshInvoke).mockResolvedValueOnce(undefined)

      const pd = await getUsePlayerDisplay()
      const result = await pd.toggleDisplay()

      expect(result).toBe(true)
      expect(pd.isDisplayOpen.value).toBe(true)
    })

    it('closes when open', async () => {
      const { invoke: freshInvoke } = await import('@tauri-apps/api/core')
      vi.mocked(freshInvoke)
        .mockResolvedValueOnce(undefined)  // open
        .mockResolvedValueOnce(undefined)  // close

      const pd = await getUsePlayerDisplay()
      await pd.openDisplay()

      const result = await pd.toggleDisplay()
      expect(result).toBe(false)
      expect(pd.isDisplayOpen.value).toBe(false)
    })
  })

  describe('sendMapToDisplay', () => {
    it('calls invoke with correct params', async () => {
      const { invoke: freshInvoke } = await import('@tauri-apps/api/core')
      vi.mocked(freshInvoke).mockResolvedValueOnce(undefined)

      const pd = await getUsePlayerDisplay()
      await pd.sendMapToDisplay(42, 'square', 70, 10, 20, 'bright', 1920, 1080)

      expect(freshInvoke).toHaveBeenCalledWith('send_map_to_display', {
        mapId: 42,
        gridType: 'square',
        gridSizePx: 70,
        gridOffsetX: 10,
        gridOffsetY: 20,
        ambientLight: 'bright',
        mapWidth: 1920,
        mapHeight: 1080,
      })
    })

    it('uses default params when not provided', async () => {
      const { invoke: freshInvoke } = await import('@tauri-apps/api/core')
      vi.mocked(freshInvoke).mockResolvedValueOnce(undefined)

      const pd = await getUsePlayerDisplay()
      await pd.sendMapToDisplay(42)

      expect(freshInvoke).toHaveBeenCalledWith('send_map_to_display', {
        mapId: 42,
        gridType: 'none',
        gridSizePx: null,
        gridOffsetX: 0,
        gridOffsetY: 0,
        ambientLight: null,
        mapWidth: null,
        mapHeight: null,
      })
    })

    it('updates currentMapId', async () => {
      const { invoke: freshInvoke } = await import('@tauri-apps/api/core')
      vi.mocked(freshInvoke).mockResolvedValueOnce(undefined)

      const pd = await getUsePlayerDisplay()
      await pd.sendMapToDisplay(42)

      expect(pd.currentMapId.value).toBe(42)
    })
  })

  describe('blackout', () => {
    it('toggleBlackout toggles state', async () => {
      const { invoke: freshInvoke } = await import('@tauri-apps/api/core')
      vi.mocked(freshInvoke).mockResolvedValueOnce(undefined)

      const pd = await getUsePlayerDisplay()
      expect(pd.isBlackout.value).toBe(false)

      await pd.toggleBlackout()
      expect(pd.isBlackout.value).toBe(true)
      expect(freshInvoke).toHaveBeenCalledWith('toggle_display_blackout', { isBlackout: true })
    })

    it('toggleBlackout can toggle back off', async () => {
      const { invoke: freshInvoke } = await import('@tauri-apps/api/core')
      vi.mocked(freshInvoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce(undefined)

      const pd = await getUsePlayerDisplay()
      await pd.toggleBlackout() // off -> on
      await pd.toggleBlackout() // on -> off
      expect(pd.isBlackout.value).toBe(false)
    })

    it('setBlackout sets explicit state', async () => {
      const { invoke: freshInvoke } = await import('@tauri-apps/api/core')
      vi.mocked(freshInvoke).mockResolvedValueOnce(undefined)

      const pd = await getUsePlayerDisplay()
      await pd.setBlackout(true)

      expect(pd.isBlackout.value).toBe(true)
      expect(freshInvoke).toHaveBeenCalledWith('toggle_display_blackout', { isBlackout: true })
    })
  })

  describe('fullscreen', () => {
    it('calls invoke for fullscreen toggle', async () => {
      const { invoke: freshInvoke } = await import('@tauri-apps/api/core')
      vi.mocked(freshInvoke).mockResolvedValueOnce(true)

      const pd = await getUsePlayerDisplay()
      const result = await pd.toggleFullscreen()

      expect(result).toBe(true)
      expect(freshInvoke).toHaveBeenCalledWith('toggle_player_display_fullscreen')
    })

    it('throws on error', async () => {
      const { invoke: freshInvoke } = await import('@tauri-apps/api/core')
      vi.mocked(freshInvoke).mockRejectedValueOnce(new Error('not supported'))

      const pd = await getUsePlayerDisplay()
      await expect(pd.toggleFullscreen()).rejects.toThrow('not supported')
    })
  })
})
