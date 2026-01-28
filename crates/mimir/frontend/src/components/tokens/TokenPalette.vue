<template>
  <div class="token-palette">
    <div class="palette-header">
      <h4>Token Palette</h4>
      <button v-if="selectedType" class="clear-btn" @click="clearSelection">
        Clear
      </button>
    </div>

    <!-- Module Monsters Quick Select (when moduleId provided) -->
    <div v-if="moduleMonsters.length > 0" class="module-monsters-section">
      <div class="section-label">Module Monsters</div>
      <div class="module-monster-list">
        <button
          v-for="mm in moduleMonsters"
          :key="mm.id"
          class="module-monster-btn"
          :class="{ active: selectedModuleMonster?.id === mm.id }"
          @click="selectModuleMonster(mm)"
        >
          <span class="mm-name">{{ mm.monster_name }}</span>
          <span class="mm-meta">
            <span v-if="mm.encounter_tag" class="mm-tag">{{ mm.encounter_tag }}</span>
            <span class="mm-qty">×{{ mm.quantity }}</span>
          </span>
        </button>
      </div>
    </div>

    <!-- Token Type Selector -->
    <div class="type-grid">
      <button
        v-for="type in tokenTypes"
        :key="type.value"
        class="type-btn"
        :class="{ active: selectedType === type.value && !selectedLightType }"
        :style="{ '--type-color': type.color }"
        @click="selectType(type.value)"
      >
        <span class="type-icon">{{ type.icon }}</span>
        <span class="type-label">{{ type.label }}</span>
      </button>
    </div>

    <!-- Light Sources -->
    <div class="light-section">
      <div class="section-label">Light Sources</div>
      <div class="light-grid">
        <button
          v-for="light in lightTypes"
          :key="light.value"
          class="light-btn"
          :class="{ active: selectedLightType === light.value }"
          @click="selectLightType(light.value)"
        >
          <span class="light-icon">{{ light.icon }}</span>
          <span class="light-label">{{ light.label }}</span>
        </button>
      </div>
      <div v-if="selectedLightType" class="placement-hint light-hint">
        Click on the map to place a {{ selectedLightType }}.
      </div>
    </div>

    <!-- Configuration (when type selected) -->
    <div v-if="selectedType" class="config-section">
      <!-- Name Input -->
      <div class="form-group">
        <label>Name</label>
        <input
          v-model="tokenName"
          type="text"
          class="form-input"
          :placeholder="getPlaceholderName()"
        />
      </div>

      <!-- Size Selector -->
      <div class="form-group">
        <label>Size</label>
        <select v-model="selectedSize" class="form-select">
          <option v-for="size in sizes" :key="size.value" :value="size.value">
            {{ size.label }} ({{ size.squares }} sq)
          </option>
        </select>
      </div>

      <!-- Color Picker -->
      <div class="form-group">
        <label>Color</label>
        <div class="color-input-row">
          <input
            v-model="selectedColor"
            type="color"
            class="color-picker"
          />
          <span class="color-value">{{ selectedColor }}</span>
        </div>
      </div>

      <!-- Monster Search (for monster type) -->
      <div v-if="selectedType === 'monster'" class="form-group">
        <label>Link Monster (optional)</label>
        <div class="search-input-wrapper">
          <input
            v-model="monsterSearch"
            type="text"
            class="form-input"
            placeholder="Search monsters..."
            @input="searchMonsters"
          />
          <div v-if="monsterResults.length > 0" class="search-results">
            <button
              v-for="monster in monsterResults"
              :key="monster.id"
              class="search-result-item"
              @click="selectMonster(monster)"
            >
              {{ monster.name }}
              <span class="monster-cr">CR {{ formatCr(monster.cr) }}</span>
            </button>
          </div>
        </div>
        <div v-if="selectedMonster" class="selected-entity">
          <span>{{ selectedMonster.name }}</span>
          <button class="remove-entity-btn" @click="clearMonster">×</button>
        </div>
      </div>

      <!-- Trap Search (for trap type) -->
      <div v-if="selectedType === 'trap'" class="form-group">
        <label>Link Trap (optional)</label>
        <div class="search-input-wrapper">
          <input
            v-model="trapSearch"
            type="text"
            class="form-input"
            placeholder="Search traps..."
            @input="searchTraps"
          />
          <div v-if="trapResults.length > 0" class="search-results">
            <button
              v-for="trap in trapResults"
              :key="`${trap.name}-${trap.source}`"
              class="search-result-item"
              @click="selectTrap(trap)"
            >
              {{ trap.name }}
              <span class="trap-type">{{ trap.trap_type !== 'Unknown' ? trap.trap_type : trap.category }}</span>
            </button>
          </div>
        </div>
        <div v-if="selectedTrap" class="selected-entity">
          <span>{{ selectedTrap.name }}</span>
          <button class="remove-entity-btn" @click="clearTrap">×</button>
        </div>
      </div>

      <!-- Visibility Toggle -->
      <div class="form-group">
        <label class="checkbox-label">
          <input v-model="visibleToPlayers" type="checkbox" />
          <span>Visible to players</span>
        </label>
      </div>

      <!-- Placement Instructions -->
      <div class="placement-hint">
        Click on the map to place a {{ selectedType }} token.
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { TokenType, TokenSize, TokenConfigWithMonster } from '@/types/api'
import { TOKEN_TYPE_COLORS } from '@/types/api'
import { useCampaignStore } from '@/stores/campaigns'

