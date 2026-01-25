<template>
  <div class="module-monsters">
    <div class="monsters-header">
      <h3>Module Monsters</h3>
      <span class="monster-count" v-if="moduleMonsters.length > 0">
        {{ moduleMonsters.length }} tagged
      </span>
    </div>

    <!-- Search Section -->
    <div class="search-section">
      <div class="search-input-wrapper">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search monsters..."
          class="search-input"
          @input="debouncedSearch"
        />
        <button
          v-if="searchQuery"
          class="clear-search"
          @click="clearSearch"
        >
          &times;
        </button>
      </div>

      <!-- Search Results -->
      <div v-if="searchResults.length > 0" class="search-results">
        <div
          v-for="monster in searchResults"
          :key="`${monster.name}-${monster.source}`"
          class="search-result-item"
        >
          <div class="monster-info">
            <span class="monster-name">{{ monster.name }}</span>
            <span class="monster-meta">
              CR {{ monster.cr }} | {{ monster.creature_type }} | {{ monster.source }}
            </span>
          </div>
          <button
            class="add-button"
            @click="addMonster(monster)"
            :disabled="isMonsterAdded(monster)"
          >
            {{ isMonsterAdded(monster) ? 'Added' : 'Add' }}
          </button>
        </div>
      </div>

      <!-- Search Loading -->
      <div v-if="isSearching" class="search-loading">
        Searching...
      </div>
    </div>

    <!-- Tagged Monsters List -->
    <div class="tagged-monsters" v-if="moduleMonsters.length > 0">
      <h4>Tagged Monsters</h4>

      <div
        v-for="monster in moduleMonsters"
        :key="monster.id"
        class="tagged-monster-item"
        @click="viewMonster(monster)"
      >
        <div class="monster-details">
          <span class="monster-name clickable">{{ monster.display_name || monster.monster_name }}</span>
          <span class="monster-source">{{ monster.monster_source }}</span>
        </div>

        <div class="monster-controls" @click.stop>
          <div class="quantity-control">
            <label>Qty:</label>
            <input
              type="number"
              :value="monster.quantity"
              min="1"
              class="quantity-input"
              @change="updateQuantity(monster, $event)"
            />
          </div>

          <div class="tag-control">
            <input
              type="text"
              :value="monster.encounter_tag || ''"
              placeholder="Encounter tag..."
              class="tag-input"
              @change="updateTag(monster, $event)"
            />
          </div>

          <button
            class="remove-button"
            @click="removeMonster(monster)"
            title="Remove monster"
          >
            &times;
          </button>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <EmptyState
      v-else
      variant="search"
      title="No monsters tagged yet"
      description="Search above to add monsters to this module."
    />

    <!-- Monster Detail Modal -->
    <AppModal
      :visible="showMonsterModal"
      :title="selectedMonster?.display_name || selectedMonster?.monster_name || 'Monster'"
      size="md"
      @close="closeMonsterModal"
    >
      <div
        v-if="monsterDetailContent"
        class="dnd-content"
        v-html="monsterDetailContent"
      ></div>
      <div v-else class="loading-content">
        Loading monster details...
      </div>
    </AppModal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useMonsters, type MonsterSummary } from '@/features/sources/composables/catalog/useMonsters'
import { formatMonsterDetails } from '@/features/sources/formatters/monsterFormatterEnhanced'
import EmptyState from '@/shared/components/ui/EmptyState.vue'
import AppModal from '@/components/shared/AppModal.vue'

interface ModuleMonster {
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
  /** Full monster data from the catalog (when loaded with _with_data) */
  monster_data?: any
  created_at: string
  updated_at: string
}

interface Props {
  moduleId: string
  moduleName: string
  moduleNumber: number
  campaignId: string
}

const props = defineProps<Props>()

const { searchMonsters } = useMonsters()

const searchQuery = ref('')
const searchResults = ref<MonsterSummary[]>([])
const isSearching = ref(false)
const moduleMonsters = ref<ModuleMonster[]>([])

// Monster detail modal state
const showMonsterModal = ref(false)
const selectedMonster = ref<ModuleMonster | null>(null)
const monsterDetailContent = ref<string>('')

let searchTimeout: ReturnType<typeof setTimeout> | null = null

// Load existing module monsters with full data
async function loadModuleMonsters() {
  try {
    const response = await invoke<{ data: ModuleMonster[] }>('list_module_monsters_with_data', {
      moduleId: props.moduleId
    })
    moduleMonsters.value = response.data || []
  } catch (error) {
    console.error('Failed to load module monsters:', error)
  }
}

