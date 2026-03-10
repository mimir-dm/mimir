/**
 * Tests for homebrew CRUD flows and data conversion logic.
 *
 * Tests the formToDataJson and dataJsonToForm conversion logic from HomebrewTab.vue,
 * plus the service layer CRUD calls for items, monsters, and spells.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createHomebrewService } from '@/services/createHomebrewService'
import type { HomebrewBase } from '@/services/createHomebrewService'
import { dataEvents } from '@/utils/dataEvents'

vi.mock('@tauri-apps/api/core')

// ============================================================
// formToDataJson logic (mirrored from HomebrewTab.vue)
// ============================================================

interface ItemFormState {
  name: string
  item_type: string
  rarity: string
  description: string
  weight: number | null
  value: number | null
  reqAttune: boolean
  reqAttuneText: string
  weaponCategory: string
  dmg1: string
  dmgType: string
  dmg2: string
  range: string
  bonusWeapon: string
  properties: string[]
  ac: number | null
  bonusAc: string
  strength: number | null
  stealth: boolean
}

function emptyForm(): ItemFormState {
  return {
    name: '', item_type: '', rarity: '', description: '',
    weight: null, value: null, reqAttune: false, reqAttuneText: '',
    weaponCategory: '', dmg1: '', dmgType: '', dmg2: '', range: '', bonusWeapon: '', properties: [],
    ac: null, bonusAc: '', strength: null, stealth: false,
  }
}

/** Mirror of formToDataJson from HomebrewTab.vue */
function formToDataJson(form: ItemFormState): string {
  const d: Record<string, unknown> = {}

  if (form.description) {
    d.entries = [form.description]
  }
  if (form.weight) d.weight = form.weight
  if (form.value) d.value = Math.round(form.value * 100) // store as cp

  if (form.reqAttune) {
    d.reqAttune = form.reqAttuneText || true
  }

  if (form.item_type === 'weapon') {
    d.weapon = true
    const hasRange = !!form.range
    const hasAmmoProp = form.properties.includes('A')
    d.type = (hasRange && hasAmmoProp) ? 'R' : 'M'
    if (form.weaponCategory) d.weaponCategory = form.weaponCategory
    if (form.dmg1) d.dmg1 = form.dmg1
    if (form.dmgType) d.dmgType = form.dmgType
    if (form.dmg2) d.dmg2 = form.dmg2
    if (form.range) d.range = form.range
    if (form.bonusWeapon) d.bonusWeapon = form.bonusWeapon
    if (form.properties.length > 0) d.property = form.properties
  }

  if (form.item_type === 'armor') {
    d.armor = true
    d.type = 'LA'
    if (form.ac) d.ac = form.ac
    if (form.bonusAc) d.bonusAc = form.bonusAc
    if (form.strength) d.strength = form.strength
    if (form.stealth) d.stealth = true
  }

  const typeCodeMap: Record<string, string> = {
    'potion': 'P', 'ring': 'RG', 'rod': 'RD', 'wand': 'WD',
    'scroll': 'SC', 'staff': 'W', 'wondrous item': 'W',
  }
  if (form.item_type && typeCodeMap[form.item_type]) {
    d.type = typeCodeMap[form.item_type]
  }

  return JSON.stringify(d)
}

