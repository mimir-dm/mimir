/**
 * Tests for useFeatureSelection composable.
 *
 * Tests multi-select with slot limits, single-select, toggle behavior,
 * callbacks, initial selection, and the string variant.
 */

import { describe, it, expect, vi } from 'vitest'
import { ref, computed } from 'vue'
import {
  useFeatureSelection,
  useStringFeatureSelection,
  type FeatureItem,
} from '../useFeatureSelection'

function makeFeature(name: string, source = 'PHB'): FeatureItem {
  return { name, source }
}

describe('useFeatureSelection', () => {
  describe('initial state', () => {
    it('starts with empty selection', () => {
      const { selected, selectionCount, isAtLimit, remainingSlots } =
        useFeatureSelection<FeatureItem>()
      expect(selected.value).toEqual([])
      expect(selectionCount.value).toBe(0)
      expect(isAtLimit.value).toBe(false)
      expect(remainingSlots.value).toBe(Infinity)
    })

    it('accepts initial selection', () => {
      const feat1 = makeFeature('GWM')
      const { selected, selectionCount } = useFeatureSelection<FeatureItem>({
        initialSelection: [feat1],
      })
      expect(selected.value).toHaveLength(1)
      expect(selectionCount.value).toBe(1)
    })
  })

  describe('toggle (multi-select)', () => {
    it('adds item when not selected', () => {
      const { toggle, selected } = useFeatureSelection<FeatureItem>()
      const feat = makeFeature('Defense')
      toggle(feat)
      expect(selected.value).toHaveLength(1)
      expect(selected.value[0].name).toBe('Defense')
    })

    it('removes item when already selected', () => {
      const feat = makeFeature('Defense')
      const { toggle, selected } = useFeatureSelection<FeatureItem>({
        initialSelection: [feat],
      })
      toggle(feat)
      expect(selected.value).toHaveLength(0)
    })

    it('respects maxSlots limit', () => {
      const { toggle, selected, isAtLimit } = useFeatureSelection<FeatureItem>({
        maxSlots: 2,
      })
      toggle(makeFeature('A'))
      toggle(makeFeature('B'))
      expect(isAtLimit.value).toBe(true)

      toggle(makeFeature('C'))
      expect(selected.value).toHaveLength(2) // C was rejected
    })

    it('allows removal even at limit', () => {
      const { toggle, selected } = useFeatureSelection<FeatureItem>({
        maxSlots: 1,
      })
      const feat = makeFeature('A')
      toggle(feat)
      expect(selected.value).toHaveLength(1)

      toggle(feat) // deselect
      expect(selected.value).toHaveLength(0)
    })

    it('calls onSelectionChange callback', () => {
      const onChange = vi.fn()
      const { toggle } = useFeatureSelection<FeatureItem>({
        onSelectionChange: onChange,
      })
      toggle(makeFeature('A'))
      expect(onChange).toHaveBeenCalledWith(expect.arrayContaining([
        expect.objectContaining({ name: 'A' }),
      ]))
    })
  })

  describe('select (single-select)', () => {
    it('replaces previous selection', () => {
      const { select, selected } = useFeatureSelection<FeatureItem>({ maxSlots: 1 })
      select(makeFeature('A'))
      expect(selected.value[0].name).toBe('A')

      select(makeFeature('B'))
      expect(selected.value).toHaveLength(1)
      expect(selected.value[0].name).toBe('B')
    })
  })

  describe('deselect', () => {
    it('removes specific item', () => {
      const a = makeFeature('A')
      const b = makeFeature('B')
      const { deselect, selected } = useFeatureSelection<FeatureItem>({
        initialSelection: [a, b],
      })
      deselect(a)
      expect(selected.value).toHaveLength(1)
      expect(selected.value[0].name).toBe('B')
    })

    it('does nothing for unselected item', () => {
      const { deselect, selected } = useFeatureSelection<FeatureItem>()
      deselect(makeFeature('X'))
      expect(selected.value).toHaveLength(0)
    })
  })

  describe('clear', () => {
    it('removes all selections', () => {
      const { toggle, clear, selected } = useFeatureSelection<FeatureItem>()
      toggle(makeFeature('A'))
      toggle(makeFeature('B'))
      expect(selected.value).toHaveLength(2)

      clear()
      expect(selected.value).toHaveLength(0)
    })
  })

  describe('isSelected', () => {
    it('returns true for selected item', () => {
      const feat = makeFeature('A')
      const { isSelected } = useFeatureSelection<FeatureItem>({
        initialSelection: [feat],
      })
      expect(isSelected(feat)).toBe(true)
    })

    it('returns false for unselected item', () => {
      const { isSelected } = useFeatureSelection<FeatureItem>()
      expect(isSelected(makeFeature('A'))).toBe(false)
    })

    it('uses custom getKey for matching', () => {
      const { toggle, isSelected } = useFeatureSelection<FeatureItem>({
        getKey: (item) => `${item.name}|${item.source}`,
      })
      const phb = makeFeature('Defense', 'PHB')
      const xge = makeFeature('Defense', 'XGE')
      toggle(phb)

      expect(isSelected(phb)).toBe(true)
      expect(isSelected(xge)).toBe(false)
    })
  })

  describe('remainingSlots', () => {
    it('tracks remaining slots accurately', () => {
      const { toggle, remainingSlots } = useFeatureSelection<FeatureItem>({
        maxSlots: 3,
      })
      expect(remainingSlots.value).toBe(3)
      toggle(makeFeature('A'))
      expect(remainingSlots.value).toBe(2)
      toggle(makeFeature('B'))
      expect(remainingSlots.value).toBe(1)
      toggle(makeFeature('C'))
      expect(remainingSlots.value).toBe(0)
    })

    it('never goes below zero', () => {
      const { toggle, remainingSlots } = useFeatureSelection<FeatureItem>({
        maxSlots: 1,
      })
      toggle(makeFeature('A'))
      toggle(makeFeature('B')) // rejected
      expect(remainingSlots.value).toBe(0)
    })
  })

  describe('reactive maxSlots', () => {
    it('responds to computed maxSlots changes', () => {
      const slots = ref(2)
      const { toggle, isAtLimit, remainingSlots } = useFeatureSelection<FeatureItem>({
        maxSlots: computed(() => slots.value),
      })
      toggle(makeFeature('A'))
      toggle(makeFeature('B'))
      expect(isAtLimit.value).toBe(true)

      slots.value = 3
      expect(isAtLimit.value).toBe(false)
      expect(remainingSlots.value).toBe(1)
    })
  })
})

describe('useStringFeatureSelection', () => {
  it('works with string items', () => {
    const { toggle, selected, isSelected } = useStringFeatureSelection({
      maxSlots: 2,
    })
    toggle('Stealth')
    toggle('Perception')

    expect(selected.value).toEqual(['Stealth', 'Perception'])
    expect(isSelected('Stealth')).toBe(true)
    expect(isSelected('Athletics')).toBe(false)
  })

  it('respects maxSlots', () => {
    const { toggle, selected, isAtLimit } = useStringFeatureSelection({
      maxSlots: 1,
    })
    toggle('Stealth')
    toggle('Perception') // rejected
    expect(selected.value).toEqual(['Stealth'])
    expect(isAtLimit.value).toBe(true)
  })

  it('deselects by name', () => {
    const { toggle, deselect, selected } = useStringFeatureSelection()
    toggle('Stealth')
    toggle('Perception')
    deselect('Stealth')
    expect(selected.value).toEqual(['Perception'])
  })

  it('clears all', () => {
    const { toggle, clear, selected } = useStringFeatureSelection()
    toggle('A')
    toggle('B')
    clear()
    expect(selected.value).toEqual([])
  })
})
