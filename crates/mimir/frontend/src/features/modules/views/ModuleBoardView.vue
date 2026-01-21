<template>
  <BaseBoardView
    entity-type="module"
    :entity="module"
    :board-config="boardConfig"
    :completed-stages="completedStages"
  >
    <!-- Document Sidebar -->
    <template #sidebar>
      <ModuleDocumentSidebar 
        v-if="module && boardConfig"
        ref="documentSidebar"
        :module-id="module.id"
        :module-stage="module.status"
        :board-config="boardConfig"
        :campaign-id="module.campaign_id"
        @select-document="handleSelectDocument"
        @create-document="handleCreateDocument"
        @document-completion-changed="handleDocumentCompletionChanged"
      />
    </template>
    
    <!-- Main Content -->
    <template #content>
      <!-- Stage Landing View (default) -->
      <ModuleStageLandingView 
        v-if="!selectedDocument && module && boardConfig"
        :stage="currentStage"
        :documents="documents"
        :module="module"
        :boardConfig="boardConfig"
        :campaign="campaign"
        @create-document="handleCreateDocumentFromTemplate"
        @edit-document="handleEditDocument" 
        @transition-stage="handleTransitionStage"
        @open-session-document="handleEditDocument"
      />
      
      <!-- Document Editor (when document selected) -->
      <DocumentEditor 
        v-else-if="selectedDocument && module"
        :document="selectedDocument"
        :campaign-id="module.campaign_id"
        :module-id="module.id"
        @close="selectedDocument = null"
        @updated="handleDocumentUpdated"
        @stage-transitioned="handleStageTransitioned"
      />
    </template>
  </BaseBoardView>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useRoute } from 'vue-router'
import { ModuleService } from '@/services/ModuleService'
import { useSharedContextStore } from '@/stores/sharedContext'
import BaseBoardView from '../../../shared/components/ui/BaseBoardView.vue'
import ModuleDocumentSidebar from '../components/ModuleDocumentSidebar.vue'
import ModuleStageLandingView from '../components/ModuleStageLandingView.vue'
import DocumentEditor from '../../campaigns/components/DocumentEditor.vue'
import { useStageProgress } from '../../../shared/composables/useStageProgress'
import { useApiCall } from '../../../shared/composables/useApiCall'
import type { Module, Document, BoardConfig, Campaign } from '../../../types'

const route = useRoute()
const moduleId = computed(() => parseInt(route.params.id as string))
const contextStore = useSharedContextStore()

// Local state
const module = ref<Module | null>(null)
const selectedDocument = ref<Document | null>(null)
const documents = ref<Document[]>([])
const boardConfig = ref<BoardConfig | null>(null)
const campaign = ref<Campaign | null>(null)
const documentSidebar = ref<any>(null)
const initializingDocuments = ref(false)

// Use composables
const { loading: boardLoading, error: boardError, execute: loadBoardApi } = useApiCall<BoardConfig>()
const { loading: moduleLoading, error: moduleError, execute: loadModuleApi } = useApiCall<Module>()
const moduleComputed = computed(() => module.value)
const boardConfigComputed = computed(() => boardConfig.value)
const { 
  currentStage, 
  isStageCompleted, 
  nextStage,
  getStageIndex 
} = useStageProgress(
  moduleComputed,
  boardConfigComputed,
  'module'
)

// Completed stages for visual indication
const completedStages = computed(() => {
  if (!boardConfig.value || !boardConfig.value.stages) return []
  return boardConfig.value.stages
    .filter(stage => isStageCompleted(stage.key))
    .map(stage => stage.key)
})
const nextStageName = computed(() => {
  return nextStage.value?.display_name || 'Next Stage'
})

// Check if can progress to next stage
const canProgressToNext = computed(() => {
  if (!boardConfig.value || !module.value || !nextStage.value) return false
  if (module.value.status === 'completed') return false
  
  // Get current stage metadata
  const currentStageConfig = boardConfig.value.stages?.find((s: any) => s.key === currentStage.value)
  if (!currentStageConfig) return false
  
  // Get required documents for current stage
  const requiredDocs = currentStageConfig.required_documents || []
  
  // Check which documents don't require completion
  const noCompletionDocs = currentStageConfig.no_completion_required_documents || []
  
  // Filter to only documents that need completion
  const completionRequiredDocs = requiredDocs.filter((docId: string) => 
    !noCompletionDocs.includes(docId)
  )
  
  // Check if all required documents that need completion are complete
  const completedDocs = documents.value.filter(doc => 
    doc.template_id && completionRequiredDocs.includes(doc.template_id) && doc.completed_at
  )
  
  return completedDocs.length === completionRequiredDocs.length && completionRequiredDocs.length > 0
})

// Proceed to next stage
// Load board configuration
const loadBoardConfiguration = async () => {
  const data = await loadBoardApi('get_board_configuration', {
    boardType: 'module'
  })
  if (data) {
    boardConfig.value = data
  }
}

