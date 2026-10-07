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
            <div class="entry-row">
              <input
                class="entry-initiative"
                type="number"
                :value="e.initiative ?? ''"
                :aria-label="`Initiative for ${e.display_name}`"
                placeholder="–"
                @change="onInitiativeChange(e.id, $event)"
              />
              <button
                class="entry-name"
                :aria-expanded="expandedId === e.id"
                :title="`Show HP controls for ${e.display_name}`"
                @click="toggleExpanded(e.id)"
              >
                {{ e.display_name }}
              </button>
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
            </div>

            <div v-if="e.conditions.length || e.is_concentrating" class="entry-badges">
              <span
                v-if="e.is_concentrating"
                class="concentration-badge"
                data-testid="concentration-badge"
                title="Concentrating"
              >
                ◎ Conc.
              </span>
              <button
                v-for="c in e.conditions"
                :key="c.name"
                class="condition-pill"
                data-testid="condition-pill"
                :title="conditionTitle(c)"
                @click="combat.removeCondition(e.id, c.name)"
              >
                {{ c.name }}
              </button>
            </div>

            <div v-if="expandedId === e.id" class="entry-details" data-testid="entry-details">
              <div
                v-if="concentrationPrompt?.entryId === e.id"
                class="concentration-prompt"
                data-testid="concentration-prompt"
                role="alert"
              >
                <span>Concentration save DC {{ concentrationPrompt.dc }}</span>
                <button class="btn" data-testid="concentration-kept" @click="concentrationPrompt = null">Kept</button>
                <button class="btn btn-danger" data-testid="concentration-lost" @click="loseConcentration(e.id)">
                  Lost
                </button>
              </div>
              <template v-if="e.current_hp !== null">
                <div class="hp-bar" :title="hpTitle(e)">
                  <div
                    class="hp-bar-fill"
                    data-testid="hp-bar-fill"
                    :class="hpBand(e)"
                    :style="{ width: `${hpPercent(e)}%` }"
                  />
                </div>
                <div class="hp-actions">
                  <input
                    v-model="hpAmount"
                    class="hp-amount"
                    data-testid="hp-amount"
                    type="number"
                    min="0"
                    placeholder="HP"
                    :aria-label="`HP amount for ${e.display_name}`"
                    @keydown.enter="applyDamage(e.id)"
                  />
                  <button class="btn btn-danger" data-testid="hp-damage" :disabled="!validAmount" @click="applyDamage(e.id)">
                    Damage
                  </button>
                  <button class="btn btn-success" data-testid="hp-heal" :disabled="!validAmount" @click="applyHeal(e.id)">
                    Heal
                  </button>
                </div>
                <label class="temp-hp">
                  Temp HP
                  <input
                    data-testid="temp-hp"
                    type="number"
                    min="0"
                    :value="e.temp_hp"
                    @change="onTempHpChange(e.id, $event)"
                  />
                </label>
              </template>
              <div v-else class="set-hp">
                <input
                  v-model="maxHpInput"
                  data-testid="set-max-hp"
                  type="number"
                  min="1"
                  placeholder="Max HP"
                  :aria-label="`Max HP for ${e.display_name}`"
                />
                <button class="btn" data-testid="save-max-hp" :disabled="!validMaxHp" @click="saveMaxHp(e.id)">
                  Set HP
                </button>
              </div>
              <div class="conditions-editor">
                <select v-model="conditionToAdd" data-testid="condition-select" aria-label="Condition">
                  <option value="">Condition…</option>
                  <option v-for="c in SRD_CONDITIONS" :key="c" :value="c">{{ c }}</option>
                </select>
                <input
                  v-model="conditionRounds"
                  data-testid="condition-duration"
                  type="number"
                  min="1"
                  placeholder="Rnds"
                  aria-label="Duration in rounds (empty: until removed)"
                  title="Duration in rounds, including this one (empty: until removed)"
                />
                <button class="btn" data-testid="add-condition" :disabled="!conditionToAdd" @click="addCondition(e.id)">
                  Add
                </button>
              </div>
              <label class="concentration">
                <input
                  type="checkbox"
                  data-testid="concentration-toggle"
                  :checked="e.is_concentrating"
                  @change="combat.setConcentration(e.id, ($event.target as HTMLInputElement).checked)"
                />
                Concentrating
              </label>
              <ul v-if="e.damage_log.length" class="hp-log" data-testid="hp-log" aria-label="Recent HP changes">
                <li v-for="(change, i) in [...e.damage_log].reverse()" :key="i" :class="change.kind">
                  {{ change.kind === 'damage' ? '−' : '+' }}{{ change.amount }} (round {{ change.round }})
                </li>
              </ul>
            </div>
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
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'
import { getMonsterDisplayName, type MonsterWithData } from '@/features/modules/composables/useModuleMonsters'
import { useCombatSession } from '../composables/useCombatSession'
import { SRD_CONDITIONS, type ActiveCondition, type CombatEntry } from '../types'

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
  if (e.current_hp === null) return '—'
  const temp = e.temp_hp > 0 ? ` +${e.temp_hp}` : ''
  return `${e.current_hp}/${e.max_hp ?? '?'}${temp}`
}

// --- HP panel (one entry expanded at a time) ---------------------------------

const expandedId = ref<string | null>(null)
const hpAmount = ref<number | string>('')
const maxHpInput = ref<number | string>('')

function toggleExpanded(entryId: string) {
  expandedId.value = expandedId.value === entryId ? null : entryId
  hpAmount.value = ''
  maxHpInput.value = ''
  conditionToAdd.value = ''
  conditionRounds.value = ''
}

