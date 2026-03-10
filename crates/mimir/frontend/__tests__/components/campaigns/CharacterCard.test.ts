/**
 * Tests for CharacterCard.vue
 *
 * Pure props-driven component — no API calls needed.
 */

import { describe, it, expect } from 'vitest'
import { shallowMountWithPlugins } from '@tests/helpers/mountHelpers'
import CharacterCard from '@/components/characters/CharacterCard.vue'

// ─── Factories ──────────────────────────────────────────────────────────────

function makeCharacter(overrides: Record<string, unknown> = {}) {
  return {
    id: 'char-1',
    campaign_id: 'camp-1',
    name: 'Thorin Ironforge',
    is_npc: 0,
    race_name: 'Mountain Dwarf',
    player_name: 'Alice',
    background_name: 'Soldier',
    classes: [
      { class_name: 'Fighter', class_source: 'PHB', level: 5, subclass_name: 'Champion' },
    ],
    ...overrides,
  }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('CharacterCard', () => {
  describe('rendering', () => {
    it('displays character name', () => {
      const wrapper = shallowMountWithPlugins(CharacterCard, {
        props: { character: makeCharacter() },
      })
      expect(wrapper.text()).toContain('Thorin Ironforge')
    })

    it('displays race name', () => {
      const wrapper = shallowMountWithPlugins(CharacterCard, {
        props: { character: makeCharacter() },
      })
      expect(wrapper.text()).toContain('Mountain Dwarf')
    })

    it('shows player name when showPlayer is true', () => {
      const wrapper = shallowMountWithPlugins(CharacterCard, {
        props: { character: makeCharacter(), showPlayer: true },
      })
      expect(wrapper.text()).toContain('Alice')
    })

    it('hides player name when showPlayer is false', () => {
      const wrapper = shallowMountWithPlugins(CharacterCard, {
        props: { character: makeCharacter(), showPlayer: false },
      })
      expect(wrapper.text()).not.toContain('Alice')
    })

    it('shows NPC badge for NPCs', () => {
      const wrapper = shallowMountWithPlugins(CharacterCard, {
        props: { character: makeCharacter({ is_npc: 1, name: 'Goblin King' }) },
      })
      expect(wrapper.text()).toContain('NPC')
    })

    it('does not show NPC badge for PCs', () => {
      const wrapper = shallowMountWithPlugins(CharacterCard, {
        props: { character: makeCharacter({ is_npc: 0 }) },
      })
      const badges = wrapper.findAll('.npc-badge')
      expect(badges.length).toBe(0)
    })

    it('handles character with no race', () => {
      const wrapper = shallowMountWithPlugins(CharacterCard, {
        props: { character: makeCharacter({ race_name: null }) },
      })
      expect(wrapper.text()).toContain('Thorin Ironforge')
    })

    it('handles character with no player name', () => {
      const wrapper = shallowMountWithPlugins(CharacterCard, {
        props: { character: makeCharacter({ player_name: null }), showPlayer: true },
      })
      expect(wrapper.text()).toContain('Thorin Ironforge')
      expect(wrapper.text()).not.toContain('null')
    })
  })

  describe('events', () => {
    it('emits click when card is clicked', async () => {
      const character = makeCharacter()
      const wrapper = shallowMountWithPlugins(CharacterCard, {
        props: { character },
      })
      await wrapper.find('.character-card').trigger('click')
      expect(wrapper.emitted('click')).toBeTruthy()
      expect(wrapper.emitted('click')![0]).toEqual([character])
    })

    it('emits view when view button is clicked', async () => {
      const character = makeCharacter()
      const wrapper = shallowMountWithPlugins(CharacterCard, {
        props: { character },
      })
      const viewBtn = wrapper.findAll('button').find(b => b.text().includes('View'))
      if (viewBtn) {
        await viewBtn.trigger('click')
        expect(wrapper.emitted('view')).toBeTruthy()
        expect(wrapper.emitted('view')![0]).toEqual([character])
      }
    })

    it('emits print when PDF button is clicked', async () => {
      const character = makeCharacter()
      const wrapper = shallowMountWithPlugins(CharacterCard, {
        props: { character },
      })
      const pdfBtn = wrapper.findAll('button').find(b => b.text().match(/PDF|Print/i))
      if (pdfBtn) {
        await pdfBtn.trigger('click')
        expect(wrapper.emitted('print')).toBeTruthy()
        expect(wrapper.emitted('print')![0]).toEqual([character])
      }
    })
  })
})
