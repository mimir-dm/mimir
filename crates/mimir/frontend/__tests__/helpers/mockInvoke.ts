/**
 * Tauri invoke mock helpers for Vitest.
 *
 * Provides a routing layer over the global `invoke` mock (set up in setup.ts)
 * so tests can register handlers per command name instead of relying on
 * fragile `.mockResolvedValueOnce()` call ordering.
 *
 * Usage:
 *   import { setupInvokeMock, mockCommand, mockCommandError, getInvokeMock } from '@tests/helpers/mockInvoke'
 *
 *   beforeEach(() => { setupInvokeMock() })
 *   afterEach(() => { resetInvokeMock() })
 *
 *   it('loads data', async () => {
 *     mockCommand('get_character', { id: 1, name: 'Test' })
 *     // component mounts, calls invoke('get_character', ...) → gets the fixture
 *   })
 */

import { vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import type { ApiResponse } from '@/types/api'

// ─── Types ───────────────────────────────────────────────────────────────────

export type InvokeHandler = (params?: Record<string, unknown>) => unknown | Promise<unknown>

interface CommandRegistration {
  /** If set, this function is called with the invoke params and its return value is used */
  handler?: InvokeHandler
  /** If set, this static response is returned */
  response?: unknown
  /** If true, calls after the first use the same response; if false, one-shot */
  persistent: boolean
  /** How many times this command has been called */
  callCount: number
}

// ─── State ───────────────────────────────────────────────────────────────────

const commandMap = new Map<string, CommandRegistration>()
let fallbackHandler: InvokeHandler | null = null
let mockInvokeInstance: ReturnType<typeof vi.mocked<typeof invoke>> | null = null

// ─── Setup / Teardown ────────────────────────────────────────────────────────

/**
 * Install the invoke router. Call in beforeEach.
 * The global vi.mock in setup.ts already replaces @tauri-apps/api/core,
 * so we just need to set the implementation on the existing mock.
 */
export function setupInvokeMock(): ReturnType<typeof vi.mocked<typeof invoke>> {
  mockInvokeInstance = vi.mocked(invoke)
  commandMap.clear()
  fallbackHandler = null

  mockInvokeInstance.mockImplementation(async (cmd: string, params?: unknown) => {
    const reg = commandMap.get(cmd)

    if (reg) {
      reg.callCount++

      let result: unknown
      if (reg.handler) {
        result = await reg.handler(params as Record<string, unknown>)
      } else {
        result = reg.response
      }

      // Remove one-shot registrations after first call
      if (!reg.persistent) {
        commandMap.delete(cmd)
      }

      return result
    }

    if (fallbackHandler) {
      return fallbackHandler(params as Record<string, unknown>)
    }

    // Default: return unsuccessful ApiResponse so components degrade gracefully
    return { success: false, error: `No mock registered for command: ${cmd}` }
  })

  return mockInvokeInstance
}

/**
 * Reset all registered mocks. Call in afterEach.
 */
export function resetInvokeMock(): void {
  commandMap.clear()
  fallbackHandler = null
  if (mockInvokeInstance) {
    mockInvokeInstance.mockReset()
    mockInvokeInstance = null
  }
}

/**
 * Get the underlying vi.mocked(invoke) instance for direct assertions.
 */
export function getInvokeMock(): ReturnType<typeof vi.mocked<typeof invoke>> {
  if (!mockInvokeInstance) {
    throw new Error('setupInvokeMock() must be called before getInvokeMock()')
  }
  return mockInvokeInstance
}

// ─── Command Registration ────────────────────────────────────────────────────

/**
 * Register a successful response for a command.
 * Wraps `data` in `{ success: true, data }` (the ApiResponse format).
 *
 * @param command  The Tauri command name (e.g. 'get_character')
 * @param data     The data payload to return
 * @param options  persistent: keep responding on repeated calls (default true)
 */
export function mockCommand<T>(
  command: string,
  data: T,
  options: { persistent?: boolean } = {},
): void {
  const { persistent = true } = options
  commandMap.set(command, {
    response: { success: true, data } as ApiResponse<T>,
    persistent,
    callCount: 0,
  })
}

/**
 * Register an error response for a command.
 *
 * @param command  The Tauri command name
 * @param error    The error message
 * @param options  persistent: keep responding (default true)
 */
export function mockCommandError(
  command: string,
  error: string,
  options: { persistent?: boolean } = {},
): void {
  const { persistent = true } = options
  commandMap.set(command, {
    response: { success: false, error } as ApiResponse<never>,
    persistent,
    callCount: 0,
  })
}

/**
 * Register a raw response (not wrapped in ApiResponse) for commands
 * that don't use the ApiResponse pattern.
 */
export function mockCommandRaw(
  command: string,
  response: unknown,
  options: { persistent?: boolean } = {},
): void {
  const { persistent = true } = options
  commandMap.set(command, {
    response,
    persistent,
    callCount: 0,
  })
}

/**
 * Register a dynamic handler for a command. The handler receives the
 * invoke params and should return the full response (including ApiResponse wrapper).
 */
export function mockCommandHandler(
  command: string,
  handler: InvokeHandler,
  options: { persistent?: boolean } = {},
): void {
  const { persistent = true } = options
  commandMap.set(command, {
    handler,
    persistent,
    callCount: 0,
  })
}

/**
 * Register a command that rejects with an error (simulates invoke throwing).
 */
export function mockCommandReject(
  command: string,
  error: string | Error,
  options: { persistent?: boolean } = {},
): void {
  const { persistent = true } = options
  const err = typeof error === 'string' ? new Error(error) : error
  commandMap.set(command, {
    handler: () => { throw err },
    persistent,
    callCount: 0,
  })
}

/**
 * Register a sequence of responses for a command. Each call consumes the next
 * response in the array. After the array is exhausted, the command is unregistered.
 */
export function mockCommandSequence<T>(
  command: string,
  responses: ApiResponse<T>[],
): void {
  let index = 0
  commandMap.set(command, {
    handler: () => {
      if (index < responses.length) {
        return responses[index++]
      }
      return { success: false, error: `Mock sequence exhausted for: ${command}` }
    },
    persistent: true,
    callCount: 0,
  })
}

/**
 * Set a fallback handler for any unregistered command.
 * Useful for tests that want to catch unexpected calls.
 */
export function setInvokeFallback(handler: InvokeHandler): void {
  fallbackHandler = handler
}

// ─── Assertions ──────────────────────────────────────────────────────────────

/**
 * Get the call count for a registered command.
 */
export function getCommandCallCount(command: string): number {
  return commandMap.get(command)?.callCount ?? 0
}

/**
 * Assert that a command was called (at least once).
 */
export function expectCommandCalled(command: string): void {
  const mock = getInvokeMock()
  const calls = mock.mock.calls.filter(([cmd]) => cmd === command)
  if (calls.length === 0) {
    throw new Error(`Expected command '${command}' to have been called, but it was not`)
  }
}

/**
 * Assert that a command was called with specific params.
 */
export function expectCommandCalledWith(command: string, params: Record<string, unknown>): void {
  const mock = getInvokeMock()
  const calls = mock.mock.calls.filter(([cmd]) => cmd === command)
  if (calls.length === 0) {
    throw new Error(`Expected command '${command}' to have been called, but it was not`)
  }
  const match = calls.some(([, p]) => {
    try {
      expect(p).toMatchObject(params)
      return true
    } catch {
      return false
    }
  })
  if (!match) {
    throw new Error(
      `Expected command '${command}' to have been called with ${JSON.stringify(params)}, ` +
      `but calls were: ${JSON.stringify(calls.map(([, p]) => p))}`,
    )
  }
}

/**
 * Get all calls to a specific command for manual inspection.
 */
export function getCommandCalls(command: string): unknown[][] {
  const mock = getInvokeMock()
  return mock.mock.calls.filter(([cmd]) => cmd === command)
}
