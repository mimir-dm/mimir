<template>
  <MainLayout>
    <div class="module-list-view">
      <div class="header">
        <h1>Modules</h1>
        <button @click="showCreateModal = true" class="btn btn-primary">
          + New Module
        </button>
      </div>
      
      <!-- Create Module Modal -->
      <AppModal
        :visible="showCreateModal"
        title="Create New Module"
        size="sm"
        @close="showCreateModal = false"
      >
        <div class="form-group">
          <label for="module-name">Module Name:</label>
          <input
            id="module-name"
            v-model="newModuleName"
            type="text"
            placeholder="Enter module name"
            @keyup.enter="confirmCreateModule"
          />
        </div>
        <div class="form-group">
          <label for="module-type">Module Type:</label>
          <select id="module-type" v-model="newModuleType">
            <option value="standard">Standard Adventure</option>
            <option value="mystery">Mystery</option>
            <option value="dungeon">Dungeon Crawl</option>
            <option value="heist">Heist</option>
            <option value="horror">Horror</option>
            <option value="political">Political Intrigue</option>
          </select>
        </div>
        <div class="form-group">
          <label for="module-sessions">Expected Sessions:</label>
          <input
            id="module-sessions"
            v-model.number="newModuleSessions"
            type="number"
            min="1"
            placeholder="4"
            @keyup.enter="confirmCreateModule"
          />
        </div>

        <template #footer>
          <button @click="showCreateModal = false" class="btn btn-secondary">
            Cancel
          </button>
          <button @click="confirmCreateModule" class="btn btn-primary">
            Create Module
          </button>
        </template>
      </AppModal>

      <div v-if="loading" class="loading-state">
        Loading modules...
      </div>

      <EmptyState
        v-else-if="modules.length === 0"
        variant="campaigns"
        title="No modules yet"
        description="Create your first module to get started"
      >
        <template #action>
          <button @click="showCreateModal = true" class="btn btn-primary">
            + New Module
          </button>
        </template>
      </EmptyState>

      <table v-else class="table table-rounded table-hover">
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
                {{ formatStatus(module.status) }}
              </span>
            </td>
            <td class="actions-cell">
              <router-link :to="`/modules/${module.id}/board`" class="btn btn-primary btn-sm">
                Open Board
              </router-link>
              <button @click="deleteModule(module.id)" class="btn btn-danger btn-sm">
                Delete
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </MainLayout>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { ModuleService } from '@/services/ModuleService'
import MainLayout from '../../../shared/components/layout/MainLayout.vue'
import AppModal from '@/components/shared/AppModal.vue'
import EmptyState from '../../../shared/components/ui/EmptyState.vue'

const route = useRoute()
const campaignId = parseInt(route.params.id as string)

// Log for debugging

interface Module {
  id: number
  campaign_id: number
  name: string
  module_number: number
  status: string
  expected_sessions: number
  actual_sessions: number
  created_at: string
  started_at: string | null
  completed_at: string | null
}

const modules = ref<Module[]>([])
const loading = ref(false)
const showCreateModal = ref(false)
const newModuleName = ref('')
const newModuleType = ref('standard')
const newModuleSessions = ref(4)

const loadModules = async () => {
  loading.value = true
  try {
    modules.value = await ModuleService.list(campaignId)
  } catch (error) {
  } finally {
    loading.value = false
  }
}

const confirmCreateModule = async () => {
  
  if (!newModuleName.value.trim()) {
    return
  }
  
  if (newModuleSessions.value < 1) {
    return
  }
  
  if (isNaN(campaignId)) {
    return
  }
  
  try {
    const newModule = await ModuleService.create({
      campaign_id: campaignId,
      name: newModuleName.value,
      module_type: newModuleType.value
    })
    // Reset form and close modal
    newModuleName.value = ''
    newModuleType.value = 'standard'
    newModuleSessions.value = 4
    showCreateModal.value = false
    
    // Reload modules
    await loadModules()
  } catch (error) {
    alert(`Failed to create module: ${error}`)
  }
}

const deleteModule = async (id: number) => {
  if (!confirm('Are you sure you want to delete this module?')) return
  
  try {
    await ModuleService.delete(id)
    await loadModules()
  } catch (error) {
    alert('Failed to delete module')
  }
}

const formatStatus = (status: string): string => {
  return status.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())
}

const formatDate = (dateString: string): string => {
  if (!dateString) return ''
  return new Date(dateString).toLocaleDateString()
}

// Watch for campaign ID changes in route
watch(() => route.params.id, (newId, oldId) => {
  if (newId !== oldId && newId) {
    // Reload modules for new campaign
    loadModules()
  }
})

onMounted(() => {
  loadModules()
})
</script>

<style scoped>
.module-list-view {
  padding: var(--spacing-xl);
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-xl);
}

.header h1 {
  margin: 0;
}

.loading-state {
  text-align: center;
  padding: var(--spacing-xl);
  color: var(--color-text-secondary);
}

/* Table-specific styles */
.module-name strong {
  color: var(--color-text);
}

.actions-cell {
  text-align: right;
  white-space: nowrap;
}

.actions-cell .btn {
  margin-left: var(--spacing-sm);
}

.status-badge {
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
}

.status-badge.planning {
  background-color: var(--color-info-bg);
  color: var(--color-info);
}

.status-badge.development {
  background-color: var(--color-warning-bg);
  color: var(--color-warning);
}

.status-badge.ready {
  background-color: var(--color-success-bg);
  color: var(--color-success);
}

.status-badge.active {
  background-color: var(--color-primary-100);
  color: var(--color-primary-600);
}

.status-badge.completed {
  background-color: var(--color-success);
  color: white;
}

.progress-bar {
  height: 8px;
  background-color: var(--color-surface-variant);
  border-radius: 4px;
  overflow: hidden;
  margin-top: var(--spacing-sm);
}

.progress-fill {
  height: 100%;
  background-color: var(--color-primary-400);
  transition: width var(--transition-base);
}
</style>