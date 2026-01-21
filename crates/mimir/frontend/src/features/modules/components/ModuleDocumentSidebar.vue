<template>
  <div class="document-sidebar">
    <DocumentSidebarHeader
      :module-stage="moduleStage"
    />

    <BackToCampaignButton :campaign-id="campaignId" />

    <!-- Document List -->
    <div class="document-content">
      <div v-if="loading" class="loading-state">
        Loading documents...
      </div>

      <!-- Active/completed modules: Simple flat list -->
      <div v-else-if="['active', 'completed'].includes(moduleStage)" class="active-documents">
        <div class="document-items">
          <DocumentItem
            v-for="doc in getAllDocumentsForActive()"
            :key="doc.templateId"
            :doc="doc"
            :is-selected="selectedDocument?.template_id === doc.templateId"
            :is-locked="false"
            :edit-icon="getEditIcon()"
            :locked-icon="getLockedIcon()"
            :show-optional-label="false"
            @click="handleDocumentClick"
            @toggle-completion="toggleDocumentCompletion"
          />
        </div>
      </div>

      <!-- Planning modules: Grouped by stages -->
      <div v-else class="stage-groups">
        <DocumentStageGroup
          v-for="stage in boardConfig?.stages || []"
          :key="stage.key"
          :stage-name="stage.display_name"
          :documents="getStageDocuments(stage.key).documents"
          :completed="getStageDocuments(stage.key).completed"
          :total="getStageDocuments(stage.key).total"
          :percentage="getStageDocuments(stage.key).percentage"
          :is-accessible="isStageAccessible(stage.key)"
          :selected-document-id="selectedDocument?.id"
          :edit-icon="getEditIcon()"
          :locked-icon="getLockedIcon()"
          @document-click="handleDocumentClick"
          @toggle-completion="toggleDocumentCompletion"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useThemeStore } from '../../../stores/theme'
import { DocumentService, type Document } from '@/services/DocumentService'
import { ModuleService } from '@/services/ModuleService'
import DocumentSidebarHeader from './DocumentSidebar/DocumentSidebarHeader.vue'
import BackToCampaignButton from './DocumentSidebar/BackToCampaignButton.vue'
import DocumentStageGroup from './DocumentSidebar/DocumentStageGroup.vue'
import DocumentItem from './DocumentSidebar/DocumentItem.vue'

// Import icon images
import lightEditIcon from '../../../assets/images/light-edit.png'
import lightLockedIcon from '../../../assets/images/light-locked.png'
import darkEditIcon from '../../../assets/images/dark-edit.png'
import darkLockedIcon from '../../../assets/images/dark-locked.png'
import hyperEditIcon from '../../../assets/images/hyper-edit.png'
import hyperLockedIcon from '../../../assets/images/hyper-locked.png'

const props = defineProps<{
  moduleId: number
  moduleStage: string
  boardConfig: any
  campaignId?: number
}>()

const emit = defineEmits<{
  selectDocument: [document: Document]
  createDocument: []
  documentCompletionChanged: [document: Document]
}>()

// Get document templates from board configuration
const stageDocuments = computed(() => {
  if (!props.boardConfig) return {}
  
  const documents: Record<string, any[]> = {}
  
  for (const stage of props.boardConfig.stages) {
    documents[stage.key] = [
      ...stage.required_documents.map((docId: string) => ({
        templateId: docId,
        title: docId.replace(/[-_]/g, ' ').split(' ').map((word: string) => 
          word.charAt(0).toUpperCase() + word.slice(1)
        ).join(' '),
        required: true
      })),
      ...stage.optional_documents.map((docId: string) => ({
        templateId: docId,
        title: docId.replace(/[-_]/g, ' ').split(' ').map((word: string) => 
          word.charAt(0).toUpperCase() + word.slice(1)
        ).join(' '),
        required: false
      }))
    ]
  }
  
  return documents
})

// State
const documents = ref<Document[]>([])
const selectedDocument = ref<Document | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)

// Theme store for icon selection
const themeStore = useThemeStore()

// Icon mapping
const iconMap = {
  light: {
    edit: lightEditIcon,
    locked: lightLockedIcon
  },
  dark: {
    edit: darkEditIcon,
    locked: darkLockedIcon
  },
  hyper: {
    edit: hyperEditIcon,
    locked: hyperLockedIcon
  }
}

// Get documents for a specific stage
const getStageDocuments = (stage: string) => {
  const templates = stageDocuments.value[stage] || []
  const stageDocumentList = templates.map((template: any) => {
    // Find matching document by template_id only
    // The module_id filtering is already done when loading documents
    const instance = documents.value.find(doc => 
      doc.template_id === template.templateId
    )
    return {
      ...template,
      instance
    }
  })
  
  // Only count required documents for completion tracking
  const requiredDocs = stageDocumentList.filter((doc: any) => doc.required)
  const completed = requiredDocs.filter((doc: any) => doc.instance?.completed_at).length
  const total = requiredDocs.length
  const percentage = total > 0 ? Math.round((completed / total) * 100) : 0
  
  return {
    documents: stageDocumentList,
    completed,
    total,
    percentage
  }
}

// Check if a stage is accessible based on module progress
const isStageAccessible = (stage: string) => {
  if (!props.boardConfig) return false
  const stageOrder = props.boardConfig.stages.map((s: any) => s.key)
  const currentIndex = stageOrder.indexOf(props.moduleStage)
  const checkIndex = stageOrder.indexOf(stage)
  return checkIndex <= currentIndex
}