// View monster details in modal
async function viewMonster(monster: ModuleMonster) {
  selectedMonster.value = monster
  showMonsterModal.value = true
  monsterDetailContent.value = ''

  try {
    // If we have monster_data, format it directly
    if (monster.monster_data) {
      monsterDetailContent.value = await formatMonsterDetails(monster.monster_data)
    } else {
      // Fetch the full monster data from catalog
      const response = await invoke<{ success: boolean; data?: any }>('get_monster_by_name', {
        name: monster.monster_name,
        source: monster.monster_source
      })
      if (response.success && response.data) {
        monsterDetailContent.value = await formatMonsterDetails(response.data)
      } else {
        monsterDetailContent.value = `<p>Monster data not found for ${monster.monster_name} (${monster.monster_source})</p>`
      }
    }
  } catch (error) {
    console.error('Failed to load monster details:', error)
    monsterDetailContent.value = `<p>Error loading monster details</p>`
  }
}

// Close monster modal
function closeMonsterModal() {
  showMonsterModal.value = false
  selectedMonster.value = null
  monsterDetailContent.value = ''
}

// Debounced search
function debouncedSearch() {
  if (searchTimeout) {
    clearTimeout(searchTimeout)
  }

  searchTimeout = setTimeout(async () => {
    if (searchQuery.value.length < 2) {
      searchResults.value = []
      return
    }

    isSearching.value = true
    try {
      console.log('Searching for:', searchQuery.value)
      const results = await searchMonsters({ query: searchQuery.value })
      console.log('Search results:', results)
      searchResults.value = results.slice(0, 10) // Limit results
    } catch (error) {
      console.error('Search failed:', error)
    } finally {
      isSearching.value = false
    }
  }, 300)
}

// Clear search
function clearSearch() {
  searchQuery.value = ''
  searchResults.value = []
}

// Check if monster is already added
function isMonsterAdded(monster: MonsterSummary): boolean {
  return moduleMonsters.value.some(
    m => m.monster_name === monster.name && m.monster_source === monster.source
  )
}

// Sync monsters to file
async function syncMonstersToFile() {
  try {
    // Get campaign directory
    const campaignResponse = await invoke<{ data: { directory_path: string } }>('get_campaign', {
      id: props.campaignId
    })

    if (!campaignResponse.data?.directory_path) {
      console.error('Could not get campaign directory')
      return
    }

    await invoke('sync_module_monsters_to_file', {
      request: {
        module_id: props.moduleId,
        campaign_directory: campaignResponse.data.directory_path,
        module_number: props.moduleNumber,
        module_name: props.moduleName
      }
    })
  } catch (error) {
    console.error('Failed to sync monsters to file:', error)
  }
}

// Add monster to module
async function addMonster(monster: MonsterSummary) {
  try {
    const response = await invoke<{ data: ModuleMonster }>('add_module_monster', {
      request: {
        module_id: props.moduleId,
        monster_name: monster.name,
        monster_source: monster.source,
        quantity: 1,
        encounter_tag: null
      }
    })

    if (response.data) {
      moduleMonsters.value.push(response.data)
      await syncMonstersToFile()
    }
  } catch (error) {
    console.error('Failed to add monster:', error)
  }
}

// Update monster quantity
async function updateQuantity(monster: ModuleMonster, event: Event) {
  const input = event.target as HTMLInputElement
  const quantity = parseInt(input.value) || 1

  try {
    await invoke('update_module_monster', {
      monsterId: monster.id,
      request: { quantity, encounter_tag: null }
    })
    monster.quantity = quantity
    await syncMonstersToFile()
  } catch (error) {
    console.error('Failed to update quantity:', error)
  }
}

// Update monster tag
async function updateTag(monster: ModuleMonster, event: Event) {
  const input = event.target as HTMLInputElement
  const tag = input.value.trim() || null

  try {
    await invoke('update_module_monster', {
      monsterId: monster.id,
      request: { quantity: null, encounter_tag: tag ? { Some: tag } : { None: null } }
    })
    monster.encounter_tag = tag
    await syncMonstersToFile()
  } catch (error) {
    console.error('Failed to update tag:', error)
  }
}

// Remove monster from module
async function removeMonster(monster: ModuleMonster) {
  try {
    await invoke('remove_module_monster', {
      monsterId: monster.id
    })
    moduleMonsters.value = moduleMonsters.value.filter(m => m.id !== monster.id)
    await syncMonstersToFile()
  } catch (error) {
    console.error('Failed to remove monster:', error)
  }
}