/** Mirror of dataJsonToForm from HomebrewTab.vue */
function dataJsonToForm(data: Record<string, unknown>): Partial<ItemFormState> {
  const result: Partial<ItemFormState> = {}

  if (Array.isArray(data.entries) && data.entries.length > 0) {
    result.description = data.entries
      .map((e: unknown) => {
        if (typeof e === 'string') return e
        if (typeof e === 'object' && e !== null) {
          const obj = e as Record<string, unknown>
          if (obj.type === 'entries' && Array.isArray(obj.entries)) {
            return (obj.entries as unknown[]).filter(s => typeof s === 'string').join('\n')
          }
          if (obj.type === 'list' && Array.isArray(obj.items)) {
            return (obj.items as unknown[]).filter(s => typeof s === 'string').join(', ')
          }
        }
        return ''
      })
      .filter(Boolean)
      .join('\n\n')
  }

  if (data.weight) result.weight = data.weight as number
  if (data.value) result.value = (data.value as number) / 100

  if (data.reqAttune) {
    result.reqAttune = true
    if (typeof data.reqAttune === 'string') result.reqAttuneText = data.reqAttune
  }

  if (data.weaponCategory) result.weaponCategory = data.weaponCategory as string
  if (data.dmg1) result.dmg1 = data.dmg1 as string
  if (data.dmgType) result.dmgType = data.dmgType as string
  if (data.dmg2) result.dmg2 = data.dmg2 as string
  if (data.range) result.range = data.range as string
  if (data.bonusWeapon) result.bonusWeapon = data.bonusWeapon as string
  if (Array.isArray(data.property)) {
    result.properties = (data.property as string[]).map(p => p.split('|')[0])
  }

  if (data.ac) result.ac = data.ac as number
  if (data.bonusAc) result.bonusAc = data.bonusAc as string

  return result
}

// ============================================================
// formToDataJson tests
// ============================================================

describe('formToDataJson - Item data conversion', () => {
  it('produces empty object for empty form', () => {
    const form = emptyForm()
    const result = JSON.parse(formToDataJson(form))
    expect(result).toEqual({})
  })

  it('includes description as entries array', () => {
    const form = emptyForm()
    form.description = 'A magical sword that glows blue.'
    const result = JSON.parse(formToDataJson(form))
    expect(result.entries).toEqual(['A magical sword that glows blue.'])
  })

  it('converts value from gp to cp', () => {
    const form = emptyForm()
    form.value = 50 // 50 gp
    const result = JSON.parse(formToDataJson(form))
    expect(result.value).toBe(5000) // 5000 cp
  })

  it('stores weight as-is', () => {
    const form = emptyForm()
    form.weight = 3
    const result = JSON.parse(formToDataJson(form))
    expect(result.weight).toBe(3)
  })

  it('stores attunement as true when no text', () => {
    const form = emptyForm()
    form.reqAttune = true
    const result = JSON.parse(formToDataJson(form))
    expect(result.reqAttune).toBe(true)
  })

  it('stores attunement text when provided', () => {
    const form = emptyForm()
    form.reqAttune = true
    form.reqAttuneText = 'by a spellcaster'
    const result = JSON.parse(formToDataJson(form))
    expect(result.reqAttune).toBe('by a spellcaster')
  })

  describe('weapon type code', () => {
    it('sets melee weapon type M for weapon without range', () => {
      const form = emptyForm()
      form.item_type = 'weapon'
      form.dmg1 = '1d8'
      form.dmgType = 'S'
      const result = JSON.parse(formToDataJson(form))
      expect(result.type).toBe('M')
      expect(result.weapon).toBe(true)
      expect(result.dmg1).toBe('1d8')
      expect(result.dmgType).toBe('S')
    })

    it('sets melee weapon type M for weapon with range but no ammunition', () => {
      const form = emptyForm()
      form.item_type = 'weapon'
      form.range = '20/60'
      form.properties = ['T'] // thrown, not ammunition
      const result = JSON.parse(formToDataJson(form))
      expect(result.type).toBe('M') // thrown melee weapon
    })

    it('sets ranged weapon type R for weapon with range and ammunition', () => {
      const form = emptyForm()
      form.item_type = 'weapon'
      form.range = '150/600'
      form.properties = ['A', 'H', '2H'] // ammunition, heavy, two-handed
      const result = JSON.parse(formToDataJson(form))
      expect(result.type).toBe('R')
      expect(result.range).toBe('150/600')
      expect(result.property).toEqual(['A', 'H', '2H'])
    })

    it('includes all weapon fields', () => {
      const form = emptyForm()
      form.item_type = 'weapon'
      form.weaponCategory = 'martial'
      form.dmg1 = '1d8'
      form.dmgType = 'S'
      form.dmg2 = '1d10'
      form.bonusWeapon = '+1'
      form.properties = ['V']
      const result = JSON.parse(formToDataJson(form))
      expect(result.weaponCategory).toBe('martial')
      expect(result.dmg1).toBe('1d8')
      expect(result.dmg2).toBe('1d10')
      expect(result.bonusWeapon).toBe('+1')
      expect(result.property).toEqual(['V'])
    })
  })

  describe('armor type code', () => {
    it('sets armor type LA and armor flag', () => {
      const form = emptyForm()
      form.item_type = 'armor'
      form.ac = 14
      const result = JSON.parse(formToDataJson(form))
      expect(result.type).toBe('LA')
      expect(result.armor).toBe(true)
      expect(result.ac).toBe(14)
    })

    it('includes all armor fields', () => {
      const form = emptyForm()
      form.item_type = 'armor'
      form.ac = 16
      form.bonusAc = '+1'
      form.strength = 13
      form.stealth = true
      const result = JSON.parse(formToDataJson(form))
      expect(result.ac).toBe(16)
      expect(result.bonusAc).toBe('+1')
      expect(result.strength).toBe(13)
      expect(result.stealth).toBe(true)
    })

    it('does not include stealth when false', () => {
      const form = emptyForm()
      form.item_type = 'armor'
      form.ac = 14
      form.stealth = false
      const result = JSON.parse(formToDataJson(form))
      expect(result.stealth).toBeUndefined()
    })
  })

  describe('other item type codes', () => {
    const typeCodeCases: [string, string][] = [
      ['potion', 'P'],
      ['ring', 'RG'],
      ['rod', 'RD'],
      ['wand', 'WD'],
      ['scroll', 'SC'],
      ['staff', 'W'],
      ['wondrous item', 'W'],
    ]

    it.each(typeCodeCases)('maps "%s" to 5etools code "%s"', (itemType, expected) => {
      const form = emptyForm()
      form.item_type = itemType
      const result = JSON.parse(formToDataJson(form))
      expect(result.type).toBe(expected)
    })
  })

  it('does not set type for unknown item types', () => {
    const form = emptyForm()
    form.item_type = 'custom_thing'
    const result = JSON.parse(formToDataJson(form))
    expect(result.type).toBeUndefined()
  })

  it('rounds value correctly for fractional gp', () => {
    const form = emptyForm()
    form.value = 0.5 // 5 sp = 0.5 gp
    const result = JSON.parse(formToDataJson(form))
    expect(result.value).toBe(50) // 50 cp
  })
})

