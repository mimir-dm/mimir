<template>
  <div class="catalog-panel">
    <!-- Header -->
    <div class="catalog-header">
      <!-- Category Tabs -->
      <ContentCategoryTabs v-model="selectedCategory" />
      
      <div class="header-controls">
        <div class="search-bar">
          <input 
            type="text" 
            v-model="searchQuery" 
            @input="debouncedSearch"
            placeholder="Search..."
            class="search-input"
          >
        </div>
        
        <div class="results-count" v-if="searchPerformed">
          {{ resultCount }} results
        </div>
      </div>
      
    </div>
    
    <!-- Content -->
    <div class="catalog-content">
      <div class="table-container">
        <SearchResults
          :category="selectedCategory"
          :results="results"
          :search-performed="searchPerformed"
          :sort-column="sortColumn"
          :sort-direction="sortDirection"
          :monster-filters="filters.monsters"
          :available-sources="classSources"
          @select-spell="selectSpell"
          @select-item="selectItem"
          @select-monster="selectMonster"
          @select-class="selectClass"
          @select-feat="selectFeat"
          @select-race="selectRace"
          @select-background="selectBackground"
          @select-action="selectAction"
          @select-condition="selectCondition"
          @select-option="selectOption"
          @select-deity="selectDeity"
          @select-object="selectObject"
          @select-trap="selectTrap"
          @select-language="selectLanguage"
          @select-reward="selectReward"
          @select-table="selectTable"
          @select-variant-rule="selectVariantRule"
          @select-vehicle="selectVehicle"
          @select-cult="selectCult"
          @select-psionic="selectPsionic"
          @sort="handleSort"
          @update-monster-filters="updateMonsterFilters"
        />
      </div>
    </div>
    
    <!-- Modal Stack -->
    <AppModal
      v-for="(modal, index) in modalStack"
      :key="`modal-${index}`"
      :visible="modal.visible"
      :title="modal.title"
      size="md"
      :stack-index="index"
      @close="() => closeModal(index)"
    >
      <div
        class="dnd-content"
        v-html="modal.content"
        @click="(e: MouseEvent) => handleContentClick(e, handleReferenceClick)"
      ></div>
    </AppModal>
  </div>
</template>

<script setup lang="ts">
import { onMounted, toRef } from 'vue'
import { useSearch } from '../composables/useSearch'
import AppModal from '@/components/shared/AppModal.vue'
import ContentCategoryTabs from '../components/search/ContentCategoryTabs.vue'
import SearchResults from '../components/search/SearchResults.vue'

// Handle clicks on cross-reference links within D&D content
function handleContentClick(
  event: MouseEvent,
  callback: (ref: { type: string; name: string; source?: string; className?: string; level?: string }) => void
) {
  const target = event.target as HTMLElement

  // Debug: Log all clicks in the content area
  console.log('handleContentClick fired:', {
    tagName: target.tagName,
    className: target.className,
    classList: Array.from(target.classList),
    hasDataRefType: target.hasAttribute('data-ref-type'),
    dataRefType: target.getAttribute('data-ref-type'),
    dataRefName: target.getAttribute('data-ref-name'),
    innerHTML: target.innerHTML?.substring(0, 100)
  })

  if (target.classList.contains('cross-ref-link') ||
      target.classList.contains('reference-link') ||
      target.classList.contains('creature-ref') ||
      target.classList.contains('item-ref') ||
      target.classList.contains('spell-ref') ||
      target.classList.contains('condition-ref') ||
      target.classList.contains('race-ref') ||
      target.classList.contains('class-ref') ||
      target.classList.contains('feat-ref') ||
      target.classList.contains('background-ref') ||
      target.classList.contains('action-ref') ||
      target.classList.contains('feature-ref') ||
      target.classList.contains('clickable') ||
      (target.tagName === 'A' && target.hasAttribute('data-ref-type'))) {

    event.preventDefault()
    event.stopPropagation()

    let type = target.getAttribute('data-ref-type') || ''

    if (!type) {
      if (target.classList.contains('creature-ref')) type = 'creature'
      else if (target.classList.contains('item-ref')) type = 'item'
      else if (target.classList.contains('spell-ref')) type = 'spell'
    }

    const name = target.getAttribute('data-ref-name') ||
                 target.getAttribute('data-name') ||
                 target.textContent || ''
    const source = target.getAttribute('data-ref-source') ||
                   target.getAttribute('data-source') ||
                   undefined

    // Extract additional attributes for class features
    const className = target.getAttribute('data-class-name') || undefined
    const level = target.getAttribute('data-level') || undefined

    console.log('handleContentClick - extracted:', { type, name, source, className, level })
    if (name && type) {
      console.log('handleContentClick - calling callback with:', { type, name, source, className, level })
      callback({ type, name, source, className, level })
    } else {
      console.log('handleContentClick - no callback: name or type missing')
    }
  }
}

interface Props {
  selectedSources: string[]
  selectedCategory: string
}

const props = defineProps<Props>()

// Create reactive references to props
const sourcesRef = toRef(props, 'selectedSources')

const {
  selectedCategory,
  searchQuery,
  searchPerformed,
  sortColumn,
  sortDirection,
  results,
  filters,
  modalStack,
  resultCount,
  classSources,
  performSearch,
  debouncedSearch,
  handleSort,
  updateMonsterFilters,
  selectSpell,
  selectItem,
  selectMonster,
  selectClass,
  selectFeat,
  selectRace,
  selectBackground,
  selectAction,
  selectCondition,
  selectOption,
  selectDeity,
  selectObject,
  selectTrap,
  selectLanguage,
  selectReward,
  selectTable,
  selectVariantRule,
  selectVehicle,
  selectCult,
  selectPsionic,
  closeModal,
  handleReferenceClick,
  initialize
} = useSearch(props.selectedCategory, sourcesRef)


onMounted(() => {
  initialize()
})
</script>

<style scoped>
.catalog-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--color-background, #0d0d0d);
}

.catalog-header {
  background: var(--color-surface, #1a1a1a);
  border-bottom: 1px solid var(--color-border, #333);
}

.header-controls {
  display: flex;
  align-items: center;
  gap: var(--spacing-lg, 16px);
  padding: var(--spacing-md, 12px) var(--spacing-lg, 16px);
  margin-top: var(--spacing-sm, 8px);
}

.search-bar {
  flex: 1;
  max-width: 400px;
}

.search-input {
  width: 100%;
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
  background: var(--color-background, #0d0d0d);
  border: 1px solid var(--color-border, #333);
  border-radius: 4px;
  color: var(--color-text, #e0e0e0);
  font-size: 0.9rem;
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary, #4a9eff);
}

.results-count {
  color: var(--color-text-secondary, #999);
  font-size: 0.9rem;
}

.catalog-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.table-container {
  flex: 1;
  overflow: auto;
  padding: 0;
}
</style>