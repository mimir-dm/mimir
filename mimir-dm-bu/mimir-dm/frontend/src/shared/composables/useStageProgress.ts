import { computed, type ComputedRef } from 'vue'
import type { BoardConfig, BoardEntity, EntityType } from '../../types'

export interface StageInfo {
  key: string
  name: string
  isActive: boolean
  isCompleted: boolean
  isAccessible: boolean
}

export function useStageProgress(
  entity: ComputedRef<BoardEntity | null>,
  boardConfig: ComputedRef<BoardConfig | null>,
  entityType: EntityType
) {
  // Dynamic stages from board configuration
  const stages = computed(() => {
    if (!boardConfig.value || !boardConfig.value.stages) return []
    return boardConfig.value.stages.map((stage) => ({
      key: stage.key,
      name: (stage.display_name || stage.displayName || stage.key).toUpperCase()
    }))
  })

  // Current stage based on entity status
  const currentStage = computed(() => {
    if (!entity.value) return ''
    
    // Handle special case for planning -> concept mapping (campaigns)
    if (entityType === 'campaign' && entity.value.status === 'planning') {
      return 'concept'
    }
    
    return entity.value.status
  })

  // Get index of a stage in the progression
  const getStageIndex = (stageKey: string): number => {
    if (!boardConfig.value || !boardConfig.value.stages) return -1
    return boardConfig.value.stages.findIndex(s => s.key === stageKey)
  }

  // Check if a stage is completed
  const isStageCompleted = (stageKey: string): boolean => {
    const currentIndex = getStageIndex(currentStage.value)
    const checkIndex = getStageIndex(stageKey)
    return checkIndex < currentIndex
  }

  // Check if a stage is accessible
  const isStageAccessible = (stageKey: string): boolean => {
    const currentIndex = getStageIndex(currentStage.value)
    const checkIndex = getStageIndex(stageKey)
    return checkIndex <= currentIndex
  }

  // Get detailed information about all stages
  const stagesInfo = computed((): StageInfo[] => {
    if (!boardConfig.value || !boardConfig.value.stages) return []
    
    return boardConfig.value.stages.map((stage) => ({
      key: stage.key,
      name: (stage.display_name || stage.displayName || stage.key).toUpperCase(),
      isActive: stage.key === currentStage.value,
      isCompleted: isStageCompleted(stage.key),
      isAccessible: isStageAccessible(stage.key)
    }))
  })

  // Get completion percentage across all stages
  const overallProgress = computed(() => {
    if (!boardConfig.value || !boardConfig.value.stages || !currentStage.value) return 0
    
    const totalStages = boardConfig.value.stages.length
    const currentIndex = getStageIndex(currentStage.value)
    
    if (currentIndex === -1) return 0
    return Math.round(((currentIndex + 1) / totalStages) * 100)
  })

  // Check if entity is in an active state (varies by entity type)
  const isInActiveState = computed(() => {
    if (!entity.value) return false
    
    const activeStates: Record<EntityType, string[]> = {
      campaign: ['active', 'concluding', 'completed'],
      module: ['active', 'completed'],
      session: ['running', 'completed']
    }
    
    return activeStates[entityType].includes(entity.value.status)
  })

  // Get next stage in progression
  const nextStage = computed(() => {
    if (!boardConfig.value || !boardConfig.value.stages || !currentStage.value) return null
    
    const currentIndex = getStageIndex(currentStage.value)
    if (currentIndex === -1 || currentIndex >= boardConfig.value.stages.length - 1) {
      return null
    }
    
    return boardConfig.value.stages[currentIndex + 1]
  })

  // Get previous stage in progression
  const previousStage = computed(() => {
    if (!boardConfig.value || !boardConfig.value.stages || !currentStage.value) return null
    
    const currentIndex = getStageIndex(currentStage.value)
    if (currentIndex <= 0) return null
    
    return boardConfig.value.stages[currentIndex - 1]
  })

  return {
    stages,
    currentStage,
    stagesInfo,
    overallProgress,
    isInActiveState,
    nextStage,
    previousStage,
    isStageCompleted,
    isStageAccessible,
    getStageIndex
  }
}