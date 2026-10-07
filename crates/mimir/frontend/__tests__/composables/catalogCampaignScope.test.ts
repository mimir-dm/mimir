/**
 * Catalog searches send the current campaign id and let the backend apply the
 * campaign's sources (MIMIR-T-0675). Explicit source filters still win.
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { setupInvokeMock, resetInvokeMock, mockCommand, getInvokeMock } from '@tests/helpers/mockInvoke'
import { useCatalogSearch } from '@/features/sources/composables/catalog/useCatalogSearch'
import { useMonsters } from '@/features/sources/composables/catalog/useMonsters'
import { useItems } from '@/features/sources/composables/catalog/useItems'
import { useTraps } from '@/features/sources/composables/catalog/useTraps'
import { useCampaignStore } from '@/stores/campaigns'

function lastArgs(command: string): Record<string, any> {
  const calls = getInvokeMock().mock.calls.filter(([cmd]) => cmd === command)
  expect(calls.length).toBeGreaterThan(0)
  return calls[calls.length - 1][1] as Record<string, any>
}

function selectCampaign(id: string | null, sources: string[] = []) {
  const store = useCampaignStore()
  store.currentCampaign = id ? ({ id, name: 'Test' } as any) : null
  store.currentCampaignSources = sources
}

describe('campaign-scoped catalog search', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    setupInvokeMock()
    for (const cmd of ['search_spells', 'search_monsters', 'search_items', 'search_traps']) {
      mockCommand(cmd, [])
    }
  })

  afterEach(() => {
    resetInvokeMock()
  })

  const runners: Array<[string, (sources?: string[]) => Promise<unknown>]> = [
    [
      'search_spells',
      (sources) =>
        useCatalogSearch({ name: 'spell', searchCommand: 'search_spells', detailsCommand: 'get_spell_by_name' })
          .search(sources ? ({ sources } as any) : ({} as any)),
    ],
    ['search_monsters', (sources) => useMonsters().searchMonsters({ query: '', sources } as any)],
    ['search_items', (sources) => useItems().searchItems({ query: '', sources } as any)],
    ['search_traps', (sources) => useTraps().searchTraps({ query: '', sources } as any)],
  ]

  for (const [command, run] of runners) {
    describe(command, () => {
      it('sends the campaign id and leaves the campaign sources to the backend', async () => {
        selectCampaign('camp-1', ['MM'])
        await run()
        const args = lastArgs(command)
        expect(args.campaignId).toBe('camp-1')
        expect(args.filter.sources).toBeNull()
      })

      it('keeps explicitly chosen sources', async () => {
        selectCampaign('camp-1', ['MM'])
        await run(['PHB'])
        expect(lastArgs(command).filter.sources).toEqual(['PHB'])
      })

      it('sends no campaign id without a selected campaign', async () => {
        selectCampaign(null)
        await run()
        expect(lastArgs(command).campaignId).toBeNull()
      })
    })
  }
})
