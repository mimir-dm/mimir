<template>
  <AppModal
    :visible="visible"
    title="Export Character"
    size="md"
    @close="handleClose"
  >
    <div class="export-dialog">
      <!-- Character Info -->
      <div class="character-info" v-if="characterName">
        <h3 class="character-name">{{ characterName }}</h3>
      </div>

      <!-- Character Sheets Section -->
      <div class="option-section">
        <label class="section-label">Character Sheets</label>
        <div class="checkbox-group">
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includeCompactSheet" />
            <span class="checkbox-label">Compact Sheet (2-page)</span>
            <span class="checkbox-desc">Stats, combat, skills, equipment summary</span>
          </label>
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includeLongForm" />
            <span class="checkbox-label">Long Form</span>
            <span class="checkbox-desc">Appearance, personality, backstory, RP notes</span>
          </label>
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includeBattleCard" />
            <span class="checkbox-label">Battle Card</span>
            <span class="checkbox-desc">Half-page combat reference (AC, HP, attacks, saves)</span>
          </label>
        </div>
      </div>

      <!-- Cards Section -->
      <div class="option-section">
        <label class="section-label">Cards</label>
        <div class="checkbox-group">
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includeSpellCards" />
            <span class="checkbox-label">Spell Cards</span>
            <span class="checkbox-desc">Printable cards for all spells (if caster)</span>
          </label>
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includeEquipmentCards" />
            <span class="checkbox-label">Equipment Cards</span>
            <span class="checkbox-desc">Cards for weapons, magic items, special ammo</span>
          </label>
        </div>
      </div>

      <!-- Detailed Sections -->
      <div class="option-section">
        <label class="section-label">Detailed Sections</label>
        <div class="checkbox-group">
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includeEquipmentDetail" />
            <span class="checkbox-label">Equipment Detail</span>
            <span class="checkbox-desc">Full inventory with descriptions and special rules</span>
          </label>
        </div>
      </div>

      <!-- Validation Warning -->
      <div v-if="!hasAnySelection" class="warning-message">
        Select at least one section to export.
      </div>

      <!-- Error Message -->
      <div v-if="error" class="error-message">
        {{ error }}
      </div>
    </div>

    <template #footer>
      <button
        @click="handleClose"
        class="btn btn-secondary"
        :disabled="isLoading"
      >
        Cancel
      </button>
      <button
        @click="handleExport"
        class="btn btn-primary"
        :disabled="isLoading || !characterId || !hasAnySelection"
      >
        <span v-if="isLoading" class="spinner-sm"></span>
        {{ isLoading ? 'Generating...' : 'Export PDF' }}
      </button>
    </template>
  </AppModal>

  <!-- PDF Preview Modal -->
  <PdfPreviewModal
    ref="pdfPreviewRef"
    :visible="showPreview"
    :title="`Character: ${characterName}`"
    :default-file-name="defaultFileName"
    @close="showPreview = false"
    @retry="handleExport"
  />
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import AppModal from '@/components/shared/AppModal.vue'
import PdfPreviewModal from './PdfPreviewModal.vue'
import { PrintService } from '../../services/PrintService'

interface Props {
  visible: boolean
  characterId: string | null
  characterName?: string
}

const props = withDefaults(defineProps<Props>(), {
  characterName: ''
})

const emit = defineEmits<{
  close: []
}>()

// State
const isLoading = ref(false)
const error = ref<string | null>(null)
const showPreview = ref(false)
const pdfPreviewRef = ref<InstanceType<typeof PdfPreviewModal> | null>(null)

// Options - defaults per wireframe spec
const options = reactive({
  includeCompactSheet: true,
  includeLongForm: false,
  includeBattleCard: false,
  includeSpellCards: true,
  includeEquipmentCards: false,
  includeEquipmentDetail: false,
})

// Computed
const hasAnySelection = computed(() => {
  return options.includeCompactSheet || options.includeLongForm ||
         options.includeBattleCard || options.includeSpellCards ||
         options.includeEquipmentCards || options.includeEquipmentDetail
})

const defaultFileName = computed(() => {
  const name = props.characterName || 'character'
  const safeName = name.replace(/[^a-z0-9\s\-_.]/gi, '').replace(/\s+/g, '_')
  return `${safeName}.pdf`
})

// Reset options when dialog opens
watch(() => props.visible, (newVisible) => {
  if (newVisible) {
    error.value = null
    // Reset to defaults
    options.includeCompactSheet = true
    options.includeLongForm = false
    options.includeBattleCard = false
    options.includeSpellCards = true
    options.includeEquipmentCards = false
    options.includeEquipmentDetail = false
  }
})

function handleClose() {
  if (!isLoading.value) {
    emit('close')
  }
}

async function handleExport() {
  if (!props.characterId || !hasAnySelection.value) {
    error.value = 'Select at least one section to export'
    return
  }

  isLoading.value = true
  error.value = null

  try {
    // Show preview modal and set loading state
    showPreview.value = true
    pdfPreviewRef.value?.setLoading(true)

    // Generate PDF with options
    const result = await PrintService.generateCharacterExport(props.characterId, {
      include_compact_sheet: options.includeCompactSheet,
      include_long_form: options.includeLongForm,
      include_battle_card: options.includeBattleCard,
      include_spell_cards: options.includeSpellCards,
      include_equipment_cards: options.includeEquipmentCards,
      include_equipment_detail: options.includeEquipmentDetail,
    })

    // Display result
    pdfPreviewRef.value?.setPdfResult(result)

    // Close this dialog
    emit('close')
  } catch (err) {
    const errorMessage = err instanceof Error ? err.message : 'Failed to generate PDF'
    error.value = errorMessage
    pdfPreviewRef.value?.setError(errorMessage)
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
.export-dialog {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
}

.character-info {
  text-align: center;
  padding-bottom: var(--spacing-md);
  border-bottom: 1px solid var(--color-border);
}

.character-name {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text);
}

.option-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.section-label {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.checkbox-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.checkbox-option {
  display: grid;
  grid-template-columns: auto 1fr;
  grid-template-rows: auto auto;
  gap: 0 var(--spacing-sm);
  align-items: start;
  cursor: pointer;
  padding: var(--spacing-sm);
  border-radius: var(--radius-sm);
  transition: background 0.15s ease;
}

.checkbox-option:hover {
  background: var(--color-surface-variant);
}

.checkbox-option input[type="checkbox"] {
  grid-row: span 2;
  margin-top: 2px;
  width: 16px;
  height: 16px;
  cursor: inherit;
}

.checkbox-label {
  font-weight: 500;
  color: var(--color-text);
}

.checkbox-desc {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.warning-message {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-warning-50);
  border-radius: var(--radius-sm);
  color: var(--color-warning-700);
  font-size: 0.875rem;
}

.theme-dark .warning-message {
  background: var(--color-warning-900);
  color: var(--color-warning-300);
}

.error-message {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-error-50);
  border-radius: var(--radius-sm);
  color: var(--color-error-700);
  font-size: 0.875rem;
}

.theme-dark .error-message {
  background: var(--color-error-900);
  color: var(--color-error-300);
}

.spinner-sm {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-right: var(--spacing-xs);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