// ============================================================
// dataJsonToForm tests
// ============================================================

describe('dataJsonToForm - JSON to form conversion', () => {
  it('extracts string entries as description', () => {
    const data = { entries: ['A magical sword that glows.'] }
    const result = dataJsonToForm(data)
    expect(result.description).toBe('A magical sword that glows.')
  })

  it('joins multiple entries with double newline', () => {
    const data = { entries: ['Line one.', 'Line two.'] }
    const result = dataJsonToForm(data)
    expect(result.description).toBe('Line one.\n\nLine two.')
  })

  it('handles nested entries objects', () => {
    const data = {
      entries: [
        { type: 'entries', entries: ['Nested line 1.', 'Nested line 2.'] },
      ],
    }
    const result = dataJsonToForm(data)
    expect(result.description).toBe('Nested line 1.\nNested line 2.')
  })

  it('handles list items', () => {
    const data = {
      entries: [
        { type: 'list', items: ['Item A', 'Item B'] },
      ],
    }
    const result = dataJsonToForm(data)
    expect(result.description).toBe('Item A, Item B')
  })

  it('converts value from cp to gp', () => {
    const data = { value: 5000 }
    const result = dataJsonToForm(data)
    expect(result.value).toBe(50) // 50 gp
  })

  it('extracts weight', () => {
    const data = { weight: 3 }
    const result = dataJsonToForm(data)
    expect(result.weight).toBe(3)
  })

  it('extracts attunement boolean', () => {
    const data = { reqAttune: true }
    const result = dataJsonToForm(data)
    expect(result.reqAttune).toBe(true)
    expect(result.reqAttuneText).toBeUndefined()
  })

  it('extracts attunement string', () => {
    const data = { reqAttune: 'by a cleric' }
    const result = dataJsonToForm(data)
    expect(result.reqAttune).toBe(true)
    expect(result.reqAttuneText).toBe('by a cleric')
  })

  it('extracts weapon fields', () => {
    const data = {
      weaponCategory: 'martial',
      dmg1: '2d6',
      dmgType: 'S',
      dmg2: '',
      range: '5',
      bonusWeapon: '+2',
      property: ['2H', 'H'],
    }
    const result = dataJsonToForm(data)
    expect(result.weaponCategory).toBe('martial')
    expect(result.dmg1).toBe('2d6')
    expect(result.dmgType).toBe('S')
    expect(result.range).toBe('5')
    expect(result.bonusWeapon).toBe('+2')
    expect(result.properties).toEqual(['2H', 'H'])
  })

  it('strips pipe suffix from property codes', () => {
    const data = { property: ['V|PHB', 'F|PHB'] }
    const result = dataJsonToForm(data)
    expect(result.properties).toEqual(['V', 'F'])
  })

  it('extracts armor fields', () => {
    const data = { ac: 18, bonusAc: '+2' }
    const result = dataJsonToForm(data)
    expect(result.ac).toBe(18)
    expect(result.bonusAc).toBe('+2')
  })

  it('handles empty data', () => {
    const result = dataJsonToForm({})
    expect(result.description).toBeUndefined()
    expect(result.weight).toBeUndefined()
    expect(result.value).toBeUndefined()
  })
})

