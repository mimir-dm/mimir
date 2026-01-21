<template>
  <div id="book-reader" :class="`theme-${currentTheme}`">
    <!-- Mode Switcher -->
    <div class="mode-switcher-bar">
      <div class="mode-switcher">
        <button 
          :class="['mode-button', { active: currentMode === 'reading' }]"
          @click="currentMode = 'reading'"
        >
          Reading
        </button>
        <button 
          :class="['mode-button', { active: currentMode === 'catalog' }]"
          @click="currentMode = 'catalog'"
        >
          Catalog
        </button>
      </div>
    </div>
    
    <!-- Different layouts for different modes -->
    <ThreePanelLayout v-if="currentMode === 'reading'">
      <template #left>
        <BookLibrary
          :library-books="libraryBooks"
          :selected-book="selectedBook"
          :is-loading-library="isLoadingLibrary"
          :is-development="isDevelopment"
          :mode="currentMode"
          @select="selectBook"
          @updateSources="selectedSources = $event"
          @remove="removeBook"
        />
      </template>
      
      <template #center>
        <BookTableOfContents
          v-if="selectedBook && bookContent?.data"
          :sections="bookContent.data"
          :selected-section="selectedSection"
          @select="selectedSection = $event"
          @jump="jumpToEntry"
        />
        <Panel v-else title="Contents" variant="default">
          <div class="empty-toc">
            <p>Select a book to view contents</p>
          </div>
        </Panel>
      </template>
      
      <template #right>
        <BookContentViewer
          :selected-book="selectedBook"
          :content="currentSection"
          :is-loading="isLoading"
          :error="error"
        />
      </template>
    </ThreePanelLayout>
    
    <!-- Two-panel layout for catalog mode -->
    <TwoPanelLayout v-else>
      <template #left>
        <BookLibrary
          :library-books="libraryBooks"
          :selected-book="selectedBook"
          :is-loading-library="isLoadingLibrary"
          :is-development="isDevelopment"
          :mode="currentMode"
          @select="selectBook"
          @updateSources="selectedSources = $event"
          @remove="removeBook"
        />
      </template>
      
      <template #right>
        <CatalogPanel :selected-category="selectedCatalogCategory" :selected-sources="selectedSources" />
      </template>
    </TwoPanelLayout>
    
    <!-- Cross-reference tooltip -->
    <div 
      v-if="tooltipVisible"
      class="cross-ref-tooltip"
      :style="{ left: `${tooltipPosition.x}px`, top: `${tooltipPosition.y}px` }"
      v-html="tooltipContent"
    />
    
    <!-- Cross-reference modal -->
    <AppModal
      :visible="modalContent.visible"
      :title="modalContent.title"
      size="md"
      @close="closeModal"
    >
      <div class="dnd-content" v-html="modalContent.content"></div>
    </AppModal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useThemeStore } from '../../../../stores/theme'
import { useSharedContextStore } from '../../../../stores/sharedContext'
import { useBookLibrary } from '../../composables/useBookLibrary'
import { useBookContent } from '../../composables/useBookContent'
import { useBookNavigation } from '../../composables/useBookNavigation'
import { useCrossReferences } from '../../composables/useCrossReferences'

// Components
import AppModal from '@/components/shared/AppModal.vue'
import ThreePanelLayout from '../../../../shared/components/layout/ThreePanelLayout.vue'
import TwoPanelLayout from '../../../../shared/components/layout/TwoPanelLayout.vue'
import Panel from '../../../../shared/components/layout/Panel.vue'
import BookLibrary from './Library.vue'
import BookTableOfContents from './TableOfContents.vue'
import BookContentViewer from './ContentViewer.vue'
import CatalogPanel from '../../views/SearchView.vue'

// Theme
const themeStore = useThemeStore()
const contextStore = useSharedContextStore()
const currentTheme = computed(() => themeStore.currentTheme)

// Mode state (reading vs catalog)
type AppMode = 'reading' | 'catalog'
const currentMode = ref<AppMode>('reading')

// Catalog state
const selectedCatalogCategory = ref('Spells')
const selectedSources = ref<string[]>([])

// Watch for source selection changes
watch(selectedSources, async (newSources) => {
  // Update context when sources change in catalog mode
  if (currentMode.value === 'catalog') {
    await contextStore.updateReference({
      ...contextStore.reference,
      catalog: {
        ...contextStore.reference?.catalog,
        selectedSources: newSources
      }
    })
  }
})

