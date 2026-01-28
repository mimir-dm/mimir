import { ref, computed, type Ref, type ComputedRef } from 'vue'

/**
 * Base feature reference - name and source
 */
export interface FeatureRef {
  name: string
  source: string
}

/**
 * Feature item with additional display properties
 */
export interface FeatureItem extends FeatureRef {
  description?: string
  cost?: number | string
  prereqs?: string
  disabled?: boolean
}

/**
 * Options for configuring the feature selection composable
 */
export interface FeatureSelectionOptions<T extends FeatureRef> {
  /** Maximum number of items that can be selected (for multi-select) */
  maxSlots?: ComputedRef<number> | Ref<number> | number
  /** Function to get the unique key for an item (defaults to name) */
  getKey?: (item: T) => string
  /** Callback when selection changes */
  onSelectionChange?: (selected: T[]) => void
  /** Initial selected items */
  initialSelection?: T[]
}

/**
 * Return type for the useFeatureSelection composable
 */
export interface FeatureSelectionReturn<T extends FeatureRef> {
  /** Currently selected items */
  selected: Ref<T[]>
  /** Check if an item is selected */
  isSelected: (item: T) => boolean
  /** Toggle selection of an item (for multi-select) */
  toggle: (item: T) => void
  /** Select a single item (clears previous selection for single-select) */
  select: (item: T) => void
  /** Deselect an item */
  deselect: (item: T) => void
  /** Clear all selections */
  clear: () => void
  /** Whether the maximum selection limit has been reached */
  isAtLimit: ComputedRef<boolean>
  /** Number of slots remaining */
  remainingSlots: ComputedRef<number>
  /** Current selection count */
  selectionCount: ComputedRef<number>
}

/**
 * Generic composable for managing feature selections (single or multi-select)
 *
 * Handles common patterns like:
 * - Toggle selection with slot limits
 * - Check if item is selected
 * - Track remaining slots
 *
 * @example
 * ```ts
 * // Multi-select with 2 slots
 * const { selected, toggle, isSelected, isAtLimit } = useFeatureSelection<MetamagicOption>({
 *   maxSlots: computed(() => 2),
 *   onSelectionChange: (items) => updatePayload(items)
 * })
 *
 * // Single-select
 * const { selected, select, isSelected } = useFeatureSelection<FightingStyle>({
 *   maxSlots: 1
 * })
 * ```
 */
export function useFeatureSelection<T extends FeatureRef>(
  options: FeatureSelectionOptions<T> = {}
): FeatureSelectionReturn<T> {
  const {
    maxSlots = Infinity,
    getKey = (item: T) => item.name,
    onSelectionChange,
    initialSelection = []
  } = options

  // Resolve maxSlots to a computed
  const maxSlotsValue = computed(() => {
    if (typeof maxSlots === 'number') return maxSlots
    return maxSlots.value
  })

  // Selection state
  const selected = ref<T[]>([...initialSelection]) as Ref<T[]>

  // Computed properties
  const selectionCount = computed(() => selected.value.length)
  const isAtLimit = computed(() => selected.value.length >= maxSlotsValue.value)
  const remainingSlots = computed(() => Math.max(0, maxSlotsValue.value - selected.value.length))

  /**
   * Check if an item is currently selected
   */
  function isSelected(item: T): boolean {
    const key = getKey(item)
    return selected.value.some(s => getKey(s) === key)
  }

  /**
   * Toggle selection of an item (for multi-select mode)
   */
  function toggle(item: T): void {
    const key = getKey(item)
    const index = selected.value.findIndex(s => getKey(s) === key)

    if (index >= 0) {
      // Already selected, remove it
      selected.value.splice(index, 1)
    } else if (selected.value.length < maxSlotsValue.value) {
      // Not selected and have room, add it
      selected.value.push(item)
    }
    // If at limit and not selected, do nothing

    onSelectionChange?.(selected.value)
  }

  /**
   * Select a single item (for single-select mode, replaces current selection)
   */
  function select(item: T): void {
    selected.value = [item]
    onSelectionChange?.(selected.value)
  }

  /**
   * Deselect a specific item
   */
  function deselect(item: T): void {
    const key = getKey(item)
    const index = selected.value.findIndex(s => getKey(s) === key)
    if (index >= 0) {
      selected.value.splice(index, 1)
      onSelectionChange?.(selected.value)
    }
  }

  /**
   * Clear all selections
   */
  function clear(): void {
    selected.value = []
    onSelectionChange?.(selected.value)
  }

  return {
    selected,
    isSelected,
    toggle,
    select,
    deselect,
    clear,
    isAtLimit,
    remainingSlots,
    selectionCount
  }
}

/**
 * Convenience type for string-based feature selection (like expertise skills)
 */
export interface StringFeatureSelectionReturn {
  selected: Ref<string[]>
  isSelected: (item: string) => boolean
  toggle: (item: string) => void
  deselect: (item: string) => void
  clear: () => void
  isAtLimit: ComputedRef<boolean>
  remainingSlots: ComputedRef<number>
  selectionCount: ComputedRef<number>
}

/**
 * Simplified composable for string-based selections (like skill expertise)
 */
export function useStringFeatureSelection(
  options: {
    maxSlots?: ComputedRef<number> | Ref<number> | number
    onSelectionChange?: (selected: string[]) => void
    initialSelection?: string[]
  } = {}
): StringFeatureSelectionReturn {
  const {
    maxSlots = Infinity,
    onSelectionChange,
    initialSelection = []
  } = options

  const maxSlotsValue = computed(() => {
    if (typeof maxSlots === 'number') return maxSlots
    return maxSlots.value
  })

  const selected = ref<string[]>([...initialSelection])

  const selectionCount = computed(() => selected.value.length)
  const isAtLimit = computed(() => selected.value.length >= maxSlotsValue.value)
  const remainingSlots = computed(() => Math.max(0, maxSlotsValue.value - selected.value.length))

  function isSelected(item: string): boolean {
    return selected.value.includes(item)
  }

  function toggle(item: string): void {
    const index = selected.value.indexOf(item)
    if (index >= 0) {
      selected.value.splice(index, 1)
    } else if (selected.value.length < maxSlotsValue.value) {
      selected.value.push(item)
    }
    onSelectionChange?.(selected.value)
  }

  function deselect(item: string): void {
    const index = selected.value.indexOf(item)
    if (index >= 0) {
      selected.value.splice(index, 1)
      onSelectionChange?.(selected.value)
    }
  }

  function clear(): void {
    selected.value = []
    onSelectionChange?.(selected.value)
  }

  return {
    selected,
    isSelected,
    toggle,
    deselect,
    clear,
    isAtLimit,
    remainingSlots,
    selectionCount
  }
}
