<template>
  <AppModal
    :visible="visible"
    title="Quick Add Token"
    size="md"
    @close="$emit('close')"
  >
          <!-- Monster Search -->
          <div class="form-group">
            <label>Search Monsters</label>
            <input
              ref="searchInput"
              v-model="searchQuery"
              type="text"
              class="form-input"
              placeholder="Type to search monsters..."
              @input="handleSearch"
            />
          </div>

          <!-- Search Results -->
          <div v-if="loading" class="loading-state">
            Searching...
          </div>

          <div v-else-if="searchResults.length > 0" class="results-list">
            <button
              v-for="monster in searchResults"
              :key="monster.id"
              class="result-item"
              :class="{ selected: selectedMonster?.id === monster.id }"
              @click="selectMonster(monster)"
            >
              <div class="result-main">
                <span class="result-name">{{ monster.name }}</span>
                <span class="result-cr">CR {{ monster.cr }}</span>
              </div>
              <div class="result-meta">
                <span class="result-size">{{ monster.size }}</span>
                <span class="result-type">{{ monster.type }}</span>
              </div>
            </button>
          </div>

          <EmptyState
            v-else-if="searchQuery.length >= 2 && !loading"
            variant="search"
            title="No monsters found"
            :description="`No monsters found matching '${searchQuery}'`"
          />

          <EmptyState
            v-else
            variant="search"
            title="Search for monsters"
            description="Type at least 2 characters to search"
          />

          <!-- Token Options (shown when monster selected) -->
          <div v-if="selectedMonster" class="token-options">
            <div class="selected-monster">
              <span class="selected-name">{{ selectedMonster.name }}</span>
              <button class="clear-btn" @click="clearSelection">×</button>
            </div>

            <div class="options-row">
              <div class="form-group">
                <label>Token Name</label>
                <input
                  v-model="tokenName"
                  type="text"
                  class="form-input"
                  :placeholder="selectedMonster.name"
                />
              </div>

              <div class="form-group">
                <label>Visible to Players</label>
                <label class="toggle">
                  <input v-model="visibleToPlayers" type="checkbox" />
                  <span class="toggle-slider"></span>
                </label>
              </div>
            </div>

            <!-- Vision Configuration -->
            <div class="vision-section">
              <button
                class="vision-header"
                @click="showVisionDetails = !showVisionDetails"
              >
                <span class="vision-title">Vision</span>
                <span class="vision-summary">
                  {{ visionType === 'normal' ? 'Normal' : `${VISION_PRESETS.find(p => p.type === visionType && p.range === visionRangeFt)?.label || visionType}` }}
                </span>
                <span class="vision-toggle">{{ showVisionDetails ? '▼' : '▶' }}</span>
              </button>

              <div v-if="showVisionDetails" class="vision-details">
                <div class="vision-presets">
                  <button
                    v-for="preset in VISION_PRESETS"
                    :key="`${preset.type}-${preset.range}`"
                    class="preset-btn"
                    :class="{ active: visionType === preset.type && visionRangeFt === preset.range }"
                    @click="applyVisionPreset(preset)"
                  >
                    {{ preset.label }}
                  </button>
                </div>

                <div v-if="visionType !== 'normal'" class="vision-custom">
                  <div class="form-group">
                    <label>Range (feet)</label>
                    <input
                      v-model.number="visionRangeFt"
                      type="number"
                      class="form-input"
                      min="0"
                      step="5"
                      placeholder="60"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>

    <template #footer>
      <button class="btn btn-secondary" @click="$emit('close')">Cancel</button>
      <button
        class="btn btn-primary"
        :disabled="!selectedMonster"
        @click="handleAdd"
      >
        Add to Map
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { TokenSize, CreateTokenRequest, VisionType } from '@/types/api'
import { VISION_PRESETS } from '@/types/api'
import AppModal from '@/components/shared/AppModal.vue'
import EmptyState from '@/shared/components/ui/EmptyState.vue'
import { useCampaignStore } from '@/stores/campaigns'

