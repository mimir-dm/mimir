<template>
  <aside class="initiative-tracker" :class="{ collapsed }" aria-label="Initiative tracker">
    <header class="tracker-header">
      <button
        class="toggle"
        data-testid="toggle-tracker"
        :title="collapsed ? 'Show initiative tracker' : 'Hide initiative tracker'"
        :aria-expanded="!collapsed"
        @click="toggleCollapsed"
      >
        {{ collapsed ? '‹' : '›' }}
      </button>
      <template v-if="!collapsed">
        <h2 class="title">Initiative</h2>
        <span v-if="combat.round.value !== null" class="round" data-testid="combat-round">
          Round {{ combat.round.value }}
        </span>
      </template>
    </header>

    <div v-if="!collapsed" class="tracker-body">
      <p v-if="combat.error.value" class="error" data-testid="combat-error" role="alert">
        {{ combat.error.value }}
      </p>

      <div v-if="!combat.state.value" class="no-combat">
        <p>No combat running in this module.</p>
        <button class="btn btn-primary" data-testid="start-combat" @click="combat.start()">
          Start combat
        </button>
      </div>

      <template v-else>
        <ol class="entries">
          <li
            v-for="e in combat.state.value.entries"
            :key="e.id"
            class="entry"
            :class="{ current: e.id === combat.state.value.current_entry_id, down: e.current_hp === 0 }"
            data-testid="combat-entry"
          >
            <input
              class="entry-initiative"
              type="number"
              :value="e.initiative ?? ''"
              :aria-label="`Initiative for ${e.display_name}`"
              placeholder="–"
              @change="onInitiativeChange(e.id, $event)"
            />
            <span class="entry-name">{{ e.display_name }}</span>
            <span class="entry-hp" :title="hpTitle(e)">{{ hpText(e) }}</span>
            <button
              class="remove"
              data-testid="remove-entry"
              :aria-label="`Remove ${e.display_name}`"
              title="Remove from combat"
              @click="combat.removeEntry(e.id)"
            >
              ×
            </button>
          </li>
          <li v-if="combat.state.value.entries.length === 0" class="empty">
            Add creatures below.
          </li>
        </ol>

        <div class="turn-controls">
          <button class="btn" data-testid="previous-turn" title="Previous turn" @click="combat.previousTurn()">
            ‹ Prev
          </button>
          <button class="btn btn-primary" data-testid="next-turn" @click="combat.nextTurn()">
            Next turn ›
          </button>
        </div>

        <section class="add" aria-label="Add to combat">
          <div class="add-row">
            <select v-model="monsterToAdd" data-testid="add-monster-select" aria-label="Monster group">
              <option value="">Monster group…</option>
              <option v-for="m in monsterGroups" :key="m.id" :value="m.id">
                {{ getMonsterDisplayName(m) }} ×{{ m.quantity }}
              </option>
            </select>
            <button class="btn" data-testid="add-monster" :disabled="!monsterToAdd" @click="addMonster">
              Add
            </button>
          </div>
          <div class="add-row">
            <select v-model="pcToAdd" data-testid="add-pc-select" aria-label="Player character">
              <option value="">Player character…</option>
              <option v-for="pc in pcs" :key="pc.id" :value="pc.id">{{ pc.name }}</option>
            </select>
            <button class="btn" data-testid="add-pc" :disabled="!pcToAdd" @click="addPc">Add</button>
          </div>
          <div class="add-row">
            <input
              v-model="customName"
              data-testid="add-custom-name"
              type="text"
              placeholder="Custom (e.g. Lair action)"
              aria-label="Custom entry name"
              @keydown.enter="addCustom"
            />
            <button class="btn" data-testid="add-custom" :disabled="!customName.trim()" @click="addCustom">
              Add
            </button>
          </div>
        </section>

        <button class="btn btn-danger end" data-testid="end-combat" @click="confirmingEnd = true">
          End combat
        </button>
      </template>
    </div>

    <AppModal :visible="confirmingEnd" title="End combat?" size="sm" @close="confirmingEnd = false">
      <p>The fight's order, HP and conditions are cleared from the tracker.</p>
      <template #footer>
        <button class="btn" @click="confirmingEnd = false">Cancel</button>
        <button class="btn btn-danger" data-testid="confirm-end-combat" @click="endCombat">End combat</button>
      </template>
    </AppModal>
  </aside>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'
import { getMonsterDisplayName, type MonsterWithData } from '@/features/modules/composables/useModuleMonsters'
import { useCombatSession } from '../composables/useCombatSession'
import type { CombatEntry } from '../types'

const props = defineProps<{
  moduleId: string
  campaignId: string
}>()

const COLLAPSED_KEY = 'mimir.initiativeTracker.collapsed'

