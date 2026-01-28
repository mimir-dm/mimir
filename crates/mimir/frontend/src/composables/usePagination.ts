import { ref, computed, watch, type Ref, type ComputedRef } from 'vue'

export interface PaginationOptions {
  /** Default page size (default: 50) */
  defaultPageSize?: number
  /** Available page size options */
  pageSizeOptions?: number[]
}

export interface PaginationResult<T> {
  /** Current page number (1-indexed) */
  currentPage: Ref<number>
  /** Items per page */
  pageSize: Ref<number>
  /** Total number of items */
  totalItems: ComputedRef<number>
  /** Total number of pages */
  totalPages: ComputedRef<number>
  /** Items for the current page */
  paginatedItems: ComputedRef<T[]>
  /** Whether there's a previous page */
  hasPreviousPage: ComputedRef<boolean>
  /** Whether there's a next page */
  hasNextPage: ComputedRef<boolean>
  /** Start index of current page (1-indexed for display) */
  startIndex: ComputedRef<number>
  /** End index of current page (1-indexed for display) */
  endIndex: ComputedRef<number>
  /** Go to a specific page */
  goToPage: (page: number) => void
  /** Go to next page */
  nextPage: () => void
  /** Go to previous page */
  previousPage: () => void
  /** Go to first page */
  firstPage: () => void
  /** Go to last page */
  lastPage: () => void
  /** Reset to first page (call when data changes) */
  reset: () => void
  /** Available page size options */
  pageSizeOptions: number[]
}

/**
 * Composable for frontend-only pagination.
 * Takes a reactive array of items and provides paginated access.
 */
export function usePagination<T>(
  items: Ref<T[]> | ComputedRef<T[]>,
  options: PaginationOptions = {}
): PaginationResult<T> {
  const {
    defaultPageSize = 50,
    pageSizeOptions = [25, 50, 100, 200]
  } = options

  const currentPage = ref(1)
  const pageSize = ref(defaultPageSize)

  const totalItems = computed(() => items.value.length)

  const totalPages = computed(() => {
    if (totalItems.value === 0) return 1
    return Math.ceil(totalItems.value / pageSize.value)
  })

  const paginatedItems = computed(() => {
    const start = (currentPage.value - 1) * pageSize.value
    const end = start + pageSize.value
    return items.value.slice(start, end)
  })

  const hasPreviousPage = computed(() => currentPage.value > 1)
  const hasNextPage = computed(() => currentPage.value < totalPages.value)

  const startIndex = computed(() => {
    if (totalItems.value === 0) return 0
    return (currentPage.value - 1) * pageSize.value + 1
  })

  const endIndex = computed(() => {
    const end = currentPage.value * pageSize.value
    return Math.min(end, totalItems.value)
  })

  function goToPage(page: number) {
    const validPage = Math.max(1, Math.min(page, totalPages.value))
    currentPage.value = validPage
  }

  function nextPage() {
    if (hasNextPage.value) {
      currentPage.value++
    }
  }

  function previousPage() {
    if (hasPreviousPage.value) {
      currentPage.value--
    }
  }

  function firstPage() {
    currentPage.value = 1
  }

  function lastPage() {
    currentPage.value = totalPages.value
  }

  function reset() {
    currentPage.value = 1
  }

  // Reset to first page when items change significantly
  watch(totalItems, (newTotal, oldTotal) => {
    // If we're beyond the last page, go to last page
    if (currentPage.value > totalPages.value) {
      currentPage.value = Math.max(1, totalPages.value)
    }
  })

  // Reset to first page when page size changes
  watch(pageSize, () => {
    currentPage.value = 1
  })

  return {
    currentPage,
    pageSize,
    totalItems,
    totalPages,
    paginatedItems,
    hasPreviousPage,
    hasNextPage,
    startIndex,
    endIndex,
    goToPage,
    nextPage,
    previousPage,
    firstPage,
    lastPage,
    reset,
    pageSizeOptions
  }
}