interface Monster {
  id: number
  name: string
  source: string
  size: string | string[] | unknown  // Can be string, array, or object from 5etools
  cr: string | string[] | unknown    // Can also be array for variable CR
}

interface Trap {
  name: string
  source: string
  trap_type: string
  category: string
}

interface ModuleMonsterWithData {
  id: number
  module_id: number
  monster_name: string
  monster_source: string
  quantity: number
  encounter_tag: string | null
  /** Custom display name (e.g., "Frost Wight" when using goblin stats) */
  display_name: string | null
  /** DM notes about customizations or thematic changes */
  notes: string | null
  monster_data: {
    id?: number
    size?: string
    cr?: string
    type?: string
  } | null
}

interface Props {
  moduleId?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  moduleId: null
})

export type LightType = 'torch' | 'lantern' | 'candle'

const emit = defineEmits<{
  'token-config-change': [config: TokenConfigWithMonster | null]
  'light-config-change': [lightType: LightType | null]
}>()

const campaignStore = useCampaignStore()

// Module monsters state
const moduleMonsters = ref<ModuleMonsterWithData[]>([])
const selectedModuleMonster = ref<ModuleMonsterWithData | null>(null)

// Token type options (PC tokens are added at play time, not during map setup)
const tokenTypes = [
  { value: 'monster' as TokenType, label: 'Monster', icon: '👹', color: TOKEN_TYPE_COLORS.monster },
  { value: 'npc' as TokenType, label: 'NPC', icon: '👤', color: TOKEN_TYPE_COLORS.npc },
  { value: 'trap' as TokenType, label: 'Trap', icon: '⚠️', color: TOKEN_TYPE_COLORS.trap },
  { value: 'marker' as TokenType, label: 'Marker', icon: '📍', color: TOKEN_TYPE_COLORS.marker }
]

// Light source options
const lightTypes = [
  { value: 'torch' as LightType, label: 'Torch', icon: '🔥' },
  { value: 'lantern' as LightType, label: 'Lantern', icon: '🏮' },
  { value: 'candle' as LightType, label: 'Candle', icon: '🕯️' }
]

// Size options
const sizes = [
  { value: 'tiny' as TokenSize, label: 'Tiny', squares: 0.5 },
  { value: 'small' as TokenSize, label: 'Small', squares: 1 },
  { value: 'medium' as TokenSize, label: 'Medium', squares: 1 },
  { value: 'large' as TokenSize, label: 'Large', squares: 2 },
  { value: 'huge' as TokenSize, label: 'Huge', squares: 3 },
  { value: 'gargantuan' as TokenSize, label: 'Gargantuan', squares: 4 }
]

// State
const selectedType = ref<TokenType | null>(null)
const selectedLightType = ref<LightType | null>(null)
const selectedSize = ref<TokenSize>('medium')
const selectedColor = ref(TOKEN_TYPE_COLORS.monster)
const tokenName = ref('')
const visibleToPlayers = ref(true)
const monsterSearch = ref('')
const monsterResults = ref<Monster[]>([])
const selectedMonster = ref<Monster | null>(null)
const selectedMonsterSource = ref<string | null>(null)  // e.g., 'MM' for Monster Manual

// Trap search state
const trapSearch = ref('')
const trapResults = ref<Trap[]>([])
const selectedTrap = ref<Trap | null>(null)

// Update color when type changes
watch(selectedType, (type) => {
  if (type) {
    selectedColor.value = TOKEN_TYPE_COLORS[type]
  }
  emitConfig()
})

watch([selectedSize, selectedColor, tokenName, visibleToPlayers, selectedMonster, selectedTrap], () => {
  emitConfig()
})

