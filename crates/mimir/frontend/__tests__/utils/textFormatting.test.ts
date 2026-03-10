/**
 * Tests for textFormatting.ts
 *
 * Tests the 5etools tag processing utility that converts
 * 5etools formatting tags to HTML elements.
 */

import { describe, it, expect } from 'vitest'
import { processFormattingTags, formatEntries } from '@/features/sources/utils/textFormatting'

describe('processFormattingTags', () => {
  describe('basic formatting', () => {
    it('converts bold tags', () => {
      expect(processFormattingTags('{@b hello}')).toBe('<strong>hello</strong>')
    })

    it('converts italic tags', () => {
      expect(processFormattingTags('{@i hello}')).toBe('<em>hello</em>')
    })

    it('converts bold-italic tags', () => {
      expect(processFormattingTags('{@bi hello}')).toBe('<strong><em>hello</em></strong>')
    })

    it('handles multiple formatting tags in one string', () => {
      const result = processFormattingTags('The {@b warrior} was {@i fast}')
      expect(result).toContain('<strong>warrior</strong>')
      expect(result).toContain('<em>fast</em>')
    })
  })

  describe('dice rolls', () => {
    it('converts simple dice tags', () => {
      const result = processFormattingTags('{@dice 2d6}')
      expect(result).toContain('class="dice-roll"')
      expect(result).toContain('2d6')
    })

    it('converts dice with display text', () => {
      const result = processFormattingTags('{@dice 2d6|two dice}')
      expect(result).toContain('two dice')
    })

    it('converts damage tags', () => {
      const result = processFormattingTags('{@damage 3d8}')
      expect(result).toContain('class="damage-roll"')
      expect(result).toContain('3d8')
    })

    it('converts hit bonus tags', () => {
      const result = processFormattingTags('{@hit +5}')
      expect(result).toContain('class="hit-bonus"')
      expect(result).toContain('+5')
    })
  })

  describe('cross-references', () => {
    it('converts spell references to clickable links', () => {
      const result = processFormattingTags('{@spell fireball}')
      expect(result).toContain('class="cross-ref-link spell-ref"')
      expect(result).toContain('data-ref-type="spell"')
      expect(result).toContain('data-ref-name="fireball"')
      expect(result).toContain('data-ref-source="PHB"')
      expect(result).toContain('fireball</a>')
    })

    it('converts spell references with explicit source', () => {
      const result = processFormattingTags('{@spell eldritch blast|PHB}')
      expect(result).toContain('data-ref-source="PHB"')
    })

    it('converts creature references', () => {
      const result = processFormattingTags('{@creature goblin}')
      expect(result).toContain('class="cross-ref-link creature-ref"')
      expect(result).toContain('data-ref-type="creature"')
      expect(result).toContain('data-ref-name="goblin"')
      expect(result).toContain('data-ref-source="MM"')
    })

    it('converts creature references with display text', () => {
      const result = processFormattingTags('{@creature goblin|MM|a sneaky goblin}')
      expect(result).toContain('a sneaky goblin</a>')
    })

    it('converts item references', () => {
      const result = processFormattingTags('{@item longsword}')
      expect(result).toContain('class="cross-ref-link item-ref"')
      expect(result).toContain('data-ref-type="item"')
      expect(result).toContain('data-ref-name="longsword"')
    })

    it('converts condition references', () => {
      const result = processFormattingTags('{@condition poisoned}')
      expect(result).toContain('class="cross-ref-link condition-ref"')
      expect(result).toContain('data-ref-type="condition"')
      expect(result).toContain('poisoned</a>')
    })

    it('converts class references', () => {
      const result = processFormattingTags('{@class Fighter}')
      expect(result).toContain('class="cross-ref-link class-ref"')
      expect(result).toContain('data-ref-type="class"')
    })

    it('converts feat references', () => {
      const result = processFormattingTags('{@feat Alert}')
      expect(result).toContain('class="cross-ref-link feat-ref"')
      expect(result).toContain('data-ref-type="feat"')
    })

    it('converts race references', () => {
      const result = processFormattingTags('{@race Elf}')
      expect(result).toContain('class="cross-ref-link race-ref"')
      expect(result).toContain('data-ref-type="race"')
    })

    it('converts background references', () => {
      const result = processFormattingTags('{@background Acolyte}')
      expect(result).toContain('class="cross-ref-link background-ref"')
      expect(result).toContain('data-ref-type="background"')
    })

    it('converts action references', () => {
      const result = processFormattingTags('{@action Dodge}')
      expect(result).toContain('class="cross-ref-link action-ref"')
      expect(result).toContain('data-ref-type="action"')
    })
  })

  describe('skills and abilities', () => {
    it('converts skill tags', () => {
      const result = processFormattingTags('{@skill Perception}')
      expect(result).toContain('class="skill"')
      expect(result).toContain('Perception')
    })

    it('converts sense tags', () => {
      const result = processFormattingTags('{@sense darkvision}')
      expect(result).toContain('class="sense"')
      expect(result).toContain('darkvision')
    })
  })

  describe('DC checks', () => {
    it('converts DC tags', () => {
      const result = processFormattingTags('{@dc 15}')
      expect(result).toContain('class="dc-check"')
      expect(result).toContain('DC 15')
    })

    it('converts DC tags with ability', () => {
      const result = processFormattingTags('{@dc 15|Dexterity}')
      expect(result).toContain('DC 15 Dexterity')
    })
  })

  describe('attack types', () => {
    it('converts melee weapon attack', () => {
      expect(processFormattingTags('{@atk mw}')).toContain('Melee Weapon Attack:')
    })

    it('converts ranged weapon attack', () => {
      expect(processFormattingTags('{@atk rw}')).toContain('Ranged Weapon Attack:')
    })

    it('converts melee spell attack', () => {
      expect(processFormattingTags('{@atk ms}')).toContain('Melee Spell Attack:')
    })

    it('converts ranged spell attack', () => {
      expect(processFormattingTags('{@atk rs}')).toContain('Ranged Spell Attack:')
    })

    it('converts hit tag', () => {
      expect(processFormattingTags('{@h}')).toContain('Hit:')
    })
  })

  describe('special tags', () => {
    it('converts recharge tags', () => {
      const result = processFormattingTags('{@recharge 5}')
      expect(result).toContain('class="recharge"')
      expect(result).toContain('Recharge 5–6')
    })

    it('converts recharge without number', () => {
      const result = processFormattingTags('{@recharge}')
      expect(result).toContain('(Recharge)')
    })

    it('converts chance tags', () => {
      const result = processFormattingTags('{@chance 50}')
      expect(result).toContain('class="chance"')
      expect(result).toContain('50% chance')
    })

    it('converts note tags', () => {
      const result = processFormattingTags('{@note important}')
      expect(result).toContain('class="note"')
      expect(result).toContain('Note: important')
    })
  })

  describe('edge cases', () => {
    it('returns empty string for null input', () => {
      expect(processFormattingTags(null)).toBe('')
    })

    it('returns empty string for undefined input', () => {
      expect(processFormattingTags(undefined)).toBe('')
    })

    it('returns empty string for empty string', () => {
      expect(processFormattingTags('')).toBe('')
    })

    it('passes through plain text unchanged', () => {
      expect(processFormattingTags('Hello world')).toBe('Hello world')
    })

    it('handles unknown tags via catch-all', () => {
      const result = processFormattingTags('{@unknown something}')
      expect(result).toContain('something')
    })
  })
})

