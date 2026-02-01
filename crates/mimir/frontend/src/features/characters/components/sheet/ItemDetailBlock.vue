<template>
  <div class="item-detail-block">
    <!-- Type & Rarity row -->
    <div class="item-stat-header">
      <span v-if="itemTypeLabel" class="item-type-label">{{ itemTypeLabel }}</span>
      <span v-if="detail.rarity && detail.rarity !== 'none'" class="rarity-badge" :class="rarityClass">{{ detail.rarity }}</span>
    </div>

    <!-- Weapon stats -->
    <div v-if="isWeapon" class="item-stat-grid">
      <div class="stat-cell" v-if="data.dmg1">
        <span class="stat-label">Damage</span>
        <span class="stat-value">{{ data.dmg1 }}{{ dmgTypeLabel ? ` ${dmgTypeLabel}` : '' }}</span>
      </div>
      <div class="stat-cell" v-if="data.dmg2">
        <span class="stat-label">Versatile</span>
        <span class="stat-value">{{ data.dmg2 }}</span>
      </div>
      <div class="stat-cell" v-if="data.range">
        <span class="stat-label">Range</span>
        <span class="stat-value">{{ data.range }}</span>
      </div>
      <div class="stat-cell" v-if="data.weight">
        <span class="stat-label">Weight</span>
        <span class="stat-value">{{ data.weight }} lb.</span>
      </div>
      <div class="stat-cell" v-if="bonusLabel">
        <span class="stat-label">Bonus</span>
        <span class="stat-value">{{ bonusLabel }}</span>
      </div>
    </div>

    <!-- Armor stats -->
    <div v-if="isArmor" class="item-stat-grid">
      <div class="stat-cell" v-if="data.ac">
        <span class="stat-label">AC</span>
        <span class="stat-value">{{ data.ac }}</span>
      </div>
      <div class="stat-cell" v-if="data.weight">
        <span class="stat-label">Weight</span>
        <span class="stat-value">{{ data.weight }} lb.</span>
      </div>
      <div class="stat-cell" v-if="data.strength">
        <span class="stat-label">Str Required</span>
        <span class="stat-value">{{ data.strength }}</span>
      </div>
      <div class="stat-cell" v-if="data.stealth">
        <span class="stat-label">Stealth</span>
        <span class="stat-value disadvantage">Disadvantage</span>
      </div>
      <div class="stat-cell" v-if="bonusLabel">
        <span class="stat-label">Bonus</span>
        <span class="stat-value">{{ bonusLabel }}</span>
      </div>
    </div>

    <!-- Generic item weight/value for non-weapon, non-armor -->
    <div v-if="!isWeapon && !isArmor && (data.weight || data.value)" class="item-stat-grid">
      <div class="stat-cell" v-if="data.weight">
        <span class="stat-label">Weight</span>
        <span class="stat-value">{{ data.weight }} lb.</span>
      </div>
      <div class="stat-cell" v-if="data.value">
        <span class="stat-label">Value</span>
        <span class="stat-value">{{ formatValue(data.value as number) }}</span>
      </div>
    </div>

    <!-- Properties -->
    <div v-if="properties.length > 0" class="item-properties">
      <span class="prop-tag" v-for="prop in properties" :key="prop">{{ prop }}</span>
    </div>

    <!-- Description / Entries -->
    <div v-if="description" class="item-description">
      {{ description }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface ItemDetail {
  name: string
  source: string
  item_type: string | null
  rarity: string | null
  data: Record<string, unknown>
  fluff: string | null
}

const props = defineProps<{
  detail: ItemDetail
}>()

const data = computed(() => props.detail.data || {})

const isWeapon = computed(() => {
  const d = data.value
  return d.weapon === true || d.dmg1 || d.weaponCategory
})

const isArmor = computed(() => {
  const d = data.value
  return d.armor === true || d.ac
})

const dmgTypeMap: Record<string, string> = {
  A: 'Acid', B: 'Bludgeoning', C: 'Cold', F: 'Fire', O: 'Force',
  L: 'Lightning', N: 'Necrotic', P: 'Piercing', I: 'Poison',
  Y: 'Psychic', R: 'Radiant', S: 'Slashing', T: 'Thunder',
}

const dmgTypeLabel = computed(() => {
  const t = data.value.dmgType as string | undefined
  return t ? dmgTypeMap[t] || t : ''
})

const itemTypeMap: Record<string, string> = {
  S: 'Shield', M: 'Melee Weapon', R: 'Ranged Weapon', A: 'Ammunition',
  LA: 'Light Armor', MA: 'Medium Armor', HA: 'Heavy Armor',
  P: 'Potion', RD: 'Rod', RG: 'Ring', SC: 'Scroll', WD: 'Wand',
  W: 'Wondrous Item', G: 'Adventuring Gear', '$': 'Trade Good',
  AT: "Artisan's Tools", GS: 'Gaming Set', INS: 'Instrument',
  MNT: 'Mount', TAH: 'Tack and Harness', TG: 'Trade Good',
  SCF: 'Spellcasting Focus', EXP: 'Explosive', AF: 'Ammunition (Firearm)',
}

const itemTypeLabel = computed(() => {
  const t = props.detail.item_type || (data.value.type as string | undefined)
  if (!t) return ''
  const clean = t.split('|')[0]
  return itemTypeMap[clean] || ''
})

const bonusLabel = computed(() => {
  const d = data.value
  if (d.bonusWeapon) return `${d.bonusWeapon} to attack & damage`
  if (d.bonusAc) return `${d.bonusAc} to AC`
  if (d.bonusSpellAttack) return `${d.bonusSpellAttack} to spell attacks`
  if (d.bonusSavingThrow) return `${d.bonusSavingThrow} to saves`
  return ''
})

const rarityClass = computed(() => {
  const r = props.detail.rarity?.toLowerCase() || ''
  if (r.includes('uncommon')) return 'uncommon'
  if (r.includes('very rare')) return 'very-rare'
  if (r.includes('rare')) return 'rare'
  if (r.includes('legendary')) return 'legendary'
  if (r.includes('artifact')) return 'artifact'
  if (r.includes('common')) return 'common'
  return ''
})

const propCodeMap: Record<string, string> = {
  F: 'Finesse', H: 'Heavy', L: 'Light', R: 'Reach', T: 'Thrown',
  '2H': 'Two-Handed', V: 'Versatile', A: 'Ammunition', LD: 'Loading', S: 'Special',
}

const properties = computed(() => {
  const d = data.value
  const result: string[] = []

  if (d.weaponCategory) {
    result.push((d.weaponCategory as string).charAt(0).toUpperCase() + (d.weaponCategory as string).slice(1))
  }

  if (d.property && Array.isArray(d.property)) {
    for (const p of d.property as string[]) {
      const clean = p.split('|')[0]
      if (propCodeMap[clean]) result.push(propCodeMap[clean])
    }
  }

  if (d.reqAttune) {
    if (d.reqAttune === true) result.push('Requires Attunement')
    else if (typeof d.reqAttune === 'string') result.push(`Attunement: ${d.reqAttune}`)
  }

  return result
})

const description = computed(() => {
  const entries = data.value.entries as unknown[]
  if (!entries || !Array.isArray(entries)) return ''

  return entries
    .map((entry) => {
      if (typeof entry === 'string') return stripTags(entry)
      if (typeof entry === 'object' && entry !== null) {
        const e = entry as Record<string, unknown>
        if (e.type === 'entries' && Array.isArray(e.entries)) {
          return (e.entries as unknown[]).filter(sub => typeof sub === 'string').map(s => stripTags(s as string)).join(' ')
        }
        if (e.type === 'list' && Array.isArray(e.items)) {
          return (e.items as unknown[]).filter(sub => typeof sub === 'string').map(s => stripTags(s as string)).join(', ')
        }
      }
      return ''
    })
    .filter(Boolean)
    .join(' ')
})

/** Strip 5etools tags like {@damage 2d6}, {@link text|url}, {@note text} */
function stripTags(text: string): string {
  return text.replace(/\{@\w+\s+([^|}]+)(?:\|[^}]*)?\}/g, '$1')
}