function selectType(type: TokenType) {
  // Clear light selection when selecting token type
  if (selectedLightType.value) {
    selectedLightType.value = null
    emit('light-config-change', null)
  }

  if (selectedType.value === type) {
    clearSelection()
  } else {
    selectedType.value = type
    tokenName.value = ''
    selectedMonster.value = null
    monsterSearch.value = ''
    monsterResults.value = []
    selectedTrap.value = null
    trapSearch.value = ''
    trapResults.value = []
  }
}

function selectLightType(type: LightType) {
  // Clear token selection when selecting light type
  if (selectedType.value) {
    selectedType.value = null
    emit('token-config-change', null)
  }

  if (selectedLightType.value === type) {
    selectedLightType.value = null
    emit('light-config-change', null)
  } else {
    selectedLightType.value = type
    emit('light-config-change', type)
  }
}

function clearSelection() {
  selectedType.value = null
  selectedLightType.value = null
  tokenName.value = ''
  selectedMonster.value = null
  selectedMonsterSource.value = null
  monsterSearch.value = ''
  monsterResults.value = []
  selectedModuleMonster.value = null
  selectedTrap.value = null
  trapSearch.value = ''
  trapResults.value = []
  emit('light-config-change', null)
  emit('token-config-change', null)
}

// Fetch module monsters when moduleId changes
async function loadModuleMonsters() {
  if (!props.moduleId) {
    moduleMonsters.value = []
    return
  }

  try {
    const response = await invoke<{ success: boolean; data?: ModuleMonsterWithData[] }>(
      'list_module_monsters_with_data',
      { moduleId: props.moduleId }
    )
    if (response.success && response.data) {
      moduleMonsters.value = response.data
    }
  } catch (e) {
    console.error('Failed to load module monsters:', e)
    moduleMonsters.value = []
  }
}

// Select a module monster for quick placement
function selectModuleMonster(mm: ModuleMonsterWithData) {
  // Toggle off if already selected
  if (selectedModuleMonster.value?.id === mm.id) {
    selectedModuleMonster.value = null
    clearSelection()
    return
  }

  selectedModuleMonster.value = mm
  selectedType.value = 'monster'
  tokenName.value = mm.monster_name

  // Get monster size from data (handles various formats: string, array, etc.)
  if (mm.monster_data?.size) {
    const sizeMap: Record<string, TokenSize> = {
      // Abbreviated (5etools format)
      't': 'tiny',
      's': 'small',
      'm': 'medium',
      'l': 'large',
      'h': 'huge',
      'g': 'gargantuan',
      // Full names
      'tiny': 'tiny',
      'small': 'small',
      'medium': 'medium',
      'large': 'large',
      'huge': 'huge',
      'gargantuan': 'gargantuan'
    }
    const normalizedSizeStr = normalizeSize(mm.monster_data.size)
    selectedSize.value = sizeMap[normalizedSizeStr] || 'medium'
  }

  // Set linked monster info (for display, not for DB linking)
  selectedMonster.value = {
    id: 0,  // Not used - monster_id is not stable
    name: mm.monster_name,
    source: mm.monster_source,
    size: mm.monster_data?.size || 'Medium',
    cr: mm.monster_data?.cr || 'N/A'
  }
  selectedMonsterSource.value = mm.monster_source

  selectedColor.value = TOKEN_TYPE_COLORS.monster
  emitConfig()
}

// Watch for moduleId changes
watch(() => props.moduleId, () => {
  loadModuleMonsters()
}, { immediate: true })

function getPlaceholderName(): string {
  switch (selectedType.value) {
    case 'monster': return selectedMonster.value?.name || 'Goblin'
    case 'pc': return 'Player Character'
    case 'npc': return 'NPC Name'
    case 'trap': return 'Pit Trap'
    case 'marker': return 'Point of Interest'
    default: return 'Token'
  }
}

// Monster search
let searchTimeout: ReturnType<typeof setTimeout> | null = null
async function searchMonsters() {
  if (searchTimeout) clearTimeout(searchTimeout)

  if (monsterSearch.value.length < 2) {
    monsterResults.value = []
    return
  }

  searchTimeout = setTimeout(async () => {
    try {
      // Get campaign sources for filtering (null means no filter)
      const sources = campaignStore.currentCampaignSources.length > 0
        ? campaignStore.currentCampaignSources
        : null

      const response = await invoke<{ success: boolean; data?: any[] }>('search_monsters', {
        filter: {
          name_contains: monsterSearch.value,
          sources: sources,
        },
        limit: 10,
        offset: 0
      })
      console.log('Monster search response:', response)
      if (response.success && response.data && Array.isArray(response.data)) {
        monsterResults.value = response.data.map(m => ({
          id: m.id || 0,
          name: m.name,
          source: m.source || 'MM',
          size: m.size || 'Medium',
          cr: m.cr || 'N/A'
        }))
      }
    } catch (e) {
      console.error('Failed to search monsters:', e)
    }
  }, 300)
}

