<template>
  <div class="modules-section">
    <div class="section-header">
      <h3>{{ title }}</h3>
      <button @click="$emit('createModule')" class="btn btn-primary">
        New Module
      </button>
    </div>

    <div v-if="loading" class="loading-state">
      Loading modules...
    </div>

    <EmptyState
      v-else-if="modules.length === 0"
      variant="campaigns"
      title="No modules yet"
      :description="emptyMessage"
    />

    <table v-else class="table table-rounded table-hover modules-table">
      <thead>
        <tr>
          <th>Module</th>
          <th>Status</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="module in modules" :key="module.id" class="module-row">
          <td class="module-name">
            <strong>Module #{{ module.module_number }}:</strong> {{ module.name }}
          </td>
          <td>
            <span class="status-badge" :class="module.status">
              {{ module.status }}
            </span>
          </td>
          <td class="actions-cell">
            <button
              class="btn btn-warning btn-sm"
              :class="{ disabled: module.status !== 'ready' && module.status !== 'active' }"
              :disabled="module.status !== 'ready' && module.status !== 'active'"
              @click="handlePlayModule(module)"
            >
              Play
            </button>
            <router-link :to="`/modules/${module.id}/board`" class="btn btn-ghost btn-sm">
              Open
            </router-link>
            <button
              class="btn btn-secondary btn-sm"
              @click="openExportDialog(module)"
              title="Export module as PDF"
            >
              PDF
            </button>
          </td>
        </tr>
      </tbody>
    </table>

    <!-- Module Export Dialog -->
    <ModuleExportDialog
      :visible="showExportDialog"
      :module-id="selectedModule?.id ?? null"
      :module-name="selectedModule?.name"
      :module-number="selectedModule?.module_number"
      :campaign-id="campaignId"
      @close="closeExportDialog"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import ModuleExportDialog from '../../../../components/print/ModuleExportDialog.vue'
import EmptyState from '@/shared/components/ui/EmptyState.vue'
import { useDmMapWindow } from '@/composables/useDmMapWindow'

interface ModuleData {
  id: string
  name: string
  module_number: number
  status: string
}

// DM Map window
const { openWindow: openDmMapWindow } = useDmMapWindow()

const props = withDefaults(defineProps<{
  modules: ModuleData[]
  loading: boolean
  title?: string
  emptyMessage?: string
  campaignId?: string
}>(), {
  title: 'Modules',
  emptyMessage: 'No modules yet. Create your first module to get started!'
})

defineEmits<{
  createModule: []
}>()

// Export dialog state
const showExportDialog = ref(false)
const selectedModule = ref<ModuleData | null>(null)

function openExportDialog(module: ModuleData) {
  selectedModule.value = module
  showExportDialog.value = true
}

function closeExportDialog() {
  showExportDialog.value = false
  selectedModule.value = null
}

// Play module - opens DM Map window
async function handlePlayModule(module: ModuleData) {
  if (!props.campaignId) return
  const isReady = module.status === 'ready' || module.status === 'active'
  if (!isReady) return

  try {
    await openDmMapWindow(module.id, props.campaignId)
  } catch (e) {
    console.error('Failed to open DM Map window:', e)
  }
}
</script>

<style scoped>
/* Layout-specific: column widths for this table */
.modules-table {
  table-layout: fixed;
}

.modules-table th:nth-child(1) { width: 55%; }
.modules-table th:nth-child(2) { width: 15%; }
.modules-table th:last-child { width: 30%; }

.actions-cell {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  justify-content: flex-end;
}

.actions-cell .btn {
  min-width: 4rem;
}

/* Disabled state for Play button when module not ready */
.btn.disabled {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}
</style>
