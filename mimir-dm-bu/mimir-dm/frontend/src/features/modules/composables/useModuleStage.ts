import { computed, type Ref } from 'vue'
import { ModuleService } from '@/services/ModuleService'
import type { Module, BoardConfig, Document } from '@/types'

interface StageInfo {
  title: string
  subtitle: string
  color?: string
  phase?: string
}

export function useModuleStage(
  module: Ref<Module | null>,
  stage: Ref<string>,
  boardConfig: Ref<BoardConfig | null>,
  documents: Ref<Document[]>
) {
  // Stage information based on current stage
  const stageInfo = computed((): StageInfo => {
    const stageConfigs: Record<string, StageInfo> = {
      planning: {
        title: 'Planning Stage',
        subtitle: 'Define your module structure and prepare content',
        color: 'badge-info',
        phase: 'Planning'
      },
      development: {
        title: 'Development Stage',
        subtitle: 'Build and refine your module content',
        color: 'badge-warning',
        phase: 'In Development'
      },
      ready: {
        title: 'Ready Stage',
        subtitle: 'Final review before going live',
        color: 'badge-success',
        phase: 'Ready'
      },
      active: {
        title: 'Active Stage',
        subtitle: 'Your module is live and in use',
        color: 'badge-primary',
        phase: 'Active'
      },
      completed: {
        title: 'Completed',
        subtitle: 'Module has been completed',
        color: 'badge-neutral',
        phase: 'Completed'
      }
    }
    return stageConfigs[stage.value] || stageConfigs.planning
  })

  // Document progress computation
  const documentProgress = computed(() => {
    if (!boardConfig.value || !boardConfig.value.stages) {
      return { completed: 0, total: 0, percentage: 0 }
    }
    
    const currentStageInfo = boardConfig.value.stages.find((s: any) => s.key === stage.value)
    if (!currentStageInfo) {
      return { completed: 0, total: 0, percentage: 0 }
    }
    
    const requiredDocs = currentStageInfo.required_documents || []
    const noCompletionRequired = currentStageInfo.no_completion_required_documents || []
    const completionRequiredDocs = requiredDocs.filter((docId: string) => 
      !noCompletionRequired.includes(docId)
    )
    
    const total = completionRequiredDocs.length
    const completed = completionRequiredDocs.filter((docId: string) => {
      const doc = documents.value.find(d => d.template_id === docId)
      return doc?.completed_at
    }).length
    
    const percentage = total > 0 ? Math.round((completed / total) * 100) : 0
    
    return { completed, total, percentage }
  })

  // Check if can progress to next stage
  const nextStageAvailable = computed(() => {
    if (!boardConfig.value || !boardConfig.value.stages || stage.value === 'completed') return false
    
    const currentStageInfo = boardConfig.value.stages.find((s: any) => s.key === stage.value)
    if (!currentStageInfo) return false
    
    // Check if there's a next stage
    const stageOrder = boardConfig.value.stages.map((s: any) => s.key)
    const currentIndex = stageOrder.indexOf(stage.value)
    if (currentIndex >= stageOrder.length - 1) return false
    
    // Check if required documents are complete
    const requiredDocs = currentStageInfo.required_documents || []
    const noCompletionRequired = currentStageInfo.no_completion_required_documents || []
    const completionRequiredDocs = requiredDocs.filter((docId: string) => 
      !noCompletionRequired.includes(docId)
    )
    
    const completedDocs = documents.value.filter(doc => 
      doc.template_id && completionRequiredDocs.includes(doc.template_id) && doc.completed_at
    )
    
    // If there are no documents requiring completion (like in active stage), allow progression
    // Otherwise, check if all required documents are completed
    if (completionRequiredDocs.length === 0) {
      return true
    }
    
    return completedDocs.length === completionRequiredDocs.length
  })

  const nextStageName = computed(() => {
    if (!boardConfig.value || !boardConfig.value.stages) return ''
    const stageOrder = boardConfig.value.stages.map((s: any) => s.key)
    const currentIndex = stageOrder.indexOf(stage.value)
    if (currentIndex < stageOrder.length - 1) {
      const nextStage = boardConfig.value.stages[currentIndex + 1]
      return nextStage.display_name || nextStage.displayName || nextStage.title || ''
    }
    return ''
  })

  const nextStagePrompt = computed(() => {
    if (!boardConfig.value || !boardConfig.value.stages) return ''
    const currentStageInfo = boardConfig.value.stages.find((s: any) => s.key === stage.value)
    return currentStageInfo?.transition_prompt || 'You have completed all requirements for this stage.'
  })

  // Transition to next stage
  async function transitionToNextStage() {
    if (!nextStageAvailable.value || !module.value) return
    
    try {
      const stageOrder = boardConfig.value?.stages?.map((s: any) => s.key) || []
      const currentIndex = stageOrder.indexOf(stage.value)
      const nextStageKey = stageOrder[currentIndex + 1]
      
      await ModuleService.updateStatus(module.value.id, nextStageKey)
      
      // Reload or emit event to refresh
      window.location.reload()
    } catch (error) {
      throw error
    }
  }

  return {
    stageInfo,
    documentProgress,
    nextStageAvailable,
    nextStageName,
    nextStagePrompt,
    transitionToNextStage
  }
}