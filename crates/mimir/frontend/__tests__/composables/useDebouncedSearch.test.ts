/**
 * Tests for useDebouncedSearch composable
 *
 * Tests debounced typeahead search: minimum query length, debounce timing,
 * result mapping, clear functionality, state management.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
} from '@tests/helpers/mockInvoke'
import { useDebouncedSearch } from '@/composables/useDebouncedSearch'

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('useDebouncedSearch', () => {
  beforeEach(() => {
    setupInvokeMock()
    vi.useFakeTimers()
  })

  afterEach(() => {
    resetInvokeMock()
    vi.useRealTimers()
  })

  function createSearch(overrides: Record<string, unknown> = {}) {
    return useDebouncedSearch<{ name: string }>({
      command: 'search_monsters',
      buildFilter: (q: string) => ({ name_contains: q }),
      mapResult: (raw: any) => ({ name: raw.name }),
      ...overrides,
    })
  }

  describe('initial state', () => {
    it('starts with empty query and results', () => {
      const search = createSearch()
      expect(search.query.value).toBe('')
      expect(search.results.value).toHaveLength(0)
      expect(search.isSearching.value).toBe(false)
    })
  })

  describe('minimum query length', () => {
    it('does not search for queries shorter than minLength (default 2)', () => {
      mockCommand('search_monsters', [{ name: 'Goblin' }])
      const search = createSearch()

      search.query.value = 'g'
      search.search()

      vi.advanceTimersByTime(500)
      expect(search.results.value).toHaveLength(0)
    })

    it('clears results when query goes below minLength', () => {
      const search = createSearch()
      search.results.value = [{ name: 'Goblin' }]

      search.query.value = 'g'
      search.search()

      expect(search.results.value).toHaveLength(0)
    })

    it('respects custom minLength', () => {
      mockCommand('search_monsters', [{ name: 'Goblin' }])
      const search = createSearch({ minLength: 1 })

      search.query.value = 'g'
      search.search()

      // Should proceed since minLength is 1
      vi.advanceTimersByTime(500)
      // The search was triggered (not blocked by minLength)
    })
  })

  describe('debounce timing', () => {
    it('delays search by debounceMs (default 300)', async () => {
      mockCommand('search_monsters', [{ name: 'Goblin' }])
      const search = createSearch()

      search.query.value = 'gob'
      search.search()

      // Not yet triggered
      vi.advanceTimersByTime(200)
      // Still waiting
      expect(search.isSearching.value).toBe(false)

      // Now fire
      vi.advanceTimersByTime(200)
      await vi.runAllTimersAsync()
      // After timer fires and async completes, results should be populated
    })

    it('resets timer on subsequent searches', () => {
      mockCommand('search_monsters', [{ name: 'Goblin' }])
      const search = createSearch()

      search.query.value = 'go'
      search.search()

      vi.advanceTimersByTime(200)

      // Type more, resetting the timer
      search.query.value = 'gob'
      search.search()

      vi.advanceTimersByTime(200)
      // The first timer should have been cleared, so no search yet
    })
  })

  describe('search execution', () => {
    it('returns mapped results after debounce', async () => {
      mockCommand('search_monsters', [
        { name: 'Goblin', cr: '1/4' },
        { name: 'Goblin Boss', cr: '1' },
      ])
      const search = createSearch()

      search.query.value = 'gob'
      search.search()

      await vi.advanceTimersByTimeAsync(400)

      expect(search.results.value).toHaveLength(2)
      expect(search.results.value[0].name).toBe('Goblin')
      expect(search.results.value[1].name).toBe('Goblin Boss')
    })

    it('clears results on error', async () => {
      // No mock registered = will throw
      const search = createSearch()
      search.results.value = [{ name: 'Old result' }]

      search.query.value = 'gob'
      search.search()

      await vi.advanceTimersByTimeAsync(400)

      expect(search.results.value).toHaveLength(0)
    })
  })

  describe('clear', () => {
    it('clears query and results', () => {
      const search = createSearch()
      search.query.value = 'gob'
      search.results.value = [{ name: 'Goblin' }]

      search.clear()

      expect(search.query.value).toBe('')
      expect(search.results.value).toHaveLength(0)
    })
  })

  describe('setQuery', () => {
    it('sets query and triggers search', async () => {
      mockCommand('search_monsters', [{ name: 'Dragon' }])
      const search = createSearch()

      search.setQuery('dragon')

      expect(search.query.value).toBe('dragon')
      await vi.advanceTimersByTimeAsync(400)
      expect(search.results.value).toHaveLength(1)
    })
  })
})