// Load module data
const loadModule = async () => {
  try {
    // Load board configuration first
    await loadBoardConfiguration()
    
    const response = await invoke<{ data: Module }>('get_module', { 
      id: moduleId.value 
    })
    module.value = response.data
    
    // Update context with module info
    if (module.value) {
      await contextStore.updateModule({
        id: module.value.id.toString(),
        name: module.value.name,
        campaignId: module.value.campaign_id.toString(),
        currentStage: module.value.status || undefined
      })
    }
    
    // Load campaign info
    await loadCampaign()
    
    // Initialize stage documents if needed
    await initializeStageDocuments()
    
    // Load existing documents
    await loadDocuments()
  } catch (e) {
  }
}

// Load campaign info for directory path
const loadCampaign = async () => {
  if (!module.value) return
  
  try {
    const response = await invoke<{ data: any }>('get_campaign', {
      id: module.value.campaign_id
    })
    campaign.value = response.data
  } catch (e) {
  }
}

// Initialize documents for the current stage
const initializeStageDocuments = async () => {
  if (!module.value || !campaign.value) return

  // Guard against concurrent calls (prevents SQLite locking)
  if (initializingDocuments.value) return
  initializingDocuments.value = true

  try {
    const response = await invoke<{ data: string[] }>('initialize_module_documents', {
      request: {
        module_id: moduleId.value,
        campaign_directory: campaign.value.directory_path
      }
    })

    if (response.data && response.data.length > 0) {
      // Reload documents after initialization
      await loadDocuments()

      // Force sidebar to reload documents too
      if (documentSidebar.value?.loadDocuments) {
        await documentSidebar.value.loadDocuments()
      }
    }
  } catch (e) {
  } finally {
    initializingDocuments.value = false
  }
}

// Load all documents for the module
const loadDocuments = async () => {
  try {
    const response = await invoke<{ data: Document[] }>('get_module_documents', {
      request: {
        module_id: moduleId.value
      }
    })
    documents.value = response.data || []
  } catch (e) {
  }
}

// Handle document selection from sidebar
const handleSelectDocument = (document: Document) => {
  selectedDocument.value = document
}

// Handle create document from sidebar
const handleCreateDocument = () => {
  // Feature not yet implemented: Should open a document creation dialog
  // allowing users to create a new document from scratch with custom fields
  console.warn('Document creation from sidebar not yet implemented')
}

// Handle create document from template (from StageLandingView)
const handleCreateDocumentFromTemplate = async (templateId: string) => {
  if (!module.value || !campaign.value) return
  
  try {
    const response = await invoke<{ data: Document }>('create_document_from_template', {
      campaignId: module.value.campaign_id,
      moduleId: moduleId.value,
      templateId: templateId
    })
    
    if (response.data) {
      documents.value.push(response.data)
      selectedDocument.value = response.data
    }
  } catch (e) {
  }
}

// Handle edit document (from StageLandingView)
const handleEditDocument = (document: Document) => {
  selectedDocument.value = document
}

// Handle stage transition
const handleTransitionStage = async (newStage: string) => {
  try {
    const updatedModule = await ModuleService.transitionStage(moduleId.value, newStage)
    
    if (updatedModule) {
      module.value = updatedModule
      
      // Initialize documents for the new stage
      await initializeStageDocuments()
      
      // Reload documents for new stage
      await loadDocuments()
    }
  } catch (e) {
  }
}

// Handle document updated (e.g., marked as complete)
const handleDocumentUpdated = (updatedDocument: Document) => {
  // Update the document in our local list
  const index = documents.value.findIndex(d => d.id === updatedDocument.id)
  if (index !== -1) {
    documents.value[index] = updatedDocument
  }
}

// Handle document completion changed from sidebar
const handleDocumentCompletionChanged = (updatedDocument: Document) => {
  // Update the document in our local list
  const index = documents.value.findIndex(d => 
    d.id === updatedDocument.id || 
    (d.template_id === updatedDocument.template_id && d.module_id === updatedDocument.module_id)
  )
  if (index !== -1) {
    documents.value[index] = updatedDocument
  } else {
    // If not found, add it (for temporary documents)
    documents.value.push(updatedDocument)
  }
  
  // Force reactivity update for the landing view
  documents.value = [...documents.value]
}

// Handle stage transition
const handleStageTransitioned = async (updatedModule: Module) => {
  // Update the module with proper reactivity
  if (module.value) {
    module.value = { ...module.value, status: updatedModule.status }
  }
  
  // Initialize documents for the new stage
  await initializeStageDocuments()
  
  // Reload documents for the new stage
  await loadDocuments()
  
  // Force sidebar to reload documents
  if (documentSidebar.value?.loadDocuments) {
    await documentSidebar.value.loadDocuments()
  }
  
  // Clear document selection to show landing page
  selectedDocument.value = null
}

// Watch for module ID changes (when navigating between modules)
watch(() => route.params.id, (newId, oldId) => {
  if (newId !== oldId && newId) {
    // Clear current state
    selectedDocument.value = null
    documents.value = []
    
    // Reload module data
    loadModule()
  }
})

onMounted(() => {
  loadModule()
})
</script>

<style scoped>
/* Module-specific overrides if needed */
</style>