// --- Conditions and concentration -------------------------------------------

const conditionToAdd = ref('')
const conditionRounds = ref<number | string>('')
const concentrationPrompt = ref<{ entryId: string; dc: number } | null>(null)

function conditionTitle(c: ActiveCondition): string {
  const until = c.expires_round === null ? '' : ` until the end of round ${c.expires_round}`
  return `${c.name}${until} — click to remove`
}

async function addCondition(entryId: string) {
  if (!conditionToAdd.value) return
  const rounds = positiveInt(conditionRounds.value, 1)
  await combat.addCondition(entryId, conditionToAdd.value, rounds)
  conditionToAdd.value = ''
  conditionRounds.value = ''
}

async function loseConcentration(entryId: string) {
  concentrationPrompt.value = null
  await combat.setConcentration(entryId, false)
}

function positiveInt(value: number | string, min: number): number | null {
  const n = typeof value === 'number' ? value : Number.parseInt(String(value).trim(), 10)
  return Number.isInteger(n) && n >= min ? n : null
}

const validAmount = computed(() => positiveInt(hpAmount.value, 0) !== null)
const validMaxHp = computed(() => positiveInt(maxHpInput.value, 1) !== null)

function hpPercent(e: CombatEntry): number {
  if (e.current_hp === null || !e.max_hp) return 0
  return Math.round((Math.max(0, e.current_hp) / e.max_hp) * 100)
}

function hpBand(e: CombatEntry): string {
  const pct = hpPercent(e)
  return pct > 50 ? 'healthy' : pct > 0 ? 'bloodied' : 'down'
}

async function applyDamage(entryId: string) {
  const amount = positiveInt(hpAmount.value, 0)
  if (amount === null) return
  const result = await combat.damage(entryId, amount)
  if (!result) return
  hpAmount.value = ''
  concentrationPrompt.value =
    result.concentration_dc !== null ? { entryId, dc: result.concentration_dc } : null
}

async function applyHeal(entryId: string) {
  const amount = positiveInt(hpAmount.value, 0)
  if (amount === null) return
  await combat.heal(entryId, amount)
  hpAmount.value = ''
}

function onTempHpChange(entryId: string, event: Event) {
  const amount = positiveInt((event.target as HTMLInputElement).value, 0)
  combat.setTempHp(entryId, amount ?? 0)
}

async function saveMaxHp(entryId: string) {
  const maxHp = positiveInt(maxHpInput.value, 1)
  if (maxHp === null) return
  await combat.setMaxHp(entryId, maxHp)
  maxHpInput.value = ''
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
  padding: 4px var(--spacing-xs);
  border-radius: var(--radius-sm);
  border-left: 3px solid transparent;
}

.entry-row {
  display: grid;
  grid-template-columns: 44px 1fr auto 20px;
  align-items: center;
  gap: var(--spacing-xs);
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
  text-align: left;
  border: none;
  background: transparent;
  color: inherit;
  font: inherit;
  padding: 0;
  cursor: pointer;
}

.entry-details {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  margin: var(--spacing-xs) 0 var(--spacing-xs) 48px;
  font-weight: normal;
}

.hp-bar {
  height: 6px;
  border-radius: 3px;
  background: var(--color-background);
  overflow: hidden;
}

.hp-bar-fill {
  height: 100%;
  transition: width var(--transition-fast, 0.15s);
}

.hp-bar-fill.healthy {
  background: var(--color-success);
}

.hp-bar-fill.bloodied {
  background: var(--color-warning);
}

.hp-bar-fill.down {
  background: var(--color-error);
}

.hp-actions,
.set-hp {
  display: flex;
  gap: var(--spacing-xs);
}

.hp-amount,
.set-hp input,
.temp-hp input {
  width: 64px;
  min-width: 0;
  padding: 2px 4px;
  background: var(--color-background);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.hp-actions .btn,
.set-hp .btn {
  padding: 2px 8px;
  font-size: 0.8rem;
}

.temp-hp {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  font-size: 0.8rem;
  color: var(--color-text-secondary);
}

.entry-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin: 2px 0 0 48px;
}

.condition-pill,
.concentration-badge {
  font-size: 0.7rem;
  line-height: 1.4;
  padding: 0 6px;
  border-radius: 999px;
  border: 1px solid var(--color-border);
  background: var(--color-background);
  color: var(--color-text);
}

.condition-pill {
  cursor: pointer;
}

.condition-pill:hover {
  text-decoration: line-through;
}

.concentration-badge {
  border-color: var(--color-info, var(--color-border));
  color: var(--color-info, var(--color-text));
}

.conditions-editor {
  display: flex;
  gap: var(--spacing-xs);
}

.conditions-editor select {
  flex: 1;
  min-width: 0;
}

.conditions-editor select,
.conditions-editor input {
  padding: 2px 4px;
  background: var(--color-background);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.conditions-editor input {
  width: 60px;
}

.conditions-editor .btn {
  padding: 2px 8px;
  font-size: 0.8rem;
}

.concentration {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  font-size: 0.8rem;
}

.concentration-prompt {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: 4px var(--spacing-xs);
  border-radius: var(--radius-sm);
  background: color-mix(in srgb, var(--color-warning) 20%, transparent);
  font-size: 0.8rem;
  font-weight: 600;
}

.concentration-prompt span {
  flex: 1;
}

.concentration-prompt .btn {
  padding: 2px 8px;
  font-size: 0.75rem;
}

.hp-log {
  list-style: none;
  margin: 0;
  padding: 0;
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.hp-log .damage {
  color: var(--color-error);
}

.hp-log .heal {
  color: var(--color-success);
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
