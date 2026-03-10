/**
 * Tests for useSpells composable
 *
 * Tests the spell catalog composable: search with filters,
 * filter transformation for backend compatibility, details fetching.
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
  expectCommandCalledWith,
} from '@tests/helpers/mockInvoke'
import { useSpells } from '@/features/sources/composables/catalog/useSpells'

// ─── Factories ──────────────────────────────────────────────────────────────

function makeSpell(overrides: Record<string, unknown> = {}) {
  return {
    name: 'Fireball',
    level: 3,
    school: 'V',
    source: 'PHB',
    concentration: false,
    ritual: false,
    casting_time: '1 action',
    range: '150 feet',
    components: 'V, S, M',
    classes: ['Sorcerer', 'Wizard'],
    description: 'A bright streak flashes...',
    ...overrides,
  }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('useSpells', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  it('searches spells with query filter', async () => {
    mockCommand('search_spells', [makeSpell()])

    const { searchSpells, spells } = useSpells()
    await searchSpells({ query: 'fire' })

    expect(spells.value).toHaveLength(1)
    expectCommandCalledWith('search_spells', {
      filter: expect.objectContaining({ name_contains: 'fire' }),
      limit: 10000,
      offset: 0,
    })
  })

  it('transforms level filter (single value)', async () => {
    mockCommand('search_spells', [])

    const { searchSpells } = useSpells()
    await searchSpells({ levels: [3] })

    expectCommandCalledWith('search_spells', {
      filter: expect.objectContaining({ level: 3 }),
      limit: 10000,
      offset: 0,
    })
  })

  it('transforms level filter (multiple values → null)', async () => {
    mockCommand('search_spells', [])

    const { searchSpells } = useSpells()
    await searchSpells({ levels: [1, 2, 3] })

    // Multiple levels → null (backend doesn't support multi-level filter)
    expectCommandCalledWith('search_spells', {
      filter: expect.objectContaining({ level: null }),
      limit: 10000,
      offset: 0,
    })
  })

  it('transforms school filter (single value)', async () => {
    mockCommand('search_spells', [])

    const { searchSpells } = useSpells()
    await searchSpells({ schools: ['V'] })

    expectCommandCalledWith('search_spells', {
      filter: expect.objectContaining({ school: 'V' }),
      limit: 10000,
      offset: 0,
    })
  })

  it('passes ritual flag', async () => {
    mockCommand('search_spells', [])

    const { searchSpells } = useSpells()
    await searchSpells({ ritual: true })

    expectCommandCalledWith('search_spells', {
      filter: expect.objectContaining({ ritual: true }),
      limit: 10000,
      offset: 0,
    })
  })

  it('passes concentration flag', async () => {
    mockCommand('search_spells', [])

    const { searchSpells } = useSpells()
    await searchSpells({ concentration: true })

    expectCommandCalledWith('search_spells', {
      filter: expect.objectContaining({ concentration: true }),
      limit: 10000,
      offset: 0,
    })
  })

  it('passes sources filter', async () => {
    mockCommand('search_spells', [])

    const { searchSpells } = useSpells()
    await searchSpells({ sources: ['PHB', 'XGE'] })

    expectCommandCalledWith('search_spells', {
      filter: expect.objectContaining({ sources: ['PHB', 'XGE'] }),
      limit: 10000,
      offset: 0,
    })
  })

  it('gets spell details', async () => {
    const fullSpell = { ...makeSpell(), entries: ['A bright streak...'], time: [{ number: 1, unit: 'action' }] }
    mockCommand('get_spell_by_name', fullSpell)

    const { getSpellDetails } = useSpells()
    const details = await getSpellDetails('Fireball', 'PHB')

    expect(details).not.toBeNull()
    expectCommandCalledWith('get_spell_by_name', { name: 'Fireball', source: 'PHB' })
  })
})