function formatValue(cp: number): string {
  if (cp >= 100) return `${Math.floor(cp / 100)} gp`
  if (cp >= 10) return `${Math.floor(cp / 10)} sp`
  return `${cp} cp`
}
</script>

<style scoped>
.item-detail-block {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.item-stat-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.item-type-label {
  font-size: 0.8rem;
  color: var(--color-text-secondary);
  font-weight: 500;
}

.rarity-badge {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: capitalize;
  padding: 1px 8px;
  border-radius: var(--radius-sm);
  background: var(--color-surface-variant);
}

.rarity-badge.common { color: var(--color-text-secondary); }
.rarity-badge.uncommon { color: #16a34a; background: #f0fdf4; }
.rarity-badge.rare { color: #2563eb; background: #eff6ff; }
.rarity-badge.very-rare { color: #7c3aed; background: #f5f3ff; }
.rarity-badge.legendary { color: #ea580c; background: #fff7ed; }
.rarity-badge.artifact { color: #dc2626; background: #fef2f2; }

.item-stat-grid {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-sm) var(--spacing-lg);
}

.stat-cell {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.stat-label {
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-secondary);
  font-weight: 600;
}

.stat-value {
  font-size: 0.9rem;
  font-weight: 500;
}

.stat-value.disadvantage {
  color: #dc2626;
}

.item-properties {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.prop-tag {
  font-size: 0.75rem;
  padding: 1px 6px;
  border-radius: var(--radius-sm);
  background: var(--color-surface-variant);
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border);
}

.item-description {
  font-size: 0.85rem;
  color: var(--color-text);
  line-height: 1.5;
  border-top: 1px solid var(--color-border);
  padding-top: var(--spacing-sm);
}
</style>
