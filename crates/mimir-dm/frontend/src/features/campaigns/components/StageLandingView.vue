<template>
  <div class="stage-landing">
    <!-- Stage Header -->
    <StageHeader
      :title="stageInfo.title"
      :subtitle="stageInfo.subtitle"
    />

    <!-- Campaign Summary Actions -->
    <div class="campaign-actions" v-if="campaign?.id">
      <div class="summary-controls">
        <button
          class="btn btn-ghost btn-sm"
          :disabled="refreshingSummary"
          @click="handleRefreshSummary"
          title="Regenerate the AI story summary from session notes"
        >
          {{ refreshingSummary ? 'Refreshing...' : 'Refresh Story Summary' }}
        </button>
        <span v-if="summaryLastUpdated" class="summary-timestamp">
          Last updated: {{ formatTimestamp(summaryLastUpdated) }}
        </span>
        <span v-else class="summary-timestamp summary-none">
          No summary generated yet
        </span>
      </div>
    </div>

    <!-- Next Steps (shown at top when ready, except for active/concluding stages) -->
    <StageTransitionCard
      v-if="nextStageAvailable && stage !== 'active' && stage !== 'concluding'"
      :next-stage-name="nextStageName"
      :transition-prompt="nextStagePrompt"
      @transition="transitionToNextStage"
    />

    <!-- Stage-Specific Content from Backend -->
    <div class="stage-content-section" v-if="stageContent && stage !== 'active' && stage !== 'concluding'">
      <div :class="`stage-${stage}`">
        <div class="activity-section" v-html="stageContent"></div>
      </div>

      <!-- Campaign NPCs Section for Integration stage -->
      <CampaignNPCs
        v-if="stage === 'integration' && campaign?.id"
        class="mt-4"
        :campaign-id="campaign.id"
      />
    </div>

    <!-- Module Management for Active and Concluding stages -->
    <div class="stage-content-section" v-else-if="stage === 'active' || stage === 'concluding'">
      <ModulesTable
        v-if="stage === 'active'"
        :modules="modules"
        :loading="modulesLoading"
        :campaign-id="campaign?.id"
        title="Modules"
        @create-module="showCreateModal = true"
      />

      <ModulesTable
        v-else-if="stage === 'concluding'"
        :modules="modules"
        :loading="modulesLoading"
        :campaign-id="campaign?.id"
        title="Active Modules"
        empty-message="No active modules."
        @create-module="showCreateModal = true"
      />

      <!-- Campaign NPCs Section -->
      <CampaignNPCs
        v-if="campaign?.id"
        class="mt-4"
        :campaign-id="campaign.id"
      />

      <!-- Campaign Maps Section -->
      <CampaignMaps
        v-if="campaign?.id"
        class="mt-4"
        :campaign-id="campaign.id"
      />

      <!-- Show backend content after modules -->
      <div v-if="stageContent" :class="`stage-${stage}`" class="mt-4">
        <div class="activity-section" v-html="stageContent"></div>
      </div>

      <!-- Stage transition at bottom for active/concluding stages -->
      <StageTransitionCard
        v-if="nextStageAvailable"
        class="mt-4"
        :next-stage-name="nextStageName"
        :transition-prompt="nextStagePrompt"
        @transition="transitionToNextStage"
      />
    </div>

    <!-- Stage Guidance Content (fallback for stages without backend content) -->
    <StageGuidance
      v-else
      :stage="stage"
      :total-modules="modules.length || 0"
      :total-sessions="getTotalSessions()"
      :total-documents="getTotalDocuments()"
    />

    <!-- Create Module Modal -->
    <CreateModuleModal
      :show="showCreateModal"
      @close="showCreateModal = false"
      @create="handleCreateModule"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onActivated, watch } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { ModuleService } from '@/services/ModuleService'
import { boardConfigService } from '../../../services/boardConfigService'
import StageHeader from './StageLanding/StageHeader.vue'
import StageTransitionCard from './StageLanding/StageTransitionCard.vue'
import ModulesTable from './StageLanding/ModulesTable.vue'
import CreateModuleModal from './StageLanding/CreateModuleModal.vue'
import StageGuidance from './StageLanding/StageGuidance.vue'
import CampaignNPCs from './StageLanding/CampaignNPCs.vue'
import CampaignMaps from './StageLanding/CampaignMaps.vue'

