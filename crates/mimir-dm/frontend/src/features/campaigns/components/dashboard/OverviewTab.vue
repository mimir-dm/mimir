<template>
  <div class="overview-tab">
    <!-- Stage Header -->
    <StageHeader
      v-if="stageInfo"
      :title="stageInfo.title || ''"
      :subtitle="stageInfo.subtitle || ''"
    />

    <!-- Quick Stats -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-value">{{ modulesCount }}</div>
        <div class="stat-label">Modules</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ documentsCount }}</div>
        <div class="stat-label">Documents</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ sessionsCount }}</div>
        <div class="stat-label">Sessions</div>
      </div>
    </div>

    <!-- Campaign Summary -->
    <div class="summary-section" v-if="campaignDescription">
      <h3>About This Campaign</h3>
      <p class="summary-text">{{ campaignDescription }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { ModuleService } from '@/services/ModuleService'
import StageHeader from '../StageLanding/StageHeader.vue'
import type { Campaign, BoardConfig } from '@/types'

const props = defineProps<{
  campaign?: Campaign
  boardConfig?: BoardConfig
  documents?: any[]
}>()

// Module data
const modules = ref<any[]>([])
const modulesLoading = ref(false)

// Computed stats
const modulesCount = computed(() => modules.value.length)
const documentsCount = computed(() => props.documents?.length || 0)
const sessionsCount = computed(() => {
  return modules.value.reduce((total, module) =>
    total + (module.actual_sessions || module.session_count || 0), 0
  )
})

// Campaign description (may come from extended campaign data)
const campaignDescription = computed(() => {
  const c = props.campaign as any
  return c?.description || c?.summary || null
})

// Get stage info from board configuration
const stageInfo = computed(() => {
  if (!props.boardConfig || !props.campaign?.status) {
    return null
  }
  const currentStageInfo = props.boardConfig.stages?.find(
    (s: any) => s.key === props.campaign?.status
  ) as any
  if (!currentStageInfo) {
    return null
  }
  return {
    title: currentStageInfo.display_name as string | undefined,
    subtitle: (currentStageInfo.description || currentStageInfo.help_text) as string | undefined
  }
})

// Load modules
async function loadModules() {
  if (!props.campaign?.id) return

  modulesLoading.value = true
  try {
    modules.value = await ModuleService.list(props.campaign.id)
  } catch (e) {
    console.error('Failed to load modules:', e)
  } finally {
    modulesLoading.value = false
  }
}

// Watch for campaign changes
watch(() => props.campaign?.id, () => {
  loadModules()
}, { immediate: true })

onMounted(() => {
  loadModules()
})
</script>

<style scoped>
.overview-tab {
  padding: var(--spacing-lg, 16px);
  max-width: 900px;
  height: 100%;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg, 16px);
}

/* Stats Grid */
.stats-grid {
  display: flex;
  gap: var(--spacing-md, 12px);
}

.stat-card {
  background: var(--color-surface, #1a1a1a);
  border: 1px solid var(--color-border, #333);
  border-radius: var(--radius-md, 8px);
  padding: var(--spacing-md, 12px) var(--spacing-lg, 16px);
  display: flex;
  align-items: baseline;
  gap: var(--spacing-sm, 8px);
}

.stat-value {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--color-primary, #4a9eff);
  line-height: 1;
}

.stat-label {
  font-size: 0.875rem;
  color: var(--color-text-muted, #888);
}

/* Summary Section */
.summary-section {
  background: var(--color-surface, #1a1a1a);
  border: 1px solid var(--color-border, #333);
  border-radius: var(--radius-md, 8px);
  padding: var(--spacing-lg, 16px);
}

.summary-section h3 {
  margin: 0 0 var(--spacing-sm, 8px) 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text, #e0e0e0);
}

.summary-text {
  margin: 0;
  color: var(--color-text-muted, #888);
  line-height: 1.6;
}

/* Responsive */
@media (max-width: 600px) {
  .stats-grid {
    flex-direction: column;
  }
}
</style>
