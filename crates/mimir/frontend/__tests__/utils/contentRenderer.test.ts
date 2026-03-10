/**
 * Tests for contentRenderer.ts
 *
 * Tests the content rendering utility that converts BookSection
 * structures into HTML for display.
 */

import { describe, it, expect } from 'vitest'
import { renderSection } from '@/features/sources/utils/renderers/contentRenderer'

describe('renderSection', () => {
  it('returns empty string for null input', () => {
    expect(renderSection(null as any)).toBe('')
  })

  it('renders section name as h1', () => {
    const section = {
      name: 'Chapter 1',
      entries: ['Hello world'],
    }
    const result = renderSection(section as any)
    expect(result).toContain('<h1>Chapter 1</h1>')
  })

  it('renders string entries as paragraphs', () => {
    const section = {
      entries: ['First paragraph', 'Second paragraph'],
    }
    const result = renderSection(section as any)
    expect(result).toContain('<p>First paragraph</p>')
    expect(result).toContain('<p>Second paragraph</p>')
  })

  it('wraps content in section-content div', () => {
    const section = { entries: ['Content'] }
    const result = renderSection(section as any)
    expect(result).toContain('class="section-content"')
  })
})

describe('renderEntry (via renderSection)', () => {
  function render(entries: any[]) {
    return renderSection({ entries } as any)
  }

  describe('section entries', () => {
    it('renders section type with heading', () => {
      const result = render([
        { type: 'section', name: 'My Section', entries: ['Text'] },
      ])
      expect(result).toContain('class="section"')
      expect(result).toContain('My Section')
    })

    it('uses depth-based header levels', () => {
      // At depth 0, section gets h2
      const result = render([
        { type: 'section', name: 'Level 1', entries: [] },
      ])
      expect(result).toContain('<h2>')
    })
  })

  describe('entries type', () => {
    it('renders entries with name as heading', () => {
      const result = render([
        { type: 'entries', name: 'Subsection', entries: ['Content'] },
      ])
      expect(result).toContain('Subsection')
      expect(result).toContain('<p>Content</p>')
    })

    it('renders entries without name', () => {
      const result = render([
        { type: 'entries', entries: ['Just content'] },
      ])
      expect(result).toContain('<p>Just content</p>')
    })
  })

  describe('inset entries', () => {
    it('renders insetReadaloud with correct class', () => {
      const result = render([
        { type: 'insetReadaloud', entries: ['Read this aloud'] },
      ])
      expect(result).toContain('class="inset-readaloud"')
      expect(result).toContain('Read this aloud')
    })

    it('renders generic inset with correct class', () => {
      const result = render([
        { type: 'inset', name: 'Sidebar', entries: ['Extra info'] },
      ])
      expect(result).toContain('class="inset"')
      expect(result).toContain('Sidebar')
    })
  })

  describe('list entries', () => {
    it('renders lists with ul and li', () => {
      const result = render([
        { type: 'list', items: ['Item 1', 'Item 2', 'Item 3'] },
      ])
      expect(result).toContain('<ul class="content-list">')
      expect(result).toContain('<li>Item 1</li>')
      expect(result).toContain('<li>Item 2</li>')
      expect(result).toContain('<li>Item 3</li>')
    })
  })

  describe('table entries', () => {
    it('renders tables with headers and rows', () => {
      const result = render([
        {
          type: 'table',
          colLabels: ['Name', 'CR'],
          rows: [['Goblin', '1/4'], ['Dragon', '17']],
        },
      ])
      expect(result).toContain('<table')
      expect(result).toContain('<th>Name</th>')
      expect(result).toContain('<th>CR</th>')
      expect(result).toContain('<td>Goblin</td>')
      expect(result).toContain('<td>Dragon</td>')
    })

    it('renders table captions', () => {
      const result = render([
        {
          type: 'table',
          caption: 'Monster Table',
          colLabels: ['Name'],
          rows: [['Goblin']],
        },
      ])
      expect(result).toContain('<caption>Monster Table</caption>')
    })

    it('renders dice roll cells', () => {
      const result = render([
        {
          type: 'table',
          colLabels: ['d6', 'Result'],
          rows: [[{ roll: { min: 1, max: 2 } }, 'Nothing']],
        },
      ])
      expect(result).toContain('1-2')
    })
  })

  describe('quote entries', () => {
    it('renders blockquotes with attribution', () => {
      const result = render([
        { type: 'quote', entries: ['Wise words'], by: 'Elminster' },
      ])
      expect(result).toContain('<blockquote>')
      expect(result).toContain('Wise words')
      expect(result).toContain('— Elminster')
    })
  })

  describe('image entries', () => {
    it('renders image placeholders', () => {
      const result = render([
        { type: 'image', href: { path: 'img/map.png' }, title: 'Map' },
      ])
      expect(result).toContain('class="image-container"')
      expect(result).toContain('data-image-path="img/map.png"')
      expect(result).toContain('Map')
    })
  })

  describe('item entries', () => {
    it('renders item name/entry pairs', () => {
      const result = render([
        { type: 'item', name: 'Str', entry: '18 (+4)' },
      ])
      expect(result).toContain('<strong>Str:</strong>')
      expect(result).toContain('18 (+4)')
    })
  })

  describe('statblock entries', () => {
    it('renders creature statblock references as links', () => {
      const result = render([
        { type: 'statblock', name: 'Goblin', source: 'MM', tag: 'creature' },
      ])
      expect(result).toContain('class="cross-ref-link creature-ref"')
      expect(result).toContain('Goblin')
    })

    it('renders non-creature statblocks as text', () => {
      const result = render([
        { type: 'statblock', name: 'Longsword', source: 'PHB', tag: 'item' },
      ])
      expect(result).toContain('See: Longsword')
      expect(result).not.toContain('cross-ref-link')
    })
  })

  describe('5etools tag processing', () => {
    it('processes formatting tags in string entries', () => {
      const result = render([
        'The {@creature goblin} attacks with {@dice 1d6} damage',
      ])
      expect(result).toContain('class="cross-ref-link creature-ref"')
      expect(result).toContain('class="dice-roll"')
    })
  })
})