// ============================================================
// Homebrew Service CRUD integration tests
// ============================================================

interface HomebrewItem extends HomebrewBase {
  item_type: string | null
  rarity: string | null
}

interface CreateItem {
  campaign_id: string
  name: string
  item_type?: string
  rarity?: string
  data: string
}

interface UpdateItem {
  name?: string
  item_type?: string | null
  rarity?: string | null
  data?: string
}

interface HomebrewMonster extends HomebrewBase {
  cr: string | null
  creature_type: string | null
  size: string | null
}

interface CreateMonster {
  campaign_id: string
  name: string
  data: string
  cr?: string
  creature_type?: string
  size?: string
}

interface UpdateMonster {
  name?: string
  data?: string
  cr?: string | null
  creature_type?: string | null
  size?: string | null
}

interface HomebrewSpell extends HomebrewBase {
  level: number | null
  school: string | null
}

interface CreateSpell {
  campaign_id: string
  name: string
  data: string
  level?: number
  school?: string
}

interface UpdateSpell {
  name?: string
  data?: string
  level?: number | null
  school?: string | null
}

describe('Homebrew Item CRUD via service', () => {
  let invoke: ReturnType<typeof vi.fn>
  let itemService: ReturnType<typeof createHomebrewService<HomebrewItem, CreateItem, UpdateItem>>

  const mockItem: HomebrewItem = {
    id: 'item-1',
    campaign_id: 'camp-1',
    name: 'Flame Tongue',
    data: '{"type":"M","weapon":true,"dmg1":"1d8","dmgType":"S"}',
    item_type: 'weapon',
    rarity: 'rare',
    cloned_from_name: null,
    cloned_from_source: null,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  }

  beforeEach(async () => {
    vi.clearAllMocks()
    dataEvents.clear()
    const { invoke: mockInvoke } = await import('@tauri-apps/api/core')
    invoke = mockInvoke as any
    itemService = createHomebrewService<HomebrewItem, CreateItem, UpdateItem>({
      commandSuffix: 'item',
      eventPrefix: 'homebrew-item',
      label: 'item',
    })
  })

  it('creates item with correct invoke command and payload', async () => {
    invoke.mockResolvedValueOnce({ success: true, data: mockItem })
    const payload: CreateItem = {
      campaign_id: 'camp-1',
      name: 'Flame Tongue',
      item_type: 'weapon',
      rarity: 'rare',
      data: formToDataJson({
        ...emptyForm(),
        item_type: 'weapon',
        dmg1: '1d8',
        dmgType: 'S',
      }),
    }
    await itemService.create(payload)
    expect(invoke).toHaveBeenCalledWith('create_homebrew_item', { input: payload })
  })

  it('updates item with correct command', async () => {
    invoke.mockResolvedValueOnce({ success: true, data: mockItem })
    const update: UpdateItem = { name: 'Flame Tongue +1', rarity: 'very rare' }
    await itemService.update('item-1', update)
    expect(invoke).toHaveBeenCalledWith('update_homebrew_item', { id: 'item-1', input: update })
  })

  it('deletes item with correct command', async () => {
    invoke.mockResolvedValueOnce({ success: true, data: null })
    await itemService.delete('item-1')
    expect(invoke).toHaveBeenCalledWith('delete_homebrew_item', { id: 'item-1' })
  })

  it('lists items for campaign', async () => {
    invoke.mockResolvedValueOnce({ success: true, data: [mockItem] })
    const result = await itemService.list('camp-1')
    expect(invoke).toHaveBeenCalledWith('list_homebrew_items', { campaignId: 'camp-1' })
    expect(result).toHaveLength(1)
    expect(result[0].name).toBe('Flame Tongue')
  })

  it('emits created event on create', async () => {
    invoke.mockResolvedValueOnce({ success: true, data: mockItem })
    const handler = vi.fn()
    dataEvents.on('homebrew-item:created', handler)
    await itemService.create({
      campaign_id: 'camp-1', name: 'Test', data: '{}',
    })
    expect(handler).toHaveBeenCalled()
  })

  it('emits deleted event on delete', async () => {
    invoke.mockResolvedValueOnce({ success: true, data: null })
    const handler = vi.fn()
    dataEvents.on('homebrew-item:deleted', handler)
    await itemService.delete('item-1')
    expect(handler).toHaveBeenCalled()
  })
})