// Book library management
const {
  libraryBooks,
  selectedBook,
  isLoadingLibrary,
  isDevelopment,
  loadLibraryBooks,
  removeBook,
  selectBook,
} = useBookLibrary()

// Book content management
const {
  bookContent,
  selectedSection,
  isLoading,
  error,
  loadBookContent,
  jumpToEntry: jumpToEntryBase,
  getCurrentSection
} = useBookContent()

// Navigation
const { scrollToElement } = useBookNavigation()

// Cross-references
const {
  tooltipContent,
  tooltipVisible,
  tooltipPosition,
  modalContent,
  handleCrossRefHover,
  handleCrossRefClick,
  hideTooltip,
  closeModal
} = useCrossReferences()

// Current section content
const currentSection = computed(() => getCurrentSection())

// Jump to entry with scroll
function jumpToEntry(sectionIndex: number, entryId: string) {
  jumpToEntryBase(sectionIndex, entryId)
}

// Watch for book selection changes
watch(selectedBook, async (newBook) => {
  if (newBook) {
    loadBookContent(newBook)
    
    // Update reference context with selected book (reading mode)
    if (currentMode.value === 'reading') {
      const bookName = newBook.name || newBook.id
      await contextStore.updateReference({
        ...contextStore.reference,
        activeTab: 'reading',
        reading: {
          currentBook: bookName,
          currentSection: contextStore.reference?.reading?.currentSection
        },
        catalog: undefined // Clear catalog context when in reading mode
      })
    }
  }
}, { immediate: true })

// Watch for mode changes
watch(currentMode, async (newMode) => {
  // Update reference context based on mode
  if (newMode === 'reading') {
    await contextStore.updateReference({
      activeTab: 'reading',
      reading: {
        currentBook: selectedBook.value?.name || selectedBook.value?.id,
        currentSection: undefined
      },
      catalog: undefined
    })
  } else {
    await contextStore.updateReference({
      activeTab: 'catalog',
      reading: undefined,
      catalog: {
        selectedCategory: selectedCatalogCategory.value,
        selectedItems: [],
        searchQuery: '',
        selectedSources: selectedSources.value
      }
    })
  }
})

// Watch for catalog category changes
watch(selectedCatalogCategory, async (newCategory) => {
  if (currentMode.value === 'catalog') {
    await contextStore.updateReference({
      ...contextStore.reference,
      activeTab: 'catalog',
      catalog: {
        ...contextStore.reference?.catalog,
        selectedCategory: newCategory
      }
    })
  }
})

// Setup cross-reference event handlers
function setupCrossRefHandlers() {
  // Remove old listeners
  document.removeEventListener('mouseover', handleCrossRefHover as any)
  document.removeEventListener('mouseout', hideTooltip)
  document.removeEventListener('click', handleCrossRefClick as any)
  
  // Add new listeners
  document.addEventListener('mouseover', handleCrossRefHover as any)
  document.addEventListener('mouseout', (e) => {
    const target = e.target as HTMLElement
    if (target.classList?.contains('cross-ref-link')) {
      hideTooltip()
    }
  })
  document.addEventListener('click', handleCrossRefClick as any)
}

// Load initial data
onMounted(async () => {
  // Register this window with context service
  ;(window as any).__TAURI_WINDOW_ID__ = 'reference'
  await contextStore.registerWindow({
    id: 'reference',
    type: 'reference',
    title: 'Source Library',
    focused: true
  })
  
  // Initialize reference context
  await contextStore.updateReference({
    activeTab: currentMode.value,
    reading: currentMode.value === 'reading' ? {
      currentBook: undefined,
      currentSection: undefined
    } : undefined,
    catalog: currentMode.value === 'catalog' ? {
      selectedCategory: selectedCatalogCategory.value,
      selectedItems: [],
      searchQuery: '',
      selectedSources: []
    } : undefined
  })
  
  // Initialize theme - exactly as in original BookApp.vue
  themeStore.applyTheme()
  await themeStore.initThemeSync()
  
  // Load library books
  await loadLibraryBooks()
  
  // Setup cross-reference handlers
  setupCrossRefHandlers()
})

// Re-setup handlers when content changes
watch([bookContent, selectedSection], () => {
  nextTick(() => {
    setupCrossRefHandlers()
  })
})

// Cleanup on unmount
onUnmounted(() => {
  contextStore.unregisterWindow('reference')
  document.removeEventListener('mouseover', handleCrossRefHover as any)
  document.removeEventListener('mouseout', hideTooltip)
  document.removeEventListener('click', handleCrossRefClick as any)
})
</script>

<!-- Component styles have been moved to centralized CSS files -->
