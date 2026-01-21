<template>
  <div class="stage-landing">
    <!-- Module Header -->
    <StageHeader
      :module="module"
      :stage-info="stageInfo"
    />

    <!-- Module Export Dialog -->
    <ModuleExportDialog
      :visible="showExportDialog"
      :module-id="module.id"
      :module-name="module.name"
      :module-number="module.module_number"
      :campaign-id="module.campaign_id"
      @close="showExportDialog = false"
    />

    <!-- Next Steps (for planning/prep stages) -->
    <StageTransitionCard
      v-if="stage !== 'ready' && stage !== 'active' && stage !== 'completed'"
      :available="nextStageAvailable"
      :prompt="nextStagePrompt"
      :next-stage-name="nextStageName"
      @transition="transitionToNextStage"
    />

    <!-- Play Mode Button (for ready stage) -->
    <div v-if="stage === 'ready'" class="play-mode-section mt-8">
      <div class="play-mode-card">
        <div class="play-mode-content">
          <h2>Ready to Play</h2>
          <p>Your module is prepped and ready. Enter Play Mode to run your session with quick access to documents, monsters, and notes.</p>
        </div>
        <div class="play-mode-actions">
          <button class="play-mode-button" @click="enterPlayMode">
            Enter Play Mode
          </button>
          <button class="print-button" @click="showExportDialog = true">
            Print
          </button>
        </div>
      </div>

      <!-- Option to mark complete without playing -->
      <div class="complete-option mt-4">
        <button class="complete-button" @click="transitionToNextStage">
          Mark Module Complete
        </button>
      </div>
    </div>

    <!-- Active Stage - Continue or Complete -->
    <div v-if="stage === 'active'" class="play-mode-section mt-8">
      <div class="play-mode-card active">
        <div class="play-mode-content">
          <h2>Module In Progress</h2>
          <p>This module is currently being run. Continue in Play Mode or mark it complete when finished.</p>
        </div>
        <div class="play-mode-actions">
          <button class="play-mode-button" @click="enterPlayMode">
            Continue Playing
          </button>
          <button class="print-button" @click="showExportDialog = true">
            Print
          </button>
        </div>
      </div>

      <div class="complete-option mt-4">
        <button class="complete-button" @click="transitionToNextStage">
          Mark Module Complete
        </button>
      </div>
    </div>

    <!-- Completed Stage Summary -->
    <div v-if="stage === 'completed'" class="completed-section mt-8">
      <div class="completed-card">
        <h2>Module Completed</h2>
        <p>This module has been completed. You can still view its documents and monsters.</p>
      </div>
    </div>

    <!-- Monster Tagging (for all stages except completed) -->
    <div v-if="showMonsters" class="mt-8">
      <ModuleMonsters
        :module-id="module.id"
        :module-name="module.name"
        :module-number="module.module_number"
        :campaign-id="module.campaign_id"
      />
    </div>

    <!-- Module Maps (for all stages except completed) -->
    <div v-if="showMonsters" class="mt-8">
      <ModuleMaps
        :module-id="module.id"
        :campaign-id="module.campaign_id"
      />
    </div>

    <!-- Campaign NPCs (for all stages) -->
    <div v-if="module.campaign_id" class="mt-8">
      <ModuleNPCs
        :module-id="module.id"
        :campaign-id="module.campaign_id"
      />
    </div>

    <!-- User Documents (for all stages) -->
    <div v-if="module.campaign_id" class="mt-8">
      <ModuleUserDocuments
        :module-id="module.id"
        :campaign-id="module.campaign_id"
      />
    </div>

    <!-- Stage-Specific Content from Backend -->
    <div class="stage-content-section" v-if="stageContent">
      <div :class="`stage-${stage}`">
        <div class="activity-section" v-html="stageContent"></div>
      </div>
    </div>

    <!-- Document Progress Indicator (for prep stages) -->
    <div v-if="documentProgress.total > 0 && stage !== 'ready' && stage !== 'active' && stage !== 'completed'" class="progress-section mt-6">
      <h3>Document Progress</h3>
      <div class="progress-bar">
        <div
          class="progress-fill"
          :style="{ width: `${documentProgress.percentage}%` }"
        ></div>
      </div>
      <p>{{ documentProgress.completed }} of {{ documentProgress.total }} documents completed</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import type { Module, BoardConfig, Document } from '@/types'
import StageHeader from './stage/StageHeader.vue'
import StageTransitionCard from './stage/StageTransitionCard.vue'
import ModuleMonsters from './ModuleMonsters.vue'
import ModuleMaps from './ModuleMaps.vue'
import ModuleNPCs from './ModuleNPCs.vue'
import ModuleUserDocuments from './ModuleUserDocuments.vue'
import ModuleExportDialog from '@/components/print/ModuleExportDialog.vue'
import { useModuleStage } from '../composables/useModuleStage'

