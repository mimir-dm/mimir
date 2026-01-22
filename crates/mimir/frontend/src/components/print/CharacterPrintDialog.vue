<template>
  <AppModal
    :visible="visible"
    title="Print Character"
    size="md"
    @close="handleClose"
  >
    <div class="print-dialog">
      <!-- Character Info -->
      <div class="character-info" v-if="characterName">
        <h3 class="character-name">{{ characterName }}</h3>
      </div>

      <!-- Sections -->
      <div class="section-group">
        <div class="section-header">Select sections to include:</div>

        <!-- Compact Sheet -->
        <div class="mode-card" :class="{ active: options.includeCompactSheet }">
          <label class="mode-header" @click.prevent="options.includeCompactSheet = !options.includeCompactSheet">
            <input type="checkbox" v-model="options.includeCompactSheet" @click.stop />
            <div class="mode-info">
              <span class="mode-label">Compact Sheet (2-page)</span>
              <span class="mode-desc">Stats, combat, skills, equipment summary</span>
            </div>
          </label>
        </div>

        <!-- Long Form -->
        <div class="mode-card" :class="{ active: options.includeLongForm }">
          <label class="mode-header" @click.prevent="options.includeLongForm = !options.includeLongForm">
            <input type="checkbox" v-model="options.includeLongForm" @click.stop />
            <div class="mode-info">
              <span class="mode-label">Long Form</span>
              <span class="mode-desc">Appearance, personality, backstory, RP notes</span>
            </div>
          </label>
        </div>

        <!-- Battle Card -->
        <div class="mode-card" :class="{ active: options.includeBattleCard }">
          <label class="mode-header" @click.prevent="options.includeBattleCard = !options.includeBattleCard">
            <input type="checkbox" v-model="options.includeBattleCard" @click.stop />
            <div class="mode-info">
              <span class="mode-label">Battle Card</span>
              <span class="mode-desc">Half-page combat reference card (AC, HP, attacks, saves)</span>
            </div>
          </label>
        </div>

        <!-- Spell Cards -->
        <div class="mode-card" :class="{ active: options.includeSpellCards }">
          <label class="mode-header" @click.prevent="options.includeSpellCards = !options.includeSpellCards">
            <input type="checkbox" v-model="options.includeSpellCards" @click.stop />
            <div class="mode-info">
              <span class="mode-label">Spell Cards</span>
              <span class="mode-desc">Printable cards for all spells (if caster)</span>
            </div>
          </label>
        </div>

        <!-- Equipment Cards -->
        <div class="mode-card" :class="{ active: options.includeEquipmentCards }">
          <label class="mode-header" @click.prevent="options.includeEquipmentCards = !options.includeEquipmentCards">
            <input type="checkbox" v-model="options.includeEquipmentCards" @click.stop />
            <div class="mode-info">
              <span class="mode-label">Equipment Cards</span>
              <span class="mode-desc">Printable cards for weapons, magic items, special ammo</span>
            </div>
          </label>
        </div>

        <!-- Equipment Detail -->
        <div class="mode-card" :class="{ active: options.includeEquipmentDetail }">
          <label class="mode-header" @click.prevent="options.includeEquipmentDetail = !options.includeEquipmentDetail">
            <input type="checkbox" v-model="options.includeEquipmentDetail" @click.stop />
            <div class="mode-info">
              <span class="mode-label">Equipment Detail</span>
              <span class="mode-desc">Full inventory with descriptions and special rules</span>
            </div>
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
        {{ isLoading ? 'Generating...' : 'Export' }}
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
.print-dialog {
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

.section-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.section-header {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-xs);
}

.mode-card {
  border: 2px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  cursor: pointer;
  transition: all 0.2s ease;
  overflow: hidden;
}

.mode-card:hover:not(.active) {
  border-color: var(--color-primary-400);
  background: var(--color-surface-variant);
}

.mode-card.active {
  border-color: var(--color-primary-500);
  background: var(--color-primary-50);
}

.theme-dark .mode-card.active,
.theme-hyper .mode-card.active {
  background: var(--color-primary-900);
}

.mode-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
}

.mode-info {
  display: flex;
  flex-direction: column;
}

.mode-label {
  font-weight: 600;
  color: var(--color-text);
}

.mode-desc {
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
