<template>
  <div class="session-tab">
    <!-- Active/Ready modules that can be played -->
    <section class="module-section" v-if="readyModules.length > 0">
      <h2>Ready to Play</h2>
      <div class="module-cards">
        <div
          v-for="mod in readyModules"
          :key="mod.id"
          class="module-card playable"
        >
          <div class="module-info">
            <h3>Module #{{ mod.module_number }}: {{ mod.name }}</h3>
            <div class="module-meta">
              <span class="stage-badge" :class="mod.status">{{ formatStage(mod.status) }}</span>
              <span class="session-count">{{ mod.sessions_completed || 0 }} sessions played</span>
            </div>
          </div>
          <button
            class="btn btn-primary start-button"
            @click="startSession(mod)"
          >
            Start Session
          </button>
        </div>
      </div>
    </section>

    <!-- Modules in preparation -->
    <section class="module-section" v-if="preparingModules.length > 0">
      <h2>In Preparation</h2>
      <div class="module-cards">
        <div
          v-for="mod in preparingModules"
          :key="mod.id"
          class="module-card preparing"
        >
          <div class="module-info">
            <h3>Module #{{ mod.module_number }}: {{ mod.name }}</h3>
            <div class="module-meta">
              <span class="stage-badge" :class="mod.status">{{ formatStage(mod.status) }}</span>
              <span class="prep-hint">Complete module setup to play</span>
            </div>
          </div>
          <button
            class="btn btn-ghost"
            @click="goToModule(mod)"
          >
            View Module
          </button>
        </div>
      </div>
    </section>

    <!-- Completed modules -->
    <section class="module-section" v-if="completedModules.length > 0">
      <h2>Completed</h2>
      <div class="module-cards">
        <div
          v-for="mod in completedModules"
          :key="mod.id"
          class="module-card completed"
        >
          <div class="module-info">
            <h3>Module #{{ mod.module_number }}: {{ mod.name }}</h3>
            <div class="module-meta">
              <span class="stage-badge completed">Completed</span>
              <span class="session-count">{{ mod.sessions_completed || 0 }} sessions</span>
            </div>
          </div>
          <button
            class="btn btn-ghost"
            @click="goToModule(mod)"
          >
            Review
          </button>
        </div>
      </div>
    </section>

    <!-- Empty state -->
    <div v-if="modules.length === 0 && !loading" class="empty-state">
      <div class="empty-icon">🎲</div>
      <h3>No Modules Yet</h3>
      <p>Create a module in the Modules tab to start planning your sessions.</p>
      <button class="btn btn-primary" @click="goToModules">
        Go to Modules
      </button>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="loading-state">
      Loading modules...
    </div>

    <!-- Nested route for play mode -->
    <router-view />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ModuleService } from '@/services/ModuleService'
import type { Campaign, BoardConfig } from '@/types'

const props = defineProps<{
  campaign?: Campaign
  boardConfig?: BoardConfig
  documents?: any[]
}>()

const router = useRouter()

// Local state
const modules = ref<any[]>([])
const loading = ref(false)

// Categorize modules by readiness
const readyModules = computed(() =>
  modules.value.filter(m => m.status === 'ready' || m.status === 'active')
)

const preparingModules = computed(() =>
  modules.value.filter(m => m.status === 'concept' || m.status === 'design')
)

const completedModules = computed(() =>
  modules.value.filter(m => m.status === 'completed')
)

// Format stage name
function formatStage(stage: string): string {
  return stage.charAt(0).toUpperCase() + stage.slice(1).replace(/_/g, ' ')
}

// Load modules
async function loadModules() {
  if (!props.campaign?.id) return

  loading.value = true
  try {
    modules.value = await ModuleService.list(props.campaign.id)
  } catch (e) {
    console.error('Failed to load modules:', e)
  } finally {
    loading.value = false
  }
}

// Start a play session
function startSession(mod: any) {
  if (props.campaign?.id) {
    router.push(`/campaigns/${props.campaign.id}/dashboard/session/${mod.id}/play`)
  }
}

// Navigate to module details
function goToModule(mod: any) {
  if (props.campaign?.id) {
    router.push(`/campaigns/${props.campaign.id}/dashboard/modules/${mod.id}`)
  }
}

// Navigate to modules tab
function goToModules() {
  if (props.campaign?.id) {
    router.push(`/campaigns/${props.campaign.id}/dashboard/modules`)
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
.session-tab {
  padding: var(--spacing-lg, 16px);
  height: 100%;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xl, 24px);
}

.module-section h2 {
  margin: 0 0 var(--spacing-md, 12px) 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text, #e0e0e0);
}

.module-cards {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md, 12px);
}

.module-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-md, 12px) var(--spacing-lg, 16px);
  background: var(--color-surface, #1a1a1a);
  border: 1px solid var(--color-border, #333);
  border-radius: var(--radius-md, 8px);
  transition: all 0.2s;
}

.module-card.playable {
  border-left: 3px solid var(--color-success, #10b981);
}

.module-card.playable:hover {
  border-color: var(--color-success, #10b981);
  background: var(--color-surface-variant, #252525);
}

.module-card.preparing {
  opacity: 0.7;
}

.module-card.completed {
  opacity: 0.6;
}

.module-info {
  flex: 1;
}

.module-info h3 {
  margin: 0 0 var(--spacing-xs, 4px) 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text, #e0e0e0);
}

.module-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-md, 12px);
  font-size: 0.875rem;
}

.stage-badge {
  padding: 2px 8px;
  font-size: 0.75rem;
  font-weight: 500;
  text-transform: uppercase;
  border-radius: 4px;
  background: var(--color-base-300, #333);
  color: var(--color-text-muted, #888);
}

.stage-badge.ready,
.stage-badge.active {
  background: var(--color-success, #10b981);
  color: white;
}

.stage-badge.concept,
.stage-badge.design {
  background: var(--color-info, #3b82f6);
  color: white;
}

.stage-badge.completed {
  background: var(--color-base-500, #555);
  color: white;
}

.session-count {
  color: var(--color-text-muted, #888);
}

.prep-hint {
  color: var(--color-text-muted, #888);
  font-style: italic;
}

.start-button {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs, 4px);
}

/* Empty/Loading states */
.empty-state,
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: var(--spacing-md, 12px);
  text-align: center;
  color: var(--color-text-muted, #888);
}

.empty-icon {
  font-size: 3rem;
  opacity: 0.5;
}

.empty-state h3 {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text, #e0e0e0);
}

.empty-state p {
  margin: 0;
  font-size: 0.875rem;
}
</style>