describe('Homebrew Monster CRUD via service', () => {
  let invoke: ReturnType<typeof vi.fn>
  let monsterService: ReturnType<typeof createHomebrewService<HomebrewMonster, CreateMonster, UpdateMonster>>

  const mockMonster: HomebrewMonster = {
    id: 'mon-1',
    campaign_id: 'camp-1',
    name: 'Frost Colossus',
    data: '{"name":"Frost Colossus","size":["G"],"type":"elemental","cr":"20"}',
    cr: '20',
    creature_type: 'elemental',
    size: 'G',
    cloned_from_name: null,
    cloned_from_source: null,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  }

  beforeEach(async () => {
    vi.clearAllMocks()
    dataEvents.clear()
    const { invoke: mockInvoke } = await import('@tauri-apps/api/core')
    invoke = mockInvoke as any
    monsterService = createHomebrewService<HomebrewMonster, CreateMonster, UpdateMonster>({
      commandSuffix: 'monster',
      eventPrefix: 'homebrew-monster',
      label: 'monster',
    })
  })

  it('creates monster with stat block JSON', async () => {
    invoke.mockResolvedValueOnce({ success: true, data: mockMonster })
    const payload: CreateMonster = {
      campaign_id: 'camp-1',
      name: 'Frost Colossus',
      data: JSON.stringify({ name: 'Frost Colossus', size: ['G'], type: 'elemental', cr: '20' }),
      cr: '20',
      creature_type: 'elemental',
      size: 'G',
    }
    await monsterService.create(payload)
    expect(invoke).toHaveBeenCalledWith('create_homebrew_monster', { input: payload })
  })

  it('updates monster name and data', async () => {
    invoke.mockResolvedValueOnce({ success: true, data: mockMonster })
    await monsterService.update('mon-1', {
      name: 'Frost Titan',
      cr: '22',
    })
    expect(invoke).toHaveBeenCalledWith('update_homebrew_monster', {
      id: 'mon-1',
      input: { name: 'Frost Titan', cr: '22' },
    })
  })

  it('deletes monster', async () => {
    invoke.mockResolvedValueOnce({ success: true, data: null })
    await monsterService.delete('mon-1')
    expect(invoke).toHaveBeenCalledWith('delete_homebrew_monster', { id: 'mon-1' })
  })

  it('emits created event on create', async () => {
    invoke.mockResolvedValueOnce({ success: true, data: mockMonster })
    const handler = vi.fn()
    dataEvents.on('homebrew-monster:created', handler)
    await monsterService.create({
      campaign_id: 'camp-1', name: 'Test', data: '{}',
    })
    expect(handler).toHaveBeenCalled()
  })
})