// Helper to normalize size from various formats (string, array, etc.)
function normalizeSize(size: unknown): string {
  if (!size) return 'm'
  if (typeof size === 'string') return size.toLowerCase()
  if (Array.isArray(size) && size.length > 0) return String(size[0]).toLowerCase()
  return 'm'
}

// Helper to format CR for display (handles string, array, object formats)
function formatCr(cr: unknown): string {
  if (!cr) return 'N/A'
  if (typeof cr === 'string') return cr
  if (typeof cr === 'number') return String(cr)
  if (Array.isArray(cr) && cr.length > 0) return String(cr[0])
  if (typeof cr === 'object' && cr !== null) {
    // Handle objects like { cr: "1", lair: "2" }
    const crObj = cr as Record<string, unknown>
    if ('cr' in crObj) return String(crObj.cr)
  }
  return 'N/A'
}

function selectMonster(monster: Monster) {
  selectedMonster.value = monster
  selectedMonsterSource.value = monster.source
  tokenName.value = monster.name

  // Set token size based on monster size (handles both full names and abbreviations)
  const sizeMap: Record<string, TokenSize> = {
    // Abbreviated (5etools format)
    't': 'tiny',
    's': 'small',
    'm': 'medium',
    'l': 'large',
    'h': 'huge',
    'g': 'gargantuan',
    // Full names
    'tiny': 'tiny',
    'small': 'small',
    'medium': 'medium',
    'large': 'large',
    'huge': 'huge',
    'gargantuan': 'gargantuan'
  }
  const normalizedSizeStr = normalizeSize(monster.size)
  selectedSize.value = sizeMap[normalizedSizeStr] || 'medium'

  monsterSearch.value = ''
  monsterResults.value = []
}

function clearMonster() {
  selectedMonster.value = null
  selectedMonsterSource.value = null
  tokenName.value = ''
}

// Trap search
let trapSearchTimeout: ReturnType<typeof setTimeout> | null = null
async function searchTraps() {
  if (trapSearchTimeout) clearTimeout(trapSearchTimeout)

  if (trapSearch.value.length < 2) {
    trapResults.value = []
    return
  }

  trapSearchTimeout = setTimeout(async () => {
    try {
      // Backend expects 'filter' with 'nameContains' field (Tauri v2 converts to snake_case)
      const response = await invoke<{ success: boolean; data?: any[] }>('search_traps', {
        filter: {
          nameContains: trapSearch.value
        },
        limit: 10
      })
      console.log('Trap search response:', response)
      if (response.success && response.data && Array.isArray(response.data)) {
        trapResults.value = response.data.map(t => ({
          name: t.name,
          source: t.source || 'DMG',
          trap_type: t.trap_type || t.trapHazType || 'Unknown',
          category: t.category || 'Trap'
        }))
      }
    } catch (e) {
      console.error('Failed to search traps:', e)
    }
  }, 300)
}

function selectTrap(trap: Trap) {
  selectedTrap.value = trap
  tokenName.value = trap.name
  trapSearch.value = ''
  trapResults.value = []
}

function clearTrap() {
  selectedTrap.value = null
  tokenName.value = ''
}

function emitConfig() {
  if (!selectedType.value) {
    emit('token-config-change', null)
    return
  }

  const config: Partial<TokenConfigWithMonster> = {
    name: tokenName.value || getPlaceholderName(),
    token_type: selectedType.value,
    size: selectedSize.value,
    color: selectedColor.value,
    visible_to_players: visibleToPlayers.value,
    x: 0, // Will be set on click
    y: 0,
    map_id: '' // Will be set by parent
  }

  // Set image_path and monster info for module_monsters auto-add
  // Don't use monster_id - it's not stable across reimports
  if (selectedMonster.value && selectedMonsterSource.value) {
    config.image_path = `img/bestiary/tokens/${selectedMonsterSource.value}/${selectedMonster.value.name}.webp`
    config.monster_name = selectedMonster.value.name
    config.monster_source = selectedMonsterSource.value
  }

  emit('token-config-change', config as TokenConfigWithMonster)
}

