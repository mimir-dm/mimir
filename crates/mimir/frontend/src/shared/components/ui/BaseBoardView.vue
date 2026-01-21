<template>
  <MainLayout>
    <div class="board-container" :class="`${entityType}-board-container`">
      <!-- Document Sidebar -->
      <slot name="sidebar" />
      
      <!-- Main Board Content -->
      <div class="board" :class="`${entityType}-board`">
        <!-- Kanban Stage Progress -->
        <div class="stage-progress">
          <div 
            v-for="(stage, index) in stages" 
            :key="stage.key"
            class="stage-indicator"
            :class="{ 
              active: currentStage === stage.key,
              completed: isStageCompleted(stage.key)
            }"
            :style="{ zIndex: stages.length - index }"
          >
            <div class="stage-content">
              <div class="stage-name">{{ stage.name }}</div>
              <div class="stage-marker" v-if="currentStage === stage.key">●</div>
            </div>
            <div class="stage-arrow-point"></div>
          </div>
        </div>

        <!-- Main Content Area -->
        <div class="main-content">
          <slot name="content" />
        </div>
      </div>
    </div>
  </MainLayout>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import MainLayout from '../layout/MainLayout.vue'
import type { BoardConfig, BoardEntity, EntityType } from '../../../types'

interface Props {
  entityType: EntityType
  entity: BoardEntity | null
  boardConfig: BoardConfig | null
  completedStages?: string[]
}

const props = defineProps<Props>()

// Dynamic stages from board configuration
const stages = computed(() => {
  if (!props.boardConfig || !props.boardConfig.stages) return []
  return props.boardConfig.stages.map((stage) => ({
    key: stage.key,
    name: (stage.display_name || stage.displayName || stage.key).toUpperCase()
  }))
})

// Current stage based on entity status
const currentStage = computed(() => {
  if (!props.entity) return ''
  
  // Handle special case for planning -> concept mapping (campaigns)
  if (props.entityType === 'campaign' && props.entity.status === 'planning') {
    return 'concept'
  }
  
  return props.entity.status
})

// Check if a stage is completed
const isStageCompleted = (stageKey: string): boolean => {
  if (!props.completedStages) return false
  return props.completedStages.includes(stageKey)
}

// Expose computed properties for parent components
defineExpose({
  stages,
  currentStage,
  isStageCompleted
})
</script>

<!-- Component styles have been moved to centralized CSS files -->