describe('Homebrew Spell CRUD via service', () => {
  let invoke: ReturnType<typeof vi.fn>
  let spellService: ReturnType<typeof createHomebrewService<HomebrewSpell, CreateSpell, UpdateSpell>>

  const mockSpell: HomebrewSpell = {
    id: 'spell-1',
    campaign_id: 'camp-1',
    name: 'Eldritch Storm',
    data: '{"name":"Eldritch Storm","level":5,"school":"V"}',
    level: 5,
    school: 'V',
    cloned_from_name: null,
    cloned_from_source: null,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  }

  beforeEach(async () => {
    vi.clearAllMocks()
    dataEvents.clear()
    const { invoke: mockInvoke } = await import('@tauri-apps/api/core')
    invoke = mockInvoke as any
    spellService = createHomebrewService<HomebrewSpell, CreateSpell, UpdateSpell>({
      commandSuffix: 'spell',
      eventPrefix: 'homebrew-spell',
      label: 'spell',
    })
  })

  it('creates spell with level and school', async () => {
    invoke.mockResolvedValueOnce({ success: true, data: mockSpell })
    const payload: CreateSpell = {
      campaign_id: 'camp-1',
      name: 'Eldritch Storm',
      data: JSON.stringify({ name: 'Eldritch Storm', level: 5, school: 'V' }),
      level: 5,
      school: 'V',
    }
    await spellService.create(payload)
    expect(invoke).toHaveBeenCalledWith('create_homebrew_spell', { input: payload })
  })

  it('updates spell', async () => {
    invoke.mockResolvedValueOnce({ success: true, data: mockSpell })
    await spellService.update('spell-1', { name: 'Greater Eldritch Storm', level: 7 })
    expect(invoke).toHaveBeenCalledWith('update_homebrew_spell', {
      id: 'spell-1',
      input: { name: 'Greater Eldritch Storm', level: 7 },
    })
  })

  it('deletes spell', async () => {
    invoke.mockResolvedValueOnce({ success: true, data: null })
    await spellService.delete('spell-1')
    expect(invoke).toHaveBeenCalledWith('delete_homebrew_spell', { id: 'spell-1' })
  })

  it('lists spells for campaign', async () => {
    invoke.mockResolvedValueOnce({ success: true, data: [mockSpell] })
    const result = await spellService.list('camp-1')
    expect(invoke).toHaveBeenCalledWith('list_homebrew_spells', { campaignId: 'camp-1' })
    expect(result).toHaveLength(1)
    expect(result[0].name).toBe('Eldritch Storm')
  })

  it('emits events on CRUD operations', async () => {
    const created = vi.fn()
    const updated = vi.fn()
    const deleted = vi.fn()
    dataEvents.on('homebrew-spell:created', created)
    dataEvents.on('homebrew-spell:updated', updated)
    dataEvents.on('homebrew-spell:deleted', deleted)

    invoke.mockResolvedValueOnce({ success: true, data: mockSpell })
    await spellService.create({ campaign_id: 'camp-1', name: 'Test', data: '{}' })
    expect(created).toHaveBeenCalled()

    invoke.mockResolvedValueOnce({ success: true, data: mockSpell })
    await spellService.update('spell-1', { name: 'Updated' })
    expect(updated).toHaveBeenCalled()

    invoke.mockResolvedValueOnce({ success: true, data: null })
    await spellService.delete('spell-1')
    expect(deleted).toHaveBeenCalled()
  })
})

