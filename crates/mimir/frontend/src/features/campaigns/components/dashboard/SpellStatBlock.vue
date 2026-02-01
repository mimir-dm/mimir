<template>
  <div class="spell-stat-block">
    <div class="spell-header">
      <h3 class="spell-name">{{ name }}</h3>
      <div class="spell-level-school">
        {{ levelSchoolLine }}
      </div>
    </div>

    <div class="spell-divider"></div>

    <div class="spell-properties">
      <div v-if="castingTime" class="spell-prop">
        <span class="prop-label">Casting Time:</span> {{ castingTime }}
      </div>
      <div v-if="range" class="spell-prop">
        <span class="prop-label">Range:</span> {{ range }}
      </div>
      <div v-if="components" class="spell-prop">
        <span class="prop-label">Components:</span> <span v-html="components"></span>
      </div>
      <div v-if="duration" class="spell-prop">
        <span class="prop-label">Duration:</span> {{ duration }}
      </div>
    </div>

    <div class="spell-divider"></div>

    <div v-if="entries.length > 0" class="spell-entries">
      <div v-html="formatSpellEntries(entries)"></div>
    </div>

    <div v-if="higherLevels" class="spell-higher-levels">
      <p><em><strong>At Higher Levels.</strong></em> <span v-html="higherLevels"></span></p>
    </div>

    <div v-if="classes" class="spell-classes">
      <span class="prop-label">Classes:</span> {{ classes }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { processFormattingTags } from '@/features/sources/utils/textFormatting'

const props = defineProps<{
  data: Record<string, unknown>
  name: string
}>()

const SCHOOL_LABELS: Record<string, string> = {
  A: 'Abjuration', C: 'Conjuration', D: 'Divination', E: 'Enchantment',
  V: 'Evocation', I: 'Illusion', N: 'Necromancy', T: 'Transmutation',
}

const levelSchoolLine = computed(() => {
  const level = props.data.level as number | undefined
  const school = SCHOOL_LABELS[props.data.school as string] || (props.data.school as string) || 'Unknown'

  if (level === 0) return `${school} cantrip`
  if (level === 1) return `1st-level ${school.toLowerCase()}`
  if (level === 2) return `2nd-level ${school.toLowerCase()}`
  if (level === 3) return `3rd-level ${school.toLowerCase()}`
  if (level !== undefined) return `${level}th-level ${school.toLowerCase()}`
  return school
})

const castingTime = computed(() => {
  const t = props.data.time as Array<{ number: number; unit: string }> | undefined
  if (!t || t.length === 0) return null
  return t.map(entry => `${entry.number} ${entry.unit}`).join(', ')
})

const range = computed(() => {
  const r = props.data.range as { type: string; distance?: { type: string; amount?: number } } | undefined
  if (!r) return null
  if (r.type === 'point') {
    if (r.distance?.type === 'self') return 'Self'
    if (r.distance?.type === 'touch') return 'Touch'
    if (r.distance?.amount !== undefined) return `${r.distance.amount} ${r.distance.type}`
  }
  if (r.type === 'special') return 'Special'
  if (r.type === 'self') return 'Self'
  return r.type
})

const components = computed(() => {
  const c = props.data.components as { v?: boolean; s?: boolean; m?: unknown } | undefined
  if (!c) return null
  const parts: string[] = []
  if (c.v) parts.push('V')
  if (c.s) parts.push('S')
  if (c.m) {
    if (typeof c.m === 'string') parts.push(`M (${processFormattingTags(c.m)})`)
    else if (typeof c.m === 'object' && c.m !== null && (c.m as any).text) parts.push(`M (${processFormattingTags((c.m as any).text)})`)
    else parts.push('M')
  }
  return parts.join(', ')
})

const duration = computed(() => {
  const d = props.data.duration as Array<{ type: string; duration?: { type: string; amount: number }; concentration?: boolean }> | undefined
  if (!d || d.length === 0) return null
  return d.map(entry => {
    if (entry.type === 'instant') return 'Instantaneous'
    if (entry.type === 'permanent') return 'Until dispelled'
    if (entry.type === 'special') return 'Special'
    if (entry.type === 'timed' && entry.duration) {
      const prefix = entry.concentration ? 'Concentration, up to ' : ''
      return `${prefix}${entry.duration.amount} ${entry.duration.type}${entry.duration.amount > 1 ? 's' : ''}`
    }
    return entry.type
  }).join(', ')
})

const entries = computed(() => {
  return (props.data.entries as unknown[]) || []
})

function formatSpellEntries(entries: unknown[]): string {
  if (!entries || !Array.isArray(entries)) return ''
  return entries.map(entry => {
    if (typeof entry === 'string') {
      return `<p class="spell-entry-text">${processFormattingTags(entry)}</p>`
    } else if (entry && typeof entry === 'object') {
      const obj = entry as Record<string, unknown>
      if (obj.type === 'entries' && Array.isArray(obj.entries)) {
        const name = obj.name ? `<p class="spell-sub-entry-name"><em>${processFormattingTags(String(obj.name))}.</em></p>` : ''
        return `<div class="spell-sub-entry">${name}${formatSpellEntries(obj.entries as unknown[])}</div>`
      }
      if (obj.type === 'list' && Array.isArray(obj.items)) {
        const items = (obj.items as unknown[]).map(item => {
          if (typeof item === 'string') return `<li>${processFormattingTags(item)}</li>`
          if (item && typeof item === 'object' && (item as any).name) return `<li>${processFormattingTags(String((item as any).name))}</li>`
          return `<li>${JSON.stringify(item)}</li>`
        }).join('')
        return `<ul class="spell-list-items">${items}</ul>`
      }
      if (obj.type === 'table' && Array.isArray(obj.rows)) {
        const cols = Array.isArray(obj.colLabels) ? (obj.colLabels as string[]).map(c => `<th>${processFormattingTags(c)}</th>`).join('') : ''
        const rows = (obj.rows as unknown[][]).map(row => `<tr>${(row as unknown[]).map(cell => `<td>${processFormattingTags(String(cell))}</td>`).join('')}</tr>`).join('')
        return `<table class="spell-table-inner"><thead><tr>${cols}</tr></thead><tbody>${rows}</tbody></table>`
      }
      if (obj.entries) return formatSpellEntries(obj.entries as unknown[])
    }
    return ''
  }).join('')
}

const higherLevels = computed(() => {
  const hl = props.data.entriesHigherLevel as Array<{ entries: string[] }> | undefined
  if (!hl || hl.length === 0) return null
  return hl.flatMap(h => h.entries).filter(e => typeof e === 'string').map(e => processFormattingTags(e)).join(' ')
})

const classes = computed(() => {
  const c = props.data.classes as { fromClassList?: Array<{ name: string }> } | undefined
  if (!c?.fromClassList) return null
  return c.fromClassList.map(cl => cl.name).join(', ')
})
</script>

<style scoped>
/* v-html rendered content needs :deep for scoped styles */
.spell-stat-block {
  background: var(--color-surface-variant);
  border: 2px solid var(--color-primary-300);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  font-size: 0.9rem;
  line-height: 1.5;
}

.spell-header {
  margin-bottom: var(--spacing-xs);
}

.spell-name {
  margin: 0;
  font-size: 1.1rem;
  color: var(--color-primary-500);
}

.spell-level-school {
  font-style: italic;
  color: var(--color-text-secondary);
  font-size: 0.85rem;
}

.spell-divider {
  height: 2px;
  background: var(--color-primary-300);
  margin: var(--spacing-xs) 0;
  opacity: 0.5;
}

.spell-properties {
  font-size: 0.85rem;
}

.spell-prop {
  margin: 2px 0;
}

.prop-label {
  font-weight: 700;
}

.spell-entries {
  margin-top: var(--spacing-sm);
}

:deep(.spell-entry-text) {
  margin: var(--spacing-xs) 0;
}

:deep(.spell-sub-entry) {
  margin: var(--spacing-xs) 0;
}

:deep(.spell-sub-entry-name) {
  margin: var(--spacing-xs) 0 0;
  font-weight: 600;
}

:deep(.spell-list-items) {
  margin: var(--spacing-xs) 0;
  padding-left: var(--spacing-lg);
}

:deep(.spell-table-inner) {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.85rem;
  margin: var(--spacing-xs) 0;
}

:deep(.spell-table-inner th),
:deep(.spell-table-inner td) {
  border: 1px solid var(--color-border);
  padding: 4px 8px;
  text-align: left;
}

:deep(.spell-table-inner th) {
  background: var(--color-surface);
  font-weight: 600;
}

:deep(.dice-roll),
:deep(.damage-roll) {
  font-weight: 600;
}

:deep(.ref-link) {
  color: var(--color-primary-500);
  cursor: default;
}

.spell-higher-levels {
  margin-top: var(--spacing-sm);
  font-size: 0.85rem;
}

.spell-classes {
  margin-top: var(--spacing-sm);
  font-size: 0.8rem;
  color: var(--color-text-secondary);
}
</style>
