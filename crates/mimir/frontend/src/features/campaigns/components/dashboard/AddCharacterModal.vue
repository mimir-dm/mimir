<template>
  <AppModal
    :visible="visible"
    :title="isNpc ? 'Add NPC to Campaign' : 'Add PC to Campaign'"
    size="md"
    @close="$emit('close')"
  >
    <div class="add-character-modal">
      <!-- Loading -->
      <div v-if="loading" class="loading-state">
        Loading characters...
      </div>

      <!-- Empty -->
      <div v-else-if="availableCharacters.length === 0" class="empty-state">
        <p>No {{ isNpc ? 'NPCs' : 'player characters' }} available to add.</p>
        <p class="hint">Characters already in this campaign or without a matching type won't appear here.</p>
      </div>

      <!-- Character List -->
      <div v-else class="character-list">
        <div
          v-for="character in availableCharacters"
          :key="character.id"
          class="character-option"
          :class="{ selected: selectedId === character.id }"
          @click="selectedId = character.id"
        >
          <div class="character-info">
            <span class="character-name">{{ character.character_name }}</span>
            <span class="character-details">
              Level {{ character.current_level }} {{ character.race || '' }} {{ character.class || '' }}
            </span>
          </div>
          <span v-if="character.campaign_id" class="current-campaign">
            Currently in another campaign
          </span>
          <span v-else class="no-campaign">
            Unassigned
          </span>
        </div>
      </div>
    </div>

    <template #footer>
      <button @click="$emit('close')" class="btn btn-secondary">Cancel</button>
      <button
        @click="addCharacter"
        class="btn btn-primary"
        :disabled="!selectedId || adding"
      >
        {{ adding ? 'Adding...' : 'Add to Campaign' }}
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'
import { useCharacterStore } from '@/stores/characters'
import type { Character } from '@/types/character'

const props = defineProps<{
  visible: boolean
  campaignId: number
  isNpc: boolean
}>()

const emit = defineEmits<{
  close: []
  added: []
}>()

const characterStore = useCharacterStore()

const loading = ref(false)
const adding = ref(false)
const selectedId = ref<number | null>(null)

// Filter to characters that match NPC/PC type and aren't already in this campaign
const availableCharacters = computed(() => {
  return characterStore.characters.filter(c => {
    return c.is_npc === props.isNpc && c.campaign_id !== props.campaignId
  })
})

// Load all characters when modal opens
watch(() => props.visible, async (isVisible) => {
  if (isVisible) {
    loading.value = true
    selectedId.value = null
    try {
      await characterStore.fetchAllCharacters()
    } finally {
      loading.value = false
    }
  }
})

async function addCharacter() {
  if (!selectedId.value) return

  adding.value = true
  try {
    await invoke('assign_character_to_campaign', {
      characterId: selectedId.value,
      campaignId: props.campaignId
    })
    emit('added')
  } catch (e) {
    console.error('Failed to add character to campaign:', e)
  } finally {
    adding.value = false
  }
}
</script>

<style scoped>
.add-character-modal {
  min-height: 200px;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-xl, 24px);
  text-align: center;
  color: var(--color-text-muted, #888);
}

.hint {
  font-size: 0.75rem;
  margin-top: var(--spacing-sm, 8px);
}

.character-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs, 4px);
  max-height: 400px;
  overflow-y: auto;
}

.character-option {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-sm, 8px) var(--spacing-md, 12px);
  background: var(--color-surface, #1a1a1a);
  border: 1px solid var(--color-border, #333);
  border-radius: var(--radius-md, 8px);
  cursor: pointer;
  transition: all 0.15s;
}

.character-option:hover {
  border-color: var(--color-primary, #4a9eff);
}

.character-option.selected {
  border-color: var(--color-primary, #4a9eff);
  background: var(--color-primary-900, #1e3a5f);
}

.character-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.character-name {
  font-weight: 600;
  color: var(--color-text, #e0e0e0);
}

.character-details {
  font-size: 0.75rem;
  color: var(--color-primary, #4a9eff);
}

.current-campaign,
.no-campaign {
  font-size: 0.75rem;
  color: var(--color-text-muted, #888);
}

.no-campaign {
  color: var(--color-success, #10b981);
}
</style>