const combat = useCombatSession(props.moduleId)
const collapsed = ref(localStorage.getItem(COLLAPSED_KEY) === 'true')
const confirmingEnd = ref(false)

const monsterGroups = ref<MonsterWithData[]>([])
const pcs = ref<{ id: string; name: string }[]>([])
const monsterToAdd = ref('')
const pcToAdd = ref('')
const customName = ref('')

function toggleCollapsed() {
  collapsed.value = !collapsed.value
  localStorage.setItem(COLLAPSED_KEY, String(collapsed.value))
}

function hpText(e: CombatEntry): string {
  return e.current_hp === null ? '—' : `${e.current_hp}/${e.max_hp ?? '?'}`
}

function hpTitle(e: CombatEntry): string {
  return e.current_hp === null ? 'No HP set' : `HP ${e.current_hp} of ${e.max_hp ?? '?'}`
}

function onInitiativeChange(entryId: string, event: Event) {
  const raw = (event.target as HTMLInputElement).value.trim()
  const value = raw === '' ? null : Number.parseInt(raw, 10)
  combat.setInitiative(entryId, Number.isNaN(value) ? null : value)
}

async function addMonster() {
  if (!monsterToAdd.value) return
  await combat.addMonsterGroup(monsterToAdd.value)
  monsterToAdd.value = ''
}

async function addPc() {
  if (!pcToAdd.value) return
  await combat.addCharacter(pcToAdd.value)
  pcToAdd.value = ''
}

async function addCustom() {
  const name = customName.value.trim()
  if (!name) return
  await combat.addCustom(name)
  customName.value = ''
}

async function endCombat() {
  confirmingEnd.value = false
  await combat.end()
}

async function loadAddOptions() {
  try {
    const [monsters, characters] = await Promise.all([
      invoke<{ success: boolean; data?: MonsterWithData[] }>('list_module_monsters_with_data', {
        moduleId: props.moduleId,
      }),
      invoke<{ success: boolean; data?: { id: string; name: string }[] }>('list_pcs', {
        campaignId: props.campaignId,
      }),
    ])
    monsterGroups.value = monsters.success ? (monsters.data ?? []) : []
    pcs.value = characters.success ? (characters.data ?? []) : []
  } catch {
    // The tracker still works with custom entries.
  }
}

onMounted(() => {
  combat.load()
  loadAddOptions()
})
</script>

<style scoped>
.initiative-tracker {
  width: 300px;
  flex: 0 0 300px;
  display: flex;
  flex-direction: column;
  background: var(--color-surface);
  border-left: 1px solid var(--color-border);
  color: var(--color-text);
  overflow: hidden;
}

.initiative-tracker.collapsed {
  width: 36px;
  flex-basis: 36px;
}

.tracker-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm);
  border-bottom: 1px solid var(--color-border);
}

.toggle {
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 1.25rem;
  line-height: 1;
  cursor: pointer;
}

.title {
  font-size: 0.95rem;
  margin: 0;
  flex: 1;
}

.round {
  font-size: 0.75rem;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  background: var(--color-surface-variant);
}

.tracker-body {
  flex: 1;
  overflow-y: auto;
  padding: var(--spacing-sm);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.error {
  margin: 0;
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-sm);
  background: color-mix(in srgb, var(--color-error) 15%, transparent);
  color: var(--color-error);
  font-size: 0.8rem;
}

.no-combat {
  text-align: center;
  color: var(--color-text-secondary);
}

.entries {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.entry {
  display: grid;
  grid-template-columns: 44px 1fr auto 20px;
  align-items: center;
  gap: var(--spacing-xs);
  padding: 4px var(--spacing-xs);
  border-radius: var(--radius-sm);
  border-left: 3px solid transparent;
}

.entry.current {
  background: var(--color-surface-variant);
  border-left-color: var(--color-warning);
  font-weight: 600;
}

.entry.down .entry-name {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}

.entry-initiative {
  width: 44px;
  padding: 2px 4px;
  text-align: center;
  background: var(--color-background);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.entry-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.entry-hp {
  font-size: 0.8rem;
  font-variant-numeric: tabular-nums;
  color: var(--color-text-secondary);
}

.remove {
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
}

.empty {
  color: var(--color-text-secondary);
  font-size: 0.85rem;
  padding: var(--spacing-xs);
}

.turn-controls {
  display: flex;
  gap: var(--spacing-xs);
}

.turn-controls .btn-primary {
  flex: 1;
}

.add {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.add-row {
  display: flex;
  gap: var(--spacing-xs);
}

.add-row select,
.add-row input {
  flex: 1;
  min-width: 0;
  padding: 4px;
  background: var(--color-background);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.end {
  margin-top: auto;
}
</style>