interface MonsterResult {
  id: string
  name: string
  cr: string
  size: string
  type: string
}

interface Props {
  visible: boolean
  mapId: string
  gridSizePx: number
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
  'add-token': [request: CreateTokenRequest]
}>()

const campaignStore = useCampaignStore()
const searchInput = ref<HTMLInputElement | null>(null)
const searchQuery = ref('')
const searchResults = ref<MonsterResult[]>([])
const loading = ref(false)
const selectedMonster = ref<MonsterResult | null>(null)
const tokenName = ref('')
const visibleToPlayers = ref(true)

// Vision configuration
const visionType = ref<VisionType>('normal')
const visionRangeFt = ref<number | null>(null)
const showVisionDetails = ref(false)

let searchTimeout: ReturnType<typeof setTimeout> | null = null

// Focus search input when modal opens
watch(() => props.visible, async (visible) => {
  if (visible) {
    await nextTick()
    searchInput.value?.focus()
    // Reset state
    searchQuery.value = ''
    searchResults.value = []
    selectedMonster.value = null
    tokenName.value = ''
    visibleToPlayers.value = true
    visionType.value = 'normal'
    visionRangeFt.value = null
    showVisionDetails.value = false
  }
})

function handleSearch() {
  if (searchTimeout) clearTimeout(searchTimeout)

  if (searchQuery.value.length < 2) {
    searchResults.value = []
    return
  }

  loading.value = true

  searchTimeout = setTimeout(async () => {
    try {
      // Get campaign sources for filtering (null means no filter)
      const sources = campaignStore.currentCampaignSources.length > 0
        ? campaignStore.currentCampaignSources
        : null

      const response = await invoke<{ success: boolean; data?: any[] }>('search_monsters', {
        filter: {
          name_contains: searchQuery.value,
          sources: sources,
        },
        limit: 15,
        offset: 0
      })

      if (response.success && response.data) {
        searchResults.value = response.data.map(m => ({
          id: m.id,
          name: m.name,
          cr: m.challenge_rating || m.cr || 'N/A',
          size: m.size || 'Medium',
          type: m.type || 'Unknown'
        }))
      }
    } catch (e) {
      console.error('Failed to search monsters:', e)
    } finally {
      loading.value = false
    }
  }, 300)
}

function selectMonster(monster: MonsterResult) {
  selectedMonster.value = monster
  tokenName.value = monster.name
}

function clearSelection() {
  selectedMonster.value = null
  tokenName.value = ''
}

// Map monster size to token size
function getTokenSize(monsterSize: string): TokenSize {
  const sizeMap: Record<string, TokenSize> = {
    'Tiny': 'tiny',
    'Small': 'small',
    'Medium': 'medium',
    'Large': 'large',
    'Huge': 'huge',
    'Gargantuan': 'gargantuan'
  }
  return sizeMap[monsterSize] || 'medium'
}

// Apply a vision preset
function applyVisionPreset(preset: typeof VISION_PRESETS[number]) {
  visionType.value = preset.type
  visionRangeFt.value = preset.range
  if (preset.type !== 'normal') {
    showVisionDetails.value = true
  }
}

function handleAdd() {
  if (!selectedMonster.value) return

  const request: CreateTokenRequest = {
    map_id: props.mapId,
    name: tokenName.value || selectedMonster.value.name,
    token_type: 'monster',
    size: getTokenSize(selectedMonster.value.size),
    x: 0, // Will be set by parent for placement
    y: 0,
    visible_to_players: visibleToPlayers.value,
    monster_id: selectedMonster.value.id,
    vision_type: visionType.value,
    vision_range_ft: visionRangeFt.value
  }

  emit('add-token', request)
  emit('close')
}
</script>

<style scoped>
/* Token search form styles */
.form-group {
  margin-bottom: var(--spacing-md);
}

.form-group label {
  display: block;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  margin-bottom: var(--spacing-xs);
}