// Get edit icon for current theme
const getEditIcon = (): string => {
  const theme = themeStore.currentTheme as 'light' | 'dark' | 'hyper'
  return iconMap[theme]?.edit || lightEditIcon
}

// Get locked icon for current theme
const getLockedIcon = (): string => {
  const theme = themeStore.currentTheme as 'light' | 'dark' | 'hyper'
  return iconMap[theme]?.locked || lightLockedIcon
}

// Get all documents for active/completed modules (simplified view)
const getAllDocumentsForActive = () => {
  const allDocs: any[] = []
  
  // Collect all documents from all stages (both required and optional)
  if (props.boardConfig) {
    for (const stage of props.boardConfig.stages) {
      const templates = stageDocuments.value[stage.key] || []
      for (const template of templates) {
        // Try to find matching document - check both with and without module_id filter
        // since some documents might not have module_id set properly
        const instance = documents.value.find(doc => 
          doc.template_id === template.templateId
        )
        
        if (!instance && documents.value.length > 0) {
          console.log('No instance found for template:', template.templateId, 'in documents:', documents.value)
        }
        
        // Include all documents (with or without instance)
        allDocs.push({
          ...template,
          instance
        })
      }
    }
  }
  
  // Sort alphabetically by title
  return allDocs.sort((a, b) => a.title.localeCompare(b.title))
}

// Load all documents for the module
const loadDocuments = async () => {
  loading.value = true
  error.value = null

  try {
    // Need to get campaignId if not provided as prop
    let campaignId = props.campaignId
    if (!campaignId) {
      // Get campaign ID from the module
      const module = await ModuleService.get(props.moduleId)
      campaignId = module.campaign_id
    }
    
    // First, ensure documents are initialized for this stage
    // This creates any missing documents from templates
    try {
      const initializedDocs = await ModuleService.initializeDocuments(props.moduleId)
      console.log('Initialized module documents:', initializedDocs)
      
      // If documents were initialized, wait a moment for the backend to complete
      if (initializedDocs && initializedDocs.length > 0) {
        await new Promise(resolve => setTimeout(resolve, 100))
      }
    } catch (e) {
      console.log('Document initialization error:', e)
      // Continue even if initialization fails - documents might already exist
    }
    
    console.log('Loading documents for module:', props.moduleId, 'campaign:', campaignId)
    documents.value = await DocumentService.list(props.moduleId, campaignId)
    console.log('Loaded documents:', documents.value)
  } catch (e) {
    console.error('Failed to load documents:', e)
    error.value = 'Failed to load documents'
  } finally {
    loading.value = false
  }
}

// Handle document click
const handleDocumentClick = async (doc: any) => {
  const stage = getDocumentStage(doc.templateId)
  
  // Check if stage is accessible
  if (!isStageAccessible(stage)) {
    return
  }
  
  // If document doesn't exist in database, create it first
  if (!doc.instance) {
    try {
      // First get the module to get the campaign ID
      const module = await ModuleService.get(props.moduleId)
      
      if (!module) {
        return
      }
      
      // Create the document in the database AND on disk
      const response = await invoke<{ data: Document }>('create_document_from_template', {
        campaignId: module.campaign_id,
        moduleId: props.moduleId,
        templateId: doc.templateId
      })
      
      if (response.data) {
        // Check if document already exists and update or add
        const existingIndex = documents.value.findIndex(d => 
          d.template_id === response.data.template_id && d.module_id === props.moduleId
        )
        
        if (existingIndex !== -1) {
          documents.value[existingIndex] = response.data
        } else {
          documents.value.push(response.data)
        }
        
        // Force reactivity update to make checkbox appear
        documents.value = [...documents.value]
        
        // Select the document
        selectDocument(response.data)
      }
    } catch (e) {
    }
  } else {
    selectDocument(doc.instance)
  }
}

// Get which stage a template belongs to
const getDocumentStage = (templateId: string): string => {
  for (const [stage, docs] of Object.entries(stageDocuments.value)) {
    if ((docs as any[]).some(d => d.templateId === templateId)) {
      return stage
    }
  }
  return props.moduleStage
}

// Select a document
const selectDocument = (doc: Document) => {
  selectedDocument.value = doc
  emit('selectDocument', doc)
}

// Toggle document completion status
const toggleDocumentCompletion = async (doc: any) => {
  if (!doc.instance) return
  
  try {
    const updatedDoc = doc.instance.completed_at 
      ? await DocumentService.uncomplete(doc.instance.id)
      : await DocumentService.complete(doc.instance.id)
    
    // Update the document in our local list
    const index = documents.value.findIndex(d => d.id === doc.instance.id)
    if (index !== -1) {
      documents.value[index] = updatedDoc
    }
    
    // Also update the instance reference
    doc.instance = updatedDoc
    
    // Force reactivity update
    documents.value = [...documents.value]
    
    // Emit completion status change
    emit('documentCompletionChanged', updatedDoc)
  } catch (e) {
    console.error('Failed to toggle document completion:', e)
  }
}

// Watch for module or stage changes
watch([() => props.moduleId, () => props.moduleStage], () => {
  loadDocuments()
})

onMounted(() => {
  loadDocuments()
})

// Expose loadDocuments for parent to call
defineExpose({
  loadDocuments
})
</script>

<style scoped>
.document-sidebar {
  width: 320px;
  height: 100%;
  background-color: var(--color-surface);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
}

.document-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--spacing-md);
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-xl);
  text-align: center;
  color: var(--color-text-secondary);
}

.stage-groups {
  display: flex;
  flex-direction: column;
}

.active-documents {
  padding-top: var(--spacing-sm);
}

.document-items {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}
</style>