const props = defineProps<{
  stage: string
  documents: any[]
  campaign: any
  boardConfig: any
}>()

const emit = defineEmits<{
  transitionStage: [newStage: string]
}>()

const router = useRouter()

// Stage content from backend
const stageContent = ref<string>('')

// Module state
const modules = ref<any[]>([])
const modulesLoading = ref(false)
const showCreateModal = ref(false)

// Summary invalidation state
const refreshingSummary = ref(false)
const summaryLastUpdated = ref<string | null>(null)

// Get stage info from board configuration
const stageInfo = computed(() => {
  if (!props.boardConfig) {
    return { title: '', subtitle: '' }
  }
  const currentStageInfo = props.boardConfig.stages.find((s: any) => s.key === props.stage)
  if (!currentStageInfo) {
    return { title: '', subtitle: '' }
  }
  return {
    title: currentStageInfo.display_name,
    subtitle: currentStageInfo.description
  }
})

// Load stage-specific content from backend
async function loadStageContent() {
  if (!props.boardConfig || !props.stage) return

  try {
    // Get stage content from board config
    const currentStageConfig = props.boardConfig.stages?.find((s: any) => s.key === props.stage)

    // Get content field
    const content = (currentStageConfig as any)?.content

    if (content) {
      stageContent.value = content
    } else {
      stageContent.value = ''
    }
  } catch (error) {
    stageContent.value = ''
  }
}

// Watch for stage changes
watch(() => props.stage, async () => {
  await loadStageContent()
})

// Initialize board configuration service on mount
onMounted(async () => {
  await loadStageContent()
  await loadSummaryStatus()
  if (props.boardConfig && !boardConfigService.getBoardConfig('campaign')) {
    // Transform and cache the board config in the service
    const config = {
      boardType: props.boardConfig.board_type || 'campaign',
      stages: props.boardConfig.stages.map((stage: any) => ({
        key: stage.key,
        displayName: stage.display_name,
        description: stage.description,
        requiredDocuments: stage.required_documents || [],
        optionalDocuments: stage.optional_documents || [],
        metadata: {
          displayName: stage.display_name,
          description: stage.description,
          completionMessage: stage.completion_message,
          transitionPrompt: stage.transition_prompt,
          helpText: stage.help_text
        }
      })),
      transitions: props.boardConfig.transitions || {}
    }

    // Cache it in the service
    boardConfigService.cacheBoard(config)
  }
})

// Check if can progress to next stage
const nextStageAvailable = computed(() => {
  // Check if all required documents are complete
  if (!props.boardConfig) return false
  const currentStageInfo = props.boardConfig.stages.find((s: any) => s.key === props.stage)
  if (!currentStageInfo) return false

  // Check if there's a next stage
  const stageOrder = props.boardConfig.stages.map((s: any) => s.key)
  const currentIndex = stageOrder.indexOf(props.stage)
  if (currentIndex >= stageOrder.length - 1) return false

  // Check if required documents are complete
  const requiredDocs = currentStageInfo.required_documents || []

  // If there are no required documents (like active stage), allow progression
  if (requiredDocs.length === 0) {
    return true
  }

  const completedDocs = props.documents.filter(doc =>
    requiredDocs.includes(doc.template_id) && doc.completed_at
  )

  return completedDocs.length === requiredDocs.length
})

const nextStageName = computed(() => {
  if (!props.boardConfig) return ''
  const stageOrder = props.boardConfig.stages.map((s: any) => s.key)
  const currentIndex = stageOrder.indexOf(props.stage)
  if (currentIndex < stageOrder.length - 1) {
    const nextStage = props.boardConfig.stages[currentIndex + 1]
    return nextStage.display_name
  }
  return ''
})

const nextStagePrompt = computed(() => {
  if (!props.boardConfig) return ''
  const currentStageInfo = props.boardConfig.stages.find((s: any) => s.key === props.stage)
  return currentStageInfo?.transition_prompt || ''
})

// Methods
const transitionToNextStage = () => {
  if (!props.boardConfig) return
  const stageOrder = props.boardConfig.stages.map((s: any) => s.key)
  const currentIndex = stageOrder.indexOf(props.stage)
  if (currentIndex < stageOrder.length - 1) {
    emit('transitionStage', stageOrder[currentIndex + 1])
  }
}

// Module methods
const loadModules = async () => {
  if (!props.campaign?.id) {
    console.log('No campaign ID available')
    return
  }

  console.log('Loading modules for campaign:', props.campaign.id)
  modulesLoading.value = true
  try {
    modules.value = await ModuleService.list(props.campaign.id)
    console.log('Loaded modules:', modules.value)
  } catch (e) {
    console.error('Failed to load modules:', e)
  } finally {
    modulesLoading.value = false
  }
}

const getTotalSessions = (): number => {
  return modules.value.reduce((total, module) =>
    total + (module.actual_sessions || 0), 0
  )
}

const getTotalDocuments = (): number => {
  // Count campaign docs + estimated module/session docs
  return (props.documents?.length || 0) +
         (modules.value.length * 3) + // Estimate 3 docs per module
         (getTotalSessions() * 2) // Estimate 2 docs per session
}

const handleCreateModule = async (data: { name: string; type: string; sessions: number }) => {
  if (!props.campaign?.id) {
    console.log('Missing campaign ID')
    return
  }

  console.log('Creating module for campaign:', props.campaign.id)
  try {
    const newModule = await ModuleService.create({
      campaign_id: props.campaign.id,
      name: data.name,
      module_type: data.type
      // expected_sessions not supported by ModuleService yet
    })
    console.log('Created module:', newModule)

    if (newModule) {
      // Close modal
      showCreateModal.value = false

      // Navigate to the new module's board
      router.push(`/modules/${newModule.id}/board`)
    }
  } catch (e) {
    console.error('Failed to create module:', e)
  }
}

// Format ISO timestamp to readable format
const formatTimestamp = (isoString: string): string => {
  try {
    const date = new Date(isoString)
    return date.toLocaleString()
  } catch {
    return isoString
  }
}

// Load current summary status
const loadSummaryStatus = async () => {
  if (!props.campaign?.id) return

  try {
    const response = await invoke<{ success: boolean; data?: { summary: string | null; last_updated: string | null }; error?: string }>(
      'get_campaign_summary',
      { campaignId: props.campaign.id }
    )

    if (response.success && response.data) {
      summaryLastUpdated.value = response.data.last_updated
    }
  } catch (e) {
    console.error('Error loading campaign summary status:', e)
  }
}

// Refresh (regenerate) the campaign summary using LLM
const handleRefreshSummary = async () => {
  if (!props.campaign?.id) return

  refreshingSummary.value = true
  try {
    const response = await invoke<{ success: boolean; data?: { summary: string | null; last_updated: string | null }; error?: string }>(
      'refresh_campaign_summary',
      { campaignId: props.campaign.id }
    )

    if (response.success && response.data) {
      summaryLastUpdated.value = response.data.last_updated
      console.log('Campaign summary refreshed')
    } else if (response.error) {
      console.error('Failed to refresh summary:', response.error)
    }
  } catch (e) {
    console.error('Error refreshing campaign summary:', e)
  } finally {
    refreshingSummary.value = false
  }
}

// Watch for stage changes to load modules when entering active, concluding, or completed stages
watch(() => props.stage, (newStage) => {
  if (['active', 'concluding', 'completed'].includes(newStage)) {
    loadModules()
  }
})

// Load modules if already in active, concluding, or completed stage
onMounted(() => {
  if (['active', 'concluding', 'completed'].includes(props.stage)) {
    loadModules()
  }
})

// Reload modules when component is reactivated (e.g., returning from module view)
onActivated(() => {
  if (['active', 'concluding', 'completed'].includes(props.stage)) {
    loadModules()
  }
})
</script>

<style scoped>
.campaign-actions {
  display: flex;
  justify-content: flex-end;
  padding: var(--spacing-md) 0;
  border-bottom: 1px solid var(--color-border);
  margin-bottom: var(--spacing-md);
}

.campaign-actions .btn {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.summary-controls {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
}

.summary-timestamp {
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
}

.summary-timestamp.summary-none {
  font-style: italic;
}
</style>