.form-input {
  width: 100%;
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-background);
  color: var(--color-text);
  font-size: 0.875rem;
}

.form-input:focus {
  outline: none;
  border-color: var(--color-primary-500);
}

.loading-state {
  text-align: center;
  padding: var(--spacing-lg);
  color: var(--color-text-muted);
  font-size: 0.875rem;
}

.results-list {
  max-height: 250px;
  overflow-y: auto;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  margin-bottom: var(--spacing-md);
}

.result-item {
  display: block;
  width: 100%;
  padding: var(--spacing-sm) var(--spacing-md);
  border: none;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-background);
  text-align: left;
  cursor: pointer;
  transition: background var(--transition-fast);
}

.result-item:last-child {
  border-bottom: none;
}

.result-item:hover {
  background: var(--color-base-200);
}

.result-item.selected {
  background: var(--color-primary-100);
  border-color: var(--color-primary-500);
}

.result-main {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2px;
}

.result-name {
  font-weight: 500;
  color: var(--color-text);
}

.result-cr {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  background: var(--color-base-200);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
}

.result-meta {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.result-size {
  margin-right: var(--spacing-sm);
}

.token-options {
  border-top: 1px solid var(--color-border);
  padding-top: var(--spacing-md);
  margin-top: var(--spacing-md);
}

.selected-monster {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--color-primary-100);
  border: 1px solid var(--color-primary-500);
  border-radius: var(--radius-md);
  padding: var(--spacing-sm) var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

.selected-name {
  font-weight: 600;
  color: var(--color-primary-700);
}

.clear-btn {
  width: 20px;
  height: 20px;
  border: none;
  background: none;
  color: var(--color-primary-700);
  font-size: 1rem;
  cursor: pointer;
  border-radius: var(--radius-sm);
}

.clear-btn:hover {
  background: var(--color-primary-200);
}

.options-row {
  display: flex;
  gap: var(--spacing-md);
}

.options-row .form-group:first-child {
  flex: 1;
}

/* Toggle switch */
.toggle {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
  cursor: pointer;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  inset: 0;
  background: var(--color-base-300);
  border-radius: 12px;
  transition: background var(--transition-fast);
}

.toggle-slider::before {
  content: '';
  position: absolute;
  width: 18px;
  height: 18px;
  left: 3px;
  bottom: 3px;
  background: white;
  border-radius: 50%;
  transition: transform var(--transition-fast);
}

.toggle input:checked + .toggle-slider {
  background: var(--color-primary-500);
}

.toggle input:checked + .toggle-slider::before {
  transform: translateX(20px);
}

/* Vision Section */
.vision-section {
  margin-top: var(--spacing-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.vision-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  width: 100%;
  padding: var(--spacing-sm) var(--spacing-md);
  border: none;
  background: var(--color-base-100);
  cursor: pointer;
  text-align: left;
}

.vision-header:hover {
  background: var(--color-base-200);
}

.vision-title {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
}

.vision-summary {
  flex: 1;
  font-size: 0.875rem;
  color: var(--color-text);
}

.vision-toggle {
  font-size: 0.625rem;
  color: var(--color-text-muted);
}

.vision-details {
  padding: var(--spacing-md);
  border-top: 1px solid var(--color-border);
}

.vision-presets {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-xs);
  margin-bottom: var(--spacing-md);
}

.preset-btn {
  padding: var(--spacing-xs) var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-background);
  color: var(--color-text);
  font-size: 0.75rem;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.preset-btn:hover {
  background: var(--color-base-200);
  border-color: var(--color-primary-500);
}

.preset-btn.active {
  background: var(--color-primary-100);
  border-color: var(--color-primary-500);
  color: var(--color-primary-700);
}

.vision-custom {
  border-top: 1px solid var(--color-border);
  padding-top: var(--spacing-md);
}

.vision-custom .form-group {
  margin-bottom: 0;
}

.vision-custom .form-input {
  max-width: 100px;
}
</style>
