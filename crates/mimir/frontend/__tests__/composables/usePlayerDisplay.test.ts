/**
 * Tests for usePlayerDisplay composable.
 *
 * Tests window open/close/toggle, blackout mode, sending maps,
 * fullscreen toggle, and error handling. All invoke calls are mocked.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommandRaw,
  mockCommandReject,
  expectCommandCalled,
  expectCommandCalledWith,
} from '@tests/helpers/mockInvoke'

// Must re-import usePlayerDisplay fresh per test since it uses module-level state.
// We use dynamic import + vi.resetModules() to get fresh state.
async function getUsePlayerDisplay() {
  const mod = await import('@/composables/windows/usePlayerDisplay')
  return mod.usePlayerDisplay()
}

describe('usePlayerDisplay', () => {
  beforeEach(() => {
    vi.resetModules()
    setupInvokeMock()
  })
  afterEach(() => {
    resetInvokeMock()
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
      mockCommandRaw('is_player_display_open', true)
      const pd = await getUsePlayerDisplay()
      const result = await pd.checkDisplayOpen()
      expect(result).toBe(true)
      expect(pd.isDisplayOpen.value).toBe(true)
    })

    it('returns false on error', async () => {
      mockCommandReject('is_player_display_open', 'window error')
      const pd = await getUsePlayerDisplay()
      const result = await pd.checkDisplayOpen()
      expect(result).toBe(false)
    })
  })

  describe('openDisplay', () => {
    it('calls invoke and sets state to open', async () => {
      mockCommandRaw('open_player_display_window', undefined)
      const pd = await getUsePlayerDisplay()
      await pd.openDisplay()
      expect(pd.isDisplayOpen.value).toBe(true)
      expectCommandCalled('open_player_display_window')
    })

    it('throws on error', async () => {
      mockCommandReject('open_player_display_window', 'failed')
      const pd = await getUsePlayerDisplay()
      await expect(pd.openDisplay()).rejects.toThrow('failed')
    })
  })

  describe('closeDisplay', () => {
    it('calls invoke and resets state', async () => {
      mockCommandRaw('open_player_display_window', undefined)
      mockCommandRaw('close_player_display_window', undefined)
      const pd = await getUsePlayerDisplay()

      // Open first
      await pd.openDisplay()
      expect(pd.isDisplayOpen.value).toBe(true)

      // Then close
      await pd.closeDisplay()
      expect(pd.isDisplayOpen.value).toBe(false)
      expect(pd.currentMapId.value).toBeNull()
    })
  })

  describe('toggleDisplay', () => {
    it('opens when closed', async () => {
      mockCommandRaw('open_player_display_window', undefined)
      const pd = await getUsePlayerDisplay()
      const result = await pd.toggleDisplay()
      expect(result).toBe(true)
      expect(pd.isDisplayOpen.value).toBe(true)
    })

    it('closes when open', async () => {
      mockCommandRaw('open_player_display_window', undefined)
      mockCommandRaw('close_player_display_window', undefined)
      const pd = await getUsePlayerDisplay()

      await pd.openDisplay()
      const result = await pd.toggleDisplay()
      expect(result).toBe(false)
      expect(pd.isDisplayOpen.value).toBe(false)
    })
  })

  describe('sendMapToDisplay', () => {
    it('calls invoke with correct params', async () => {
      mockCommandRaw('send_map_to_display', undefined)
      const pd = await getUsePlayerDisplay()
      await pd.sendMapToDisplay(42, 'square', 70, 10, 20, 'bright', 1920, 1080)
      expectCommandCalledWith('send_map_to_display', {
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
      mockCommandRaw('send_map_to_display', undefined)
      const pd = await getUsePlayerDisplay()
      await pd.sendMapToDisplay(42)
      expectCommandCalledWith('send_map_to_display', {
        mapId: 42,
        gridType: 'none',
        gridSizePx: null,
        gridOffsetX: 0,
        gridOffsetY: 0,
      })
    })

    it('updates currentMapId', async () => {
      mockCommandRaw('send_map_to_display', undefined)
      const pd = await getUsePlayerDisplay()
      await pd.sendMapToDisplay(42)
      expect(pd.currentMapId.value).toBe(42)
    })
  })

  describe('blackout', () => {
    it('toggleBlackout toggles state', async () => {
      mockCommandRaw('toggle_display_blackout', undefined)
      const pd = await getUsePlayerDisplay()

      expect(pd.isBlackout.value).toBe(false)
      await pd.toggleBlackout()
      expect(pd.isBlackout.value).toBe(true)
      expectCommandCalledWith('toggle_display_blackout', { isBlackout: true })
    })

    it('toggleBlackout can toggle back off', async () => {
      mockCommandRaw('toggle_display_blackout', undefined)
      const pd = await getUsePlayerDisplay()

      await pd.toggleBlackout() // off → on
      await pd.toggleBlackout() // on → off
      expect(pd.isBlackout.value).toBe(false)
    })

    it('setBlackout sets explicit state', async () => {
      mockCommandRaw('toggle_display_blackout', undefined)
      const pd = await getUsePlayerDisplay()

      await pd.setBlackout(true)
      expect(pd.isBlackout.value).toBe(true)
      expectCommandCalledWith('toggle_display_blackout', { isBlackout: true })
    })
  })

  describe('fullscreen', () => {
    it('calls invoke for fullscreen toggle', async () => {
      mockCommandRaw('toggle_player_display_fullscreen', true)
      const pd = await getUsePlayerDisplay()
      const result = await pd.toggleFullscreen()
      expect(result).toBe(true)
      expectCommandCalled('toggle_player_display_fullscreen')
    })

    it('throws on error', async () => {
      mockCommandReject('toggle_player_display_fullscreen', 'not supported')
      const pd = await getUsePlayerDisplay()
      await expect(pd.toggleFullscreen()).rejects.toThrow('not supported')
    })
  })
})
