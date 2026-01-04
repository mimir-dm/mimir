<template>
  <MainLayout>
    <div class="campaign-dashboard">
      <!-- Loading state -->
      <div v-if="loading" class="dashboard-loading">
        <div class="loading-spinner"></div>
        <p>Loading campaign...</p>
      </div>

      <!-- Error state -->
      <div v-else-if="error" class="dashboard-error">
        <p>{{ error }}</p>
        <button @click="loadCampaign" class="btn-retry">Retry</button>
      </div>

      <!-- Main dashboard -->
      <template v-else-if="campaign">
        <!-- Campaign Header -->
        <header class="dashboard-header">
          <div class="header-info">
            <h1>{{ campaign.name }}</h1>
          </div>
        </header>

        <!-- Tab Navigation -->
        <DashboardTabs :campaign-id="id" />

        <!-- Tab Content (nested router-view) -->
        <main class="dashboard-content">
          <router-view
            :campaign="campaign"
            :board-config="boardConfig"
            :documents="documents"
            @refresh="loadCampaign"
          />
        </main>
      </template>
    </div>
  </MainLayout>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, provide } from 'vue'
import { useSharedContextStore } from '@/stores/sharedContext'
import { DocumentService } from '@/services/DocumentService'
import { useApiCall } from '@/shared/composables/useApiCall'
import MainLayout from '@/shared/components/layout/MainLayout.vue'
import DashboardTabs from '../components/dashboard/DashboardTabs.vue'
import type { Campaign, BoardConfig } from '@/types'

const props = defineProps<{
  id: string
}>()

const contextStore = useSharedContextStore()

// Local state
const campaign = ref<Campaign | null>(null)
const documents = ref<any[]>([])
const boardConfig = ref<BoardConfig | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)

// API call helpers
const { execute: loadBoardApi } = useApiCall<BoardConfig>()
const { execute: loadCampaignApi } = useApiCall<Campaign>()

// Provide campaign data to child components
provide('campaign', campaign)
provide('boardConfig', boardConfig)
provide('documents', documents)
provide('campaignId', props.id)

// Load board configuration
const loadBoardConfiguration = async () => {
  const data = await loadBoardApi('get_board_configuration', {
    boardType: 'campaign'
  })
  if (data) {
    boardConfig.value = data
  }
}

// Load campaign data
const loadCampaign = async () => {
  loading.value = true
  error.value = null

  try {
    // Load board configuration first
    await loadBoardConfiguration()

    const data = await loadCampaignApi('get_campaign', {
      id: parseInt(props.id)
    })

    if (data) {
      campaign.value = data

      // Update context with campaign info
      await contextStore.updateCampaign({
        id: campaign.value.id.toString(),
        name: campaign.value.name,
        currentStage: campaign.value.status || undefined,
        directory_path: campaign.value.directory_path || undefined
      })

      // Clear module/session context
      await contextStore.updateModule({})
      await contextStore.updateSession({})

      // Load documents
      await loadDocuments()
    }
  } catch (e: any) {
    error.value = e.message || 'Failed to load campaign'
  } finally {
    loading.value = false
  }
}

// Load all documents for the campaign
const loadDocuments = async () => {
  try {
    documents.value = await DocumentService.list(undefined, parseInt(props.id))
  } catch (e) {
    console.error('Failed to load documents:', e)
  }
}

// Watch for route changes (campaign ID)
watch(() => props.id, () => {
  loadCampaign()
})

onMounted(() => {
  loadCampaign()
})
</script>

<style scoped>
.campaign-dashboard {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-base-100, #0d0d0d);
}

.dashboard-loading,
.dashboard-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: var(--spacing-md, 12px);
  color: var(--color-text-muted, #888);
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-border, #333);
  border-top-color: var(--color-primary, #4a9eff);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.btn-retry {
  padding: var(--spacing-sm, 8px) var(--spacing-lg, 16px);
  background: var(--color-primary, #4a9eff);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.btn-retry:hover {
  opacity: 0.9;
}

.dashboard-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-md, 12px) var(--spacing-lg, 16px);
  background: var(--color-surface, #1a1a1a);
  border-bottom: 1px solid var(--color-border, #333);
}

.header-info {
  display: flex;
  align-items: center;
  gap: var(--spacing-md, 12px);
}

.header-info h1 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text, #e0e0e0);
}

.dashboard-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