describe('formatEntries', () => {
  it('formats string entries as paragraphs', () => {
    const result = formatEntries(['Hello world'])
    expect(result).toContain('<p>Hello world</p>')
  })

  it('formats nested entries with headers', () => {
    const result = formatEntries([
      { type: 'entries', name: 'My Section', entries: ['Content here'] },
    ])
    expect(result).toContain('<h5>My Section</h5>')
    expect(result).toContain('<p>Content here</p>')
  })

  it('formats list entries', () => {
    const result = formatEntries([
      { type: 'list', items: ['Item 1', 'Item 2'] },
    ])
    expect(result).toContain('<ul>')
    expect(result).toContain('<li>Item 1</li>')
    expect(result).toContain('<li>Item 2</li>')
  })

  it('formats table entries', () => {
    const result = formatEntries([
      {
        type: 'table',
        colLabels: ['Name', 'Value'],
        rows: [['Sword', '10 gp']],
      },
    ])
    expect(result).toContain('<table')
    expect(result).toContain('<th>Name</th>')
    expect(result).toContain('<td>Sword</td>')
  })

  it('formats inset entries', () => {
    const result = formatEntries([
      { type: 'inset', name: 'Sidebar', entries: ['Extra info'] },
    ])
    expect(result).toContain('class="inset-readaloud"')
    expect(result).toContain('<h5>Sidebar</h5>')
  })

  it('formats quote entries', () => {
    const result = formatEntries([
      { type: 'quote', entries: ['Wise words'], by: 'Gandalf' },
    ])
    expect(result).toContain('<blockquote>')
    expect(result).toContain('Wise words')
    expect(result).toContain('— Gandalf')
  })

  it('formats section entries with h4', () => {
    const result = formatEntries([
      { type: 'section', name: 'Chapter 1', entries: ['Text'] },
    ])
    expect(result).toContain('<h4>Chapter 1</h4>')
  })

  it('processes 5etools tags within entries', () => {
    const result = formatEntries(['Cast {@spell fireball} for {@damage 8d6} damage'])
    expect(result).toContain('class="cross-ref-link spell-ref"')
    expect(result).toContain('class="damage-roll"')
  })

  it('returns empty string for null input', () => {
    expect(formatEntries(null as any)).toBe('')
  })

  it('returns empty string for non-array input', () => {
    expect(formatEntries('not an array' as any)).toBe('')
  })
})