interface Props {
  module: Module
  stage: string
  boardConfig: BoardConfig | null
  documents: Document[]
}

const props = defineProps<Props>()
const router = useRouter()

// Convert props to refs for composables
const moduleRef = computed(() => props.module)
const stageRef = computed(() => props.stage)
const boardConfigRef = computed(() => props.boardConfig)
const documentsRef = computed(() => props.documents)

// Use composables for logic
const {
  stageInfo,
  documentProgress,
  nextStageAvailable,
  nextStageName,
  nextStagePrompt,
  transitionToNextStage
} = useModuleStage(moduleRef, stageRef, boardConfigRef, documentsRef)

// Stage content from backend configuration
const stageContent = ref<string>('')
const showMonsters = computed(() => props.stage !== 'completed')
const showExportDialog = ref(false)

// Navigate to Play Mode
function enterPlayMode() {
  router.push({ name: 'module-play', params: { id: props.module.id } })
}

// Load stage-specific content from backend
async function loadStageContent() {
  if (!props.boardConfig || !props.stage) return

  try {
    // Get stage content from board config or fetch from backend
    const currentStageConfig = props.boardConfig.stages?.find((s: any) => s.key === props.stage)

    // Get content from stage configuration
    const content = (currentStageConfig as any)?.content

    if (content) {
      stageContent.value = content
    } else {
      // Optionally fetch from backend if not in config
      const response = await invoke<{ content: string }>('get_stage_content', {
        stage: props.stage,
        moduleType: (props.module as any)?.module_type || 'standard'
      })
      stageContent.value = response.content
    }
  } catch (error) {
    stageContent.value = ''
  }
}

// Load data on mount
onMounted(async () => {
  await loadStageContent()
})
</script>

<style scoped>
.stage-landing {
  padding: 1.5rem;
}

/* Play Mode Section */
.play-mode-section {
  max-width: 600px;
  margin-left: auto;
  margin-right: auto;
}

.play-mode-card {
  background: var(--color-surface);
  border: 2px solid var(--color-primary);
  border-radius: 0.75rem;
  padding: 2rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 1.5rem;
}

.play-mode-card.active {
  border-color: var(--color-accent, #e67e22);
}

.play-mode-content h2 {
  font-size: 1.5rem;
  font-weight: 600;
  margin: 0 0 0.5rem 0;
  color: var(--color-text);
}

.play-mode-content p {
  margin: 0;
  color: var(--color-text-muted);
  line-height: 1.5;
}

.play-mode-button {
  padding: 1rem 2.5rem;
  font-size: 1.1rem;
  font-weight: 600;
  background: var(--color-primary);
  color: white;
  border: none;
  border-radius: 0.5rem;
  cursor: pointer;
  transition: all 0.2s;
}

.play-mode-button:hover {
  background: var(--color-primary-dark, #2563eb);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.play-mode-actions {
  display: flex;
  gap: 1rem;
  align-items: center;
}

.print-button {
  padding: 1rem 1.5rem;
  font-size: 1rem;
  font-weight: 500;
  background: var(--color-surface);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: 0.5rem;
  cursor: pointer;
  transition: all 0.2s;
}

.print-button:hover {
  background: var(--color-surface-variant);
  border-color: var(--color-text-muted);
}

.complete-option {
  text-align: center;
}

.complete-button {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  background: transparent;
  color: var(--color-text-muted);
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  cursor: pointer;
  transition: all 0.2s;
}

.complete-button:hover {
  background: var(--color-surface);
  color: var(--color-text);
  border-color: var(--color-text-muted);
}

/* Completed Section */
.completed-section {
  max-width: 600px;
  margin-left: auto;
  margin-right: auto;
}

.completed-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 0.75rem;
  padding: 2rem;
  text-align: center;
}

.completed-card h2 {
  font-size: 1.25rem;
  font-weight: 600;
  margin: 0 0 0.5rem 0;
  color: var(--color-success, #10b981);
}

.completed-card p {
  margin: 0;
  color: var(--color-text-muted);
}

/* Progress Section */
.progress-section {
  background: var(--color-surface);
  padding: 1.5rem;
  border-radius: 0.5rem;
}

.progress-bar {
  width: 100%;
  height: 20px;
  background: var(--color-base-300);
  border-radius: 10px;
  overflow: hidden;
  margin: 1rem 0;
}

.progress-fill {
  height: 100%;
  background: var(--color-primary);
  transition: width 0.3s ease;
}

.stage-content-section {
  margin: 2rem 0;
}

.activity-section {
  background: var(--color-surface);
  padding: 1.5rem;
  border-radius: 0.5rem;
}
</style>