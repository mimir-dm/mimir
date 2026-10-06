import { describe, it, expect } from 'vitest'
import { canLevelUp, MAX_CHARACTER_LEVEL, totalLevel, type Character } from '../character'

// Only `classes` matters for these helpers.
function withLevels(...levels: number[]): Character {
  return {
    classes: levels.map((level, i) => ({ class_name: `Class${i}`, level })),
  } as unknown as Character
}

describe('level cap (MIMIR-T-0670)', () => {
  it('is 20, matching the backend', () => {
    expect(MAX_CHARACTER_LEVEL).toBe(20)
  })

  it('allows leveling below the cap, counting all classes', () => {
    expect(canLevelUp(withLevels(19))).toBe(true)
    expect(canLevelUp(withLevels(15, 4))).toBe(true)
    expect(canLevelUp(withLevels())).toBe(true)
  })

  it('refuses at the cap, counting all classes', () => {
    expect(canLevelUp(withLevels(20))).toBe(false)
    expect(canLevelUp(withLevels(15, 5))).toBe(false)
    expect(totalLevel(withLevels(15, 5))).toBe(20)
  })
})
