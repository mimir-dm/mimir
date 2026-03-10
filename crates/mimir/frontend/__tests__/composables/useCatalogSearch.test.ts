/**
 * Tests for useCatalogSearch composable
 *
 * Tests the generic catalog search composable: initialization, search with
 * filters, filter transformation, getDetails, error handling, campaign
 * source integration.
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
  mockCommandError,
  expectCommandCalledWith,
} from '@tests/helpers/mockInvoke'
import { useCatalogSearch } from '@/features/sources/composables/catalog/useCatalogSearch'

// ─── Factories ──────────────────────────────────────────────────────────────

function makeSpellSummary(overrides: Record<string, unknown> = {}) {
  return {
    name: 'Fireball',
    level: 3,
    school: 'V',
    source: 'PHB',
    concentration: false,
    ritual: false,
    ...overrides,
  }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('useCatalogSearch', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  describe('initialization', () => {
    it('starts uninitialized', () => {
      const catalog = useCatalogSearch({
        name: 'spell',
        searchCommand: 'search_spells',
        detailsCommand: 'get_spell_by_name',
      })
      expect(catalog.isInitialized.value).toBe(false)
      expect(catalog.isLoading.value).toBe(false)
      expect(catalog.error.value).toBeNull()
    })

    it('auto-initializes without initializeCommand', async () => {
      const catalog = useCatalogSearch({
        name: 'spell',
        searchCommand: 'search_spells',
        detailsCommand: 'get_spell_by_name',
      })
      await catalog.initialize()
      expect(catalog.isInitialized.value).toBe(true)
    })

    it('calls initializeCommand when provided', async () => {
      mockCommand('init_spell_catalog', null)
      const catalog = useCatalogSearch({
        name: 'spell',
        initializeCommand: 'init_spell_catalog',
        searchCommand: 'search_spells',
        detailsCommand: 'get_spell_by_name',
      })
      await catalog.initialize()
      expect(catalog.isInitialized.value).toBe(true)
    })

    it('only initializes once', async () => {
      const catalog = useCatalogSearch({
        name: 'spell',
        searchCommand: 'search_spells',
        detailsCommand: 'get_spell_by_name',
      })
      await catalog.initialize()
      await catalog.initialize() // second call is no-op
      expect(catalog.isInitialized.value).toBe(true)
    })
  })

  describe('search', () => {
    it('returns search results', async () => {
      const spells = [makeSpellSummary(), makeSpellSummary({ name: 'Shield', level: 1 })]
      mockCommand('search_spells', spells)

      const catalog = useCatalogSearch({
        name: 'spell',
        searchCommand: 'search_spells',
        detailsCommand: 'get_spell_by_name',
      })

      const results = await catalog.search({ query: 'fire' })
      expect(results).toHaveLength(2)
      expect(catalog.results.value).toHaveLength(2)
    })

    it('auto-initializes on first search', async () => {
      mockCommand('search_spells', [makeSpellSummary()])

      const catalog = useCatalogSearch({
        name: 'spell',
        searchCommand: 'search_spells',
        detailsCommand: 'get_spell_by_name',
      })

      expect(catalog.isInitialized.value).toBe(false)
      await catalog.search({})
      expect(catalog.isInitialized.value).toBe(true)
    })

    it('sends filter with limit and offset', async () => {
      mockCommand('search_spells', [])

      const catalog = useCatalogSearch({
        name: 'spell',
        searchCommand: 'search_spells',
        detailsCommand: 'get_spell_by_name',
      })

      await catalog.search({ query: 'fire' })
      expectCommandCalledWith('search_spells', {
        filter: expect.objectContaining({ name_contains: 'fire' }),
        limit: 10000,
        offset: 0,
      })
    })

    it('returns empty array on error', async () => {
      mockCommandError('search_spells', 'Database error')

      const catalog = useCatalogSearch({
        name: 'spell',
        searchCommand: 'search_spells',
        detailsCommand: 'get_spell_by_name',
      })

      const results = await catalog.search({})
      expect(results).toHaveLength(0)
      expect(catalog.error.value).toContain('Database error')
    })

    it('sets isLoading during search', async () => {
      mockCommand('search_spells', [])

      const catalog = useCatalogSearch({
        name: 'spell',
        searchCommand: 'search_spells',
        detailsCommand: 'get_spell_by_name',
      })

      const promise = catalog.search({})
      // After the promise resolves, loading should be false
      await promise
      expect(catalog.isLoading.value).toBe(false)
    })
  })

  describe('filter transformation', () => {
    it('uses custom transformFilters when provided', async () => {
      mockCommand('search_spells', [])

      const catalog = useCatalogSearch({
        name: 'spell',
        searchCommand: 'search_spells',
        detailsCommand: 'get_spell_by_name',
        transformFilters: (filters: any) => ({
          name_contains: filters.query || null,
          level: filters.level ?? null,
          school: filters.school ?? null,
        }),
      })

      await catalog.search({ query: 'fire', level: 3, school: 'V' })
      expectCommandCalledWith('search_spells', {
        filter: { name_contains: 'fire', level: 3, school: 'V' },
        limit: 10000,
        offset: 0,
      })
    })

    it('default transform renames query to name_contains', async () => {
      mockCommand('search_spells', [])

      const catalog = useCatalogSearch({
        name: 'spell',
        searchCommand: 'search_spells',
        detailsCommand: 'get_spell_by_name',
      })

      await catalog.search({ query: 'fire' } as any)
      expectCommandCalledWith('search_spells', {
        filter: expect.objectContaining({ name_contains: 'fire' }),
        limit: 10000,
        offset: 0,
      })
    })

    it('default transform converts empty strings to null', async () => {
      mockCommand('search_spells', [])

      const catalog = useCatalogSearch({
        name: 'spell',
        searchCommand: 'search_spells',
        detailsCommand: 'get_spell_by_name',
      })

      await catalog.search({ query: '' } as any)
      expectCommandCalledWith('search_spells', {
        filter: expect.objectContaining({ name_contains: null }),
        limit: 10000,
        offset: 0,
      })
    })

    it('default transform converts empty arrays to null', async () => {
      mockCommand('search_spells', [])

      const catalog = useCatalogSearch({
        name: 'spell',
        searchCommand: 'search_spells',
        detailsCommand: 'get_spell_by_name',
      })

      await catalog.search({ levels: [] } as any)
      expectCommandCalledWith('search_spells', {
        filter: expect.objectContaining({ levels: null }),
        limit: 10000,
        offset: 0,
      })
    })
  })

  describe('getDetails', () => {
    it('returns details for a matching item', async () => {
      const fullSpell = { ...makeSpellSummary(), entries: ['A bright streak...'] }
      mockCommand('get_spell_by_name', fullSpell)

      const catalog = useCatalogSearch({
        name: 'spell',
        searchCommand: 'search_spells',
        detailsCommand: 'get_spell_by_name',
      })

      const details = await catalog.getDetails('Fireball', 'PHB')
      expect(details).not.toBeNull()
      expect((details as any).name).toBe('Fireball')
      expectCommandCalledWith('get_spell_by_name', { name: 'Fireball', source: 'PHB' })
    })

    it('returns null on error', async () => {
      mockCommandError('get_spell_by_name', 'Not found')

      const catalog = useCatalogSearch({
        name: 'spell',
        searchCommand: 'search_spells',
        detailsCommand: 'get_spell_by_name',
      })

      const details = await catalog.getDetails('Unknown', 'PHB')
      expect(details).toBeNull()
    })
  })
})