onMounted(() => {
  loadModuleMonsters()
})
</script>

<style scoped>
.module-monsters {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 0.5rem;
  padding: 1rem;
}

.monsters-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.monsters-header h3 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
}

.monster-count {
  font-size: 0.875rem;
  color: var(--color-text-muted);
  background: var(--color-base-200);
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
}

/* Search Section */
.search-section {
  margin-bottom: 1rem;
}

.search-input-wrapper {
  position: relative;
}

.search-input {
  width: 100%;
  padding: 0.5rem 2rem 0.5rem 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  font-size: 0.875rem;
  background: var(--color-base-100);
  color: var(--color-text);
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.clear-search {
  position: absolute;
  right: 0.5rem;
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  font-size: 1.25rem;
  line-height: 1;
}

.clear-search:hover {
  color: var(--color-text);
}

/* Search Results */
.search-results {
  margin-top: 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  max-height: 300px;
  overflow-y: auto;
}

.search-result-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--color-border);
}

.search-result-item:last-child {
  border-bottom: none;
}

.search-result-item:hover {
  background: var(--color-base-200);
}

.monster-info {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.monster-name {
  font-weight: 500;
}

.monster-meta {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.add-button {
  padding: 0.25rem 0.75rem;
  background: var(--color-primary);
  color: white;
  border: none;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  cursor: pointer;
}

.add-button:hover:not(:disabled) {
  background: var(--color-primary-dark);
}

.add-button:disabled {
  background: var(--color-base-300);
  color: var(--color-text-muted);
  cursor: not-allowed;
}

.search-loading {
  padding: 0.5rem;
  text-align: center;
  color: var(--color-text-muted);
  font-style: italic;
  font-size: 0.875rem;
}

/* Tagged Monsters */
.tagged-monsters h4 {
  margin: 0 0 0.75rem 0;
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.tagged-monster-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem;
  background: var(--color-base-100);
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  margin-bottom: 0.5rem;
}

.monster-details {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.monster-details .monster-name {
  font-weight: 500;
}

.monster-source {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.monster-controls {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.quantity-control,
.tag-control {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.quantity-control label {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.quantity-input {
  width: 3rem;
  padding: 0.25rem;
  border: 1px solid var(--color-border);
  border-radius: 0.25rem;
  font-size: 0.75rem;
  text-align: center;
  background: var(--color-base-100);
  color: var(--color-text);
}

.quantity-input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.tag-input {
  width: 8rem;
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 0.25rem;
  font-size: 0.75rem;
  background: var(--color-base-100);
  color: var(--color-text);
}

.tag-input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.tag-input::placeholder {
  color: var(--color-text-muted);
}

.remove-button {
  background: none;
  border: none;
  color: var(--color-error);
  cursor: pointer;
  font-size: 1.25rem;
  line-height: 1;
  padding: 0.25rem;
}

.remove-button:hover {
  color: var(--color-error-dark);
}

/* Clickable monster card */
.tagged-monster-item {
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.tagged-monster-item:hover {
  background: var(--color-base-200);
}

.monster-name.clickable {
  color: var(--color-primary);
  text-decoration: underline;
  text-decoration-style: dotted;
}

.monster-name.clickable:hover {
  text-decoration-style: solid;
}

/* Modal content */
.loading-content {
  padding: 2rem;
  text-align: center;
  color: var(--color-text-muted);
}

.dnd-content {
  font-size: 0.9rem;
  line-height: 1.6;
}

.dnd-content :deep(h1),
.dnd-content :deep(h2),
.dnd-content :deep(h3) {
  margin-top: 1rem;
  margin-bottom: 0.5rem;
}

.dnd-content :deep(p) {
  margin-bottom: 0.5rem;
}

.dnd-content :deep(.stat-block) {
  border: 1px solid var(--color-border);
  border-radius: 0.5rem;
  padding: 1rem;
  background: var(--color-base-100);
}

.dnd-content :deep(.ability-scores) {
  display: flex;
  justify-content: space-between;
  text-align: center;
  margin: 1rem 0;
  padding: 0.5rem;
  background: var(--color-base-200);
  border-radius: 0.25rem;
}

.dnd-content :deep(.dice-roll),
.dnd-content :deep(.damage-roll) {
  font-family: monospace;
  font-weight: 700;
  color: var(--color-dnd-damage, #ff6b6b);
}

.dnd-content :deep(.hit-bonus) {
  font-weight: 700;
  color: var(--color-success, #34d399);
}

</style>
