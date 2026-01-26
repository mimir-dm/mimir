<template>
  <div
    v-if="visible"
    class="vision-menu"
    :style="{ left: x + 'px', top: y + 'px' }"
    @click.stop
  >
    <div class="menu-header">
      <span class="token-name">{{ token.name }}</span>
      <button class="close-btn" @click="$emit('close')">&times;</button>
    </div>

    <!-- Presets -->
    <div class="menu-section">
      <label class="section-label">Presets</label>
      <select class="preset-select" v-model="selectedPreset" @change="handlePresetChange">
        <option value="">Choose preset...</option>
        <option v-for="(preset, key) in VISION_PRESETS" :key="key" :value="key">
          {{ preset.label }}
        </option>
      </select>
    </div>

    <!-- Custom Values -->
    <div class="menu-section">
      <label class="section-label">Vision Range (ft)</label>
      <div class="input-row">
        <label>Bright:</label>
        <input
          type="number"
          :value="token.vision_bright_ft ?? ''"
          placeholder="∞"
          min="0"
          step="5"
          @change="updateBright($event)"
        >
      </div>
      <div class="input-row">
        <label>Dim:</label>
        <input
          type="number"
          :value="token.vision_dim_ft ?? ''"
          placeholder="∞"
          min="0"
          step="5"
          @change="updateDim($event)"
        >
      </div>
      <div class="input-row">
        <label>Dark:</label>
        <input
          type="number"
          :value="token.vision_dark_ft"
          min="0"
          step="5"
          @change="updateDark($event)"
        >
      </div>
    </div>

    <div class="menu-section">
      <label class="section-label">Light Source (ft)</label>
      <div class="input-row">
        <label>Radius:</label>
        <input
          type="number"
          :value="token.light_radius_ft"
          min="0"
          step="5"
          @change="updateLight($event)"
        >
      </div>
      <div class="light-presets">
        <button class="light-btn" @click="setLightRadius(0)">None</button>
        <button class="light-btn" @click="setLightRadius(40)">Torch</button>
        <button class="light-btn" @click="setLightRadius(60)">Lantern</button>
      </div>
    </div>

    <div class="menu-footer">
      <button class="reset-btn" @click="handleReset" :disabled="saving">
        {{ saving ? 'Saving...' : 'Reset' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { Token } from '@/types/api'
import { useTokenVision, VISION_PRESETS, type VisionPresetKey } from '@/composables/useTokenVision'

interface Props {
  visible: boolean
  token: Token
  x: number
  y: number
}

const props = defineProps<Props>()
const emit = defineEmits<{
  close: []
  updated: [token: Token]
}>()

const { updateVisionSettings, applyPreset, resetToDefault, findMatchingPreset } = useTokenVision()

const saving = ref(false)
const selectedPreset = ref<string>('')

// Update selected preset when token changes
watch(() => props.token, (token) => {
  const match = findMatchingPreset(token)
  selectedPreset.value = match || ''
}, { immediate: true })

async function handlePresetChange() {
  if (!selectedPreset.value) return

  saving.value = true
  try {
    const updated = await applyPreset(props.token.id, selectedPreset.value as VisionPresetKey)
    if (updated) {
      emit('updated', updated)
    }
  } finally {
    saving.value = false
  }
}

async function updateField(field: string, value: number | null) {
  saving.value = true
  try {
    const settings = {
      vision_bright_ft: props.token.vision_bright_ft,
      vision_dim_ft: props.token.vision_dim_ft,
      vision_dark_ft: props.token.vision_dark_ft,
      light_radius_ft: props.token.light_radius_ft,
      [field]: value
    }
    const updated = await updateVisionSettings(props.token.id, settings)
    if (updated) {
      emit('updated', updated)
      // Clear preset selection since we have custom values now
      selectedPreset.value = findMatchingPreset(updated) || ''
    }
  } finally {
    saving.value = false
  }
}

function updateBright(event: Event) {
  const input = event.target as HTMLInputElement
  const value = input.value === '' ? null : parseInt(input.value, 10)
  updateField('vision_bright_ft', value)
}

function updateDim(event: Event) {
  const input = event.target as HTMLInputElement
  const value = input.value === '' ? null : parseInt(input.value, 10)
  updateField('vision_dim_ft', value)
}

function updateDark(event: Event) {
  const input = event.target as HTMLInputElement
  const value = parseInt(input.value, 10) || 0
  updateField('vision_dark_ft', value)
}

function updateLight(event: Event) {
  const input = event.target as HTMLInputElement
  const value = parseInt(input.value, 10) || 0
  updateField('light_radius_ft', value)
}

function setLightRadius(radius: number) {
  updateField('light_radius_ft', radius)
}

async function handleReset() {
  saving.value = true
  try {
    const updated = await resetToDefault(props.token.id)
    if (updated) {
      emit('updated', updated)
      selectedPreset.value = 'human'
    }
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.vision-menu {
  position: fixed;
  background: var(--color-bg-secondary, #1e1e1e);
  border: 1px solid var(--color-border, #333);
  border-radius: 8px;
  padding: 12px;
  min-width: 200px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  z-index: 1000;
  font-size: 13px;
}

.menu-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--color-border, #333);
}

.token-name {
  font-weight: 600;
  color: var(--color-text-primary, #fff);
}

.close-btn {
  background: none;
  border: none;
  color: var(--color-text-secondary, #888);
  font-size: 18px;
  cursor: pointer;
  padding: 0 4px;
}

.close-btn:hover {
  color: var(--color-text-primary, #fff);
}

.menu-section {
  margin-bottom: 12px;
}

.section-label {
  display: block;
  font-size: 11px;
  text-transform: uppercase;
  color: var(--color-text-secondary, #888);
  margin-bottom: 6px;
}

.preset-select {
  width: 100%;
  padding: 6px 8px;
  background: var(--color-bg-tertiary, #2a2a2a);
  border: 1px solid var(--color-border, #333);
  border-radius: 4px;
  color: var(--color-text-primary, #fff);
  font-size: 13px;
}

.input-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.input-row label {
  width: 50px;
  color: var(--color-text-secondary, #aaa);
}

.input-row input {
  flex: 1;
  padding: 4px 8px;
  background: var(--color-bg-tertiary, #2a2a2a);
  border: 1px solid var(--color-border, #333);
  border-radius: 4px;
  color: var(--color-text-primary, #fff);
  font-size: 13px;
  width: 60px;
}

.input-row input::placeholder {
  color: var(--color-text-secondary, #666);
}

.light-presets {
  display: flex;
  gap: 4px;
  margin-top: 6px;
}

.light-btn {
  flex: 1;
  padding: 4px 8px;
  background: var(--color-bg-tertiary, #2a2a2a);
  border: 1px solid var(--color-border, #333);
  border-radius: 4px;
  color: var(--color-text-secondary, #aaa);
  font-size: 11px;
  cursor: pointer;
}

.light-btn:hover {
  background: var(--color-bg-hover, #3a3a3a);
  color: var(--color-text-primary, #fff);
}

.menu-footer {
  margin-top: 12px;
  padding-top: 8px;
  border-top: 1px solid var(--color-border, #333);
}

.reset-btn {
  width: 100%;
  padding: 6px 12px;
  background: transparent;
  border: 1px solid var(--color-border, #444);
  border-radius: 4px;
  color: var(--color-text-secondary, #888);
  font-size: 12px;
  cursor: pointer;
}

.reset-btn:hover:not(:disabled) {
  background: var(--color-bg-hover, #3a3a3a);
  color: var(--color-text-primary, #fff);
}

.reset-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