// Expose current config for parent to use
const currentConfig = computed(() => {
  if (!selectedType.value) return null
  return {
    name: tokenName.value || getPlaceholderName(),
    token_type: selectedType.value,
    size: selectedSize.value,
    color: selectedColor.value,
    visible_to_players: visibleToPlayers.value
  }
})

defineExpose({ currentConfig, clearSelection, loadModuleMonsters })
</script>

<style scoped>
.token-palette {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  width: 220px;
}

.palette-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
}

.palette-header h4 {
  margin: 0;
  font-size: 0.875rem;
  font-weight: 600;
}

.clear-btn {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-background);
  color: var(--color-text-muted);
  cursor: pointer;
}

.clear-btn:hover {
  background: var(--color-base-200);
}

/* Module Monsters Quick Select */
.module-monsters-section {
  margin-bottom: var(--spacing-md);
  padding-bottom: var(--spacing-md);
  border-bottom: 1px solid var(--color-border);
}

.section-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  margin-bottom: var(--spacing-xs);
}

.module-monster-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 150px;
  overflow-y: auto;
}

.module-monster-btn {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-xs) var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-background);
  text-align: left;
  cursor: pointer;
  transition: all 0.15s;
}

.module-monster-btn:hover {
  border-color: var(--color-primary-500);
  background: var(--color-surface);
}

.module-monster-btn.active {
  border-color: var(--color-primary-500);
  background: var(--color-primary-100);
}

.mm-name {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text);
}

.mm-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.mm-tag {
  font-size: 0.625rem;
  padding: 1px 4px;
  background: var(--color-base-200);
  border-radius: var(--radius-sm);
  color: var(--color-text-muted);
}

.mm-qty {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.type-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--spacing-xs);
  margin-bottom: var(--spacing-sm);
}

/* Light Sources Section */
.light-section {
  margin-bottom: var(--spacing-md);
  padding-top: var(--spacing-sm);
  border-top: 1px solid var(--color-border);
}

.light-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--spacing-xs);
}

.light-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: var(--spacing-xs);
  border: 2px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-background);
  cursor: pointer;
  transition: all 0.15s;
}

.light-btn:hover {
  border-color: #ffcc00;
  background: rgba(255, 204, 0, 0.1);
}

.light-btn.active {
  border-color: #ffcc00;
  background: rgba(255, 204, 0, 0.2);
}

.light-icon {
  font-size: 1.25rem;
}

.light-label {
  font-size: 0.625rem;
  font-weight: 500;
}

.light-hint {
  background: rgba(255, 204, 0, 0.15);
  color: #b38600;
}

.type-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-sm);
  border: 2px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-background);
  cursor: pointer;
  transition: all 0.15s;
}

.type-btn:hover {
  border-color: var(--type-color);
  background: var(--color-surface);
}

.type-btn.active {
  border-color: var(--type-color);
  background: color-mix(in srgb, var(--type-color) 10%, var(--color-background));
}

.type-icon {
  font-size: 1.25rem;
}

.type-label {
  font-size: 0.75rem;
  font-weight: 500;
}

.config-section {
  border-top: 1px solid var(--color-border);
  padding-top: var(--spacing-md);
}

.form-group {
  margin-bottom: var(--spacing-sm);
}

.form-group label {
  display: block;
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--color-text-muted);
  margin-bottom: var(--spacing-xs);
}

.form-input,
.form-select {
  width: 100%;
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.875rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-background);
}

.color-input-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.color-picker {
  width: 32px;
  height: 32px;
  padding: 0;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.color-value {
  font-size: 0.75rem;
  font-family: monospace;
  color: var(--color-text-muted);
}

.search-input-wrapper {
  position: relative;
}

.search-results {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  max-height: 150px;
  overflow-y: auto;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  z-index: 10;
}

.search-result-item {
  display: flex;
  justify-content: space-between;
  width: 100%;
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.875rem;
  border: none;
  background: transparent;
  text-align: left;
  cursor: pointer;
}

.search-result-item:hover {
  background: var(--color-base-200);
}

.monster-cr,
.trap-type {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.selected-entity {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-primary-100);
  border-radius: var(--radius-sm);
  font-size: 0.875rem;
}

.remove-entity-btn {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  font-size: 1rem;
}

.remove-entity-btn:hover {
  color: var(--color-error);
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  cursor: pointer;
}

.checkbox-label input {
  width: 16px;
  height: 16px;
}

.checkbox-label span {
  font-size: 0.875rem;
}

.placement-hint {
  margin-top: var(--spacing-md);
  padding: var(--spacing-sm);
  background: var(--color-primary-50);
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  text-align: center;
  color: var(--color-primary-700);
}
</style>
