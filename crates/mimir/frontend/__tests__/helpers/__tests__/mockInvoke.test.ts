import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
  mockCommandError,
  mockCommandRaw,
  mockCommandHandler,
  mockCommandReject,
  mockCommandSequence,
  getInvokeMock,
  expectCommandCalled,
  expectCommandCalledWith,
  getCommandCalls,
  setInvokeFallback,
} from '../mockInvoke'
import { invoke } from '@tauri-apps/api/core'

describe('mockInvoke helpers', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  describe('mockCommand', () => {
    it('returns ApiResponse with success and data', async () => {
      mockCommand('get_character', { id: 1, name: 'Test' })

      const result = await invoke('get_character', { characterId: 1 })

      expect(result).toEqual({
        success: true,
        data: { id: 1, name: 'Test' },
      })
    })

    it('persists across multiple calls by default', async () => {
      mockCommand('list_campaigns', [{ id: 1 }, { id: 2 }])

      const r1 = await invoke('list_campaigns')
      const r2 = await invoke('list_campaigns')

      expect(r1).toEqual(r2)
    })

    it('supports one-shot mode', async () => {
      mockCommand('get_item', { name: 'Sword' }, { persistent: false })

      const r1 = await invoke('get_item')
      expect(r1).toEqual({ success: true, data: { name: 'Sword' } })

      // Second call returns unregistered error
      const r2 = await invoke('get_item') as { success: boolean; error?: string }
      expect(r2.success).toBe(false)
    })
  })

  describe('mockCommandError', () => {
    it('returns ApiResponse with error', async () => {
      mockCommandError('get_character', 'Character not found')

      const result = await invoke('get_character', { characterId: 999 }) as { success: boolean; error?: string }

      expect(result).toEqual({
        success: false,
        error: 'Character not found',
      })
    })
  })

  describe('mockCommandRaw', () => {
    it('returns raw value without ApiResponse wrapper', async () => {
      mockCommandRaw('get_themes', [{ id: 'dark', name: 'Dark' }])

      const result = await invoke('get_themes')

      expect(result).toEqual([{ id: 'dark', name: 'Dark' }])
    })
  })

  describe('mockCommandHandler', () => {
    it('calls handler with params and returns its result', async () => {
      mockCommandHandler('search_monsters', (params) => ({
        success: true,
        data: [{ name: `Monster matching ${(params as Record<string, string>)?.query}` }],
      }))

      const result = await invoke('search_monsters', { query: 'dragon' }) as { success: boolean; data?: unknown[] }

      expect(result.success).toBe(true)
      expect(result.data).toEqual([{ name: 'Monster matching dragon' }])
    })
  })

  describe('mockCommandReject', () => {
    it('makes invoke throw an error', async () => {
      mockCommandReject('broken_command', 'Network failure')

      await expect(invoke('broken_command')).rejects.toThrow('Network failure')
    })
  })

  describe('mockCommandSequence', () => {
    it('returns responses in order', async () => {
      mockCommandSequence('get_status', [
        { success: true, data: 'loading' },
        { success: true, data: 'ready' },
      ])

      const r1 = await invoke('get_status') as { data: string }
      expect(r1.data).toBe('loading')

      const r2 = await invoke('get_status') as { data: string }
      expect(r2.data).toBe('ready')
    })

    it('returns error when sequence is exhausted', async () => {
      mockCommandSequence('get_once', [
        { success: true, data: 'only' },
      ])

      await invoke('get_once')
      const r2 = await invoke('get_once') as { success: boolean }
      expect(r2.success).toBe(false)
    })
  })

  describe('fallback handler', () => {
    it('catches unregistered commands', async () => {
      setInvokeFallback(() => ({ success: true, data: 'fallback' }))

      const result = await invoke('unknown_command') as { data: string }
      expect(result.data).toBe('fallback')
    })

    it('returns error by default for unregistered commands', async () => {
      const result = await invoke('no_such_command') as { success: boolean; error?: string }
      expect(result.success).toBe(false)
      expect(result.error).toContain('no_such_command')
    })
  })

  describe('assertions', () => {
    it('expectCommandCalled passes when command was called', async () => {
      mockCommand('test_cmd', {})
      await invoke('test_cmd')

      expect(() => expectCommandCalled('test_cmd')).not.toThrow()
    })

    it('expectCommandCalled throws when command was not called', () => {
      expect(() => expectCommandCalled('never_called')).toThrow()
    })

    it('expectCommandCalledWith checks params', async () => {
      mockCommand('test_cmd', {})
      await invoke('test_cmd', { id: 42, name: 'test' })

      expect(() => expectCommandCalledWith('test_cmd', { id: 42 })).not.toThrow()
    })

    it('getCommandCalls returns all calls to a command', async () => {
      mockCommand('multi_call', {})
      await invoke('multi_call', { a: 1 })
      await invoke('multi_call', { a: 2 })

      const calls = getCommandCalls('multi_call')
      expect(calls).toHaveLength(2)
    })
  })

  describe('getInvokeMock', () => {
    it('returns the vi.mocked(invoke) instance', () => {
      const mock = getInvokeMock()
      expect(mock).toBeDefined()
      expect(typeof mock.mockImplementation).toBe('function')
    })

    it('throws when called before setup', () => {
      resetInvokeMock()
      expect(() => getInvokeMock()).toThrow('setupInvokeMock()')
    })
  })
})
