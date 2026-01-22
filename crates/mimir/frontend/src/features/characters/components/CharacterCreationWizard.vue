<template>
  <AppModal
    :visible="visible"
    :title="isNpc ? 'Create NPC' : 'Create Character'"
    size="md"
    @close="handleClose"
  >
    <div class="wizard-body">
      <!-- Character Type Selection -->
      <div class="form-group">
        <label class="form-label">Character Type</label>
        <div class="type-buttons">
          <button
            type="button"
            class="type-button"
            :class="{ active: !isNpc }"
            @click="isNpc = false"
          >
            Player Character
          </button>
          <button
            type="button"
            class="type-button"
            :class="{ active: isNpc }"
            @click="isNpc = true"
          >
            NPC
          </button>
        </div>
      </div>

      <!-- Name -->
      <div class="form-group">
        <label class="form-label" for="name">Name *</label>
        <input
          id="name"
          v-model="formData.name"
          type="text"
          class="form-input"
          placeholder="Character name"
          required
        />
      </div>

      <!-- Player Name (PC only) -->
      <div v-if="!isNpc" class="form-group">
        <label class="form-label" for="player_name">Player Name *</label>
        <input
          id="player_name"
          v-model="formData.player_name"
          type="text"
          class="form-input"
          placeholder="Player's name"
          required
        />
      </div>

      <!-- Race -->
      <div class="form-group">
        <label class="form-label" for="race">Race</label>
        <input
          id="race"
          v-model="formData.race_name"
          type="text"
          class="form-input"
          placeholder="e.g., Human, Elf, Dwarf"
        />
      </div>

      <!-- NPC-specific fields -->
      <template v-if="isNpc">
        <div class="form-group">
          <label class="form-label" for="role">Role</label>
          <input
            id="role"
            v-model="formData.role"
            type="text"
            class="form-input"
            placeholder="e.g., Merchant, Guard, Wizard"
          />
        </div>

        <div class="form-group">
          <label class="form-label" for="location">Location</label>
          <input
            id="location"
            v-model="formData.location"
            type="text"
            class="form-input"
            placeholder="e.g., Tavern, Castle, Forest"
          />
        </div>

        <div class="form-group">
          <label class="form-label" for="faction">Faction</label>
          <input
            id="faction"
            v-model="formData.faction"
            type="text"
            class="form-input"
            placeholder="e.g., Thieves Guild, Royal Guard"
          />
        </div>
      </template>

      <!-- PC-specific fields -->
      <template v-if="!isNpc">
        <div class="form-group">
          <label class="form-label" for="background">Background</label>
          <input
            id="background"
            v-model="formData.background_name"
            type="text"
            class="form-input"
            placeholder="e.g., Acolyte, Criminal, Noble"
          />
        </div>
      </template>

      <!-- Error display -->
      <div v-if="error" class="error-message">
        {{ error }}
      </div>
    </div>

    <template #footer>
      <button type="button" @click="handleClose" class="btn btn-secondary">
        Cancel
      </button>
      <button
        type="button"
        @click="createCharacter"
        class="btn btn-primary"
        :disabled="!canCreate || creating"
      >
        {{ creating ? 'Creating...' : 'Create Character' }}
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import AppModal from '@/components/shared/AppModal.vue'
import { useCharacterStore } from '../../../stores/characters'

const props = defineProps<{
  visible: boolean
  campaignId?: string
  startAsNpc?: boolean
}>()

const emit = defineEmits<{
  close: []
  created: [characterId: string]
}>()

const characterStore = useCharacterStore()

const isNpc = ref(props.startAsNpc ?? false)
const creating = ref(false)
const error = ref<string | null>(null)

const formData = ref({
  name: '',
  player_name: '',
  race_name: '',
  background_name: '',
  role: '',
  location: '',
  faction: ''
})

const canCreate = computed(() => {
  if (!formData.value.name.trim()) return false
  if (!isNpc.value && !formData.value.player_name.trim()) return false
  return true
})

const handleClose = () => {
  if (!creating.value) {
    resetForm()
    emit('close')
  }
}

const resetForm = () => {
  isNpc.value = props.startAsNpc ?? false
  formData.value = {
    name: '',
    player_name: '',
    race_name: '',
    background_name: '',
    role: '',
    location: '',
    faction: ''
  }
  error.value = null
}

const createCharacter = async () => {
  if (!canCreate.value) return

  const campaignId = props.campaignId
  if (!campaignId) {
    error.value = 'Campaign ID is required'
    return
  }

  creating.value = true
  error.value = null

  try {
    let character
    if (isNpc.value) {
      character = await characterStore.createNpc({
        campaign_id: campaignId,
        name: formData.value.name.trim(),
        race_name: formData.value.race_name.trim() || undefined,
        role: formData.value.role.trim() || undefined,
        location: formData.value.location.trim() || undefined,
        faction: formData.value.faction.trim() || undefined
      })
    } else {
      character = await characterStore.createPc({
        campaign_id: campaignId,
        name: formData.value.name.trim(),
        player_name: formData.value.player_name.trim(),
        race_name: formData.value.race_name.trim() || undefined,
        background_name: formData.value.background_name.trim() || undefined
      })
    }

    if (character) {
      emit('created', character.id)
      resetForm()
      emit('close')
    } else {
      error.value = characterStore.error || 'Failed to create character'
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to create character'
  } finally {
    creating.value = false
  }
}
</script>

<style scoped>
.wizard-body {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.form-label {
  font-weight: 500;
  font-size: 0.875rem;
  color: var(--color-text);
}

.form-input {
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 1rem;
}

.form-input:focus {
  outline: none;
  border-color: var(--color-primary-500);
  box-shadow: 0 0 0 2px var(--color-primary-100);
}

.type-buttons {
  display: flex;
  gap: var(--spacing-sm);
}

.type-button {
  flex: 1;
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-text);
  cursor: pointer;
  transition: all var(--transition-base);
}

.type-button:hover {
  background: var(--color-surface-variant);
}

.type-button.active {
  background: var(--color-primary-500);
  color: white;
  border-color: var(--color-primary-500);
}

.error-message {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-error-bg, #fef2f2);
  color: var(--color-error, #dc2626);
  border-radius: var(--radius-md);
  font-size: 0.875rem;
}
</style>
