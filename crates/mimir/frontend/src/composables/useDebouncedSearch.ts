import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface DebouncedSearchOptions<TResult, TRaw = unknown> {
  /** The Tauri command to invoke for searching */
  command: string
  /** Minimum query length before searching (default: 2) */
  minLength?: number
  /** Debounce delay in ms (default: 300) */
  debounceMs?: number
  /** Maximum results to return (default: 10) */
  limit?: number
  /** Build the filter object from the query string */
  buildFilter: (query: string) => Record<string, unknown>
  /** Transform raw backend results to the desired format */
  mapResult: (raw: TRaw) => TResult
}

export interface DebouncedSearchReturn<TResult> {
  /** Current search query */
  query: Ref<string>
  /** Search results */
  results: Ref<TResult[]>
  /** Whether a search is in progress */
  isSearching: Ref<boolean>
  /** Trigger a search with the current query */
  search: () => void
  /** Clear the query and results */
  clear: () => void
  /** Set query and trigger search */
  setQuery: (value: string) => void
}

/**
 * Composable for debounced search with typeahead results.
 *
 * Handles common patterns like:
 * - Debounced input to avoid excessive API calls
 * - Minimum query length requirement
 * - Result transformation
 * - State cleanup
 *
 * @example
 * ```ts
 * const monsterSearch = useDebouncedSearch<Monster>({
 *   command: 'search_monsters',
 *   buildFilter: (q) => ({ name_contains: q, sources: campaignSources }),
 *   mapResult: (m) => ({ id: m.id, name: m.name, source: m.source, size: m.size, cr: m.cr })
 * })
 *
 * // In template:
 * // <input v-model="monsterSearch.query.value" @input="monsterSearch.search()" />
 * // <div v-for="monster in monsterSearch.results.value" ...>
 * ```
 */
export function useDebouncedSearch<TResult, TRaw = unknown>(
  options: DebouncedSearchOptions<TResult, TRaw>
): DebouncedSearchReturn<TResult> {
  const {
    command,
    minLength = 2,
    debounceMs = 300,
    limit = 10,
    buildFilter,
    mapResult
  } = options

  const query = ref('')
  const results = ref<TResult[]>([]) as Ref<TResult[]>
  const isSearching = ref(false)

  let timeout: ReturnType<typeof setTimeout> | null = null

  function search(): void {
    if (timeout) clearTimeout(timeout)

    if (query.value.length < minLength) {
      results.value = []
      return
    }

    timeout = setTimeout(async () => {
      try {
        isSearching.value = true
        const filter = buildFilter(query.value)

        const response = await invoke<{ success: boolean; data?: TRaw[] }>(
          command,
          { filter, limit, offset: 0 }
        )

        if (response.success && response.data && Array.isArray(response.data)) {
          results.value = response.data.map(mapResult)
        } else {
          results.value = []
        }
      } catch (e) {
        console.error(`Search failed (${command}):`, e)
        results.value = []
      } finally {
        isSearching.value = false
      }
    }, debounceMs)
  }

  function clear(): void {
    if (timeout) clearTimeout(timeout)
    query.value = ''
    results.value = []
  }

  function setQuery(value: string): void {
    query.value = value
    search()
  }

  return {
    query,
    results,
    isSearching,
    search,
    clear,
    setQuery
  }
}