// ============================================================
// Form validation edge cases
// ============================================================

describe('Form validation and edge cases', () => {
  it('formToDataJson does not include null/empty fields', () => {
    const form = emptyForm()
    form.item_type = 'weapon'
    // Only set type, no other weapon fields
    const result = JSON.parse(formToDataJson(form))
    expect(result.weapon).toBe(true)
    expect(result.type).toBe('M')
    expect(result.dmg1).toBeUndefined()
    expect(result.dmgType).toBeUndefined()
    expect(result.range).toBeUndefined()
    expect(result.property).toBeUndefined()
  })

  it('weapon type code overrides other type codes', () => {
    // If item_type is 'weapon', the type code should be M or R, not a mapped code
    const form = emptyForm()
    form.item_type = 'weapon'
    const result = JSON.parse(formToDataJson(form))
    // 'weapon' is not in typeCodeMap, so type stays as M from weapon logic
    expect(result.type).toBe('M')
  })

  it('armor overrides mapped type codes', () => {
    const form = emptyForm()
    form.item_type = 'armor'
    const result = JSON.parse(formToDataJson(form))
    // 'armor' is not in typeCodeMap, so type stays as LA from armor logic
    expect(result.type).toBe('LA')
  })

  it('round-trips weapon data through formToDataJson and dataJsonToForm', () => {
    const form: ItemFormState = {
      ...emptyForm(),
      item_type: 'weapon',
      description: 'A flaming sword.',
      weight: 3,
      value: 500, // gp
      reqAttune: true,
      reqAttuneText: 'by a fighter',
      weaponCategory: 'martial',
      dmg1: '1d8',
      dmgType: 'S',
      dmg2: '1d10',
      range: '',
      bonusWeapon: '+1',
      properties: ['V'],
    }

    const json = formToDataJson(form)
    const parsed = JSON.parse(json)
    const restored = dataJsonToForm(parsed)

    expect(restored.description).toBe('A flaming sword.')
    expect(restored.weight).toBe(3)
    expect(restored.value).toBe(500) // round-trip gp->cp->gp
    expect(restored.reqAttune).toBe(true)
    expect(restored.reqAttuneText).toBe('by a fighter')
    expect(restored.weaponCategory).toBe('martial')
    expect(restored.dmg1).toBe('1d8')
    expect(restored.dmgType).toBe('S')
    expect(restored.dmg2).toBe('1d10')
    expect(restored.bonusWeapon).toBe('+1')
    expect(restored.properties).toEqual(['V'])
  })

  it('round-trips armor data', () => {
    const form: ItemFormState = {
      ...emptyForm(),
      item_type: 'armor',
      ac: 16,
      bonusAc: '+1',
      strength: 13,
      stealth: true,
    }

    const json = formToDataJson(form)
    const parsed = JSON.parse(json)
    const restored = dataJsonToForm(parsed)

    expect(restored.ac).toBe(16)
    expect(restored.bonusAc).toBe('+1')
  })

  it('handles monster JSON validation (valid JSON)', () => {
    const validJson = '{"name":"Test","size":["M"]}'
    expect(() => JSON.parse(validJson)).not.toThrow()
  })

  it('handles monster JSON validation (invalid JSON)', () => {
    const invalidJson = '{"name": broken}'
    expect(() => JSON.parse(invalidJson)).toThrow()
  })

  it('handles spell JSON validation', () => {
    const spellJson = '{"name":"Fireball","level":3,"school":"V"}'
    const parsed = JSON.parse(spellJson)
    expect(parsed.name).toBe('Fireball')
    expect(parsed.level).toBe(3)
    expect(parsed.school).toBe('V')
  })
})
