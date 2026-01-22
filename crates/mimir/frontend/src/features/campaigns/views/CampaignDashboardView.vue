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
          <div class="header-actions">
            <button @click="showExportDialog = true" class="btn btn-secondary btn-sm">
              Export Archive
            </button>
          </div>
        </header>

        <!-- Tab Navigation -->
        <DashboardTabs :campaign-id="id" />

        <!-- Tab Content (nested router-view) -->
        <main class="dashboard-content">
          <router-view
            :campaign="campaign"
            :documents="documents"
            @refresh="loadCampaign"
          />
        </main>
      </template>

      <!-- Export Dialog -->
      <CampaignArchiveExportDialog
        :visible="showExportDialog"
        :campaign="campaign"
        @close="showExportDialog = false"
      />
    </div>
  </MainLayout>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, provide } from 'vue'
import { DocumentService } from '@/services/DocumentService'
import { useApiCall } from '@/shared/composables/useApiCall'
import MainLayout from '@/shared/components/layout/MainLayout.vue'
import DashboardTabs from '../components/dashboard/DashboardTabs.vue'
import CampaignArchiveExportDialog from '@/components/campaigns/CampaignArchiveExportDialog.vue'
import type { Campaign } from '@/types'

const props = defineProps<{
  id: string
}>()

// Local state
const campaign = ref<Campaign | null>(null)
const documents = ref<any[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const showExportDialog = ref(false)

// API call helpers
const { execute: loadCampaignApi } = useApiCall<Campaign>()

// Provide campaign data to child components
provide('campaign', campaign)
provide('documents', documents)
provide('campaignId', props.id)

// Load campaign data
const loadCampaign = async () => {
  loading.value = true
  error.value = null

  try {
    const data = await loadCampaignApi('get_campaign', {
      id: props.id
    })

    if (data) {
      campaign.value = data

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
    documents.value = await DocumentService.listForCampaign(props.id)
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
  background: var(--color-background);
}

.dashboard-loading,
.dashboard-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: var(--spacing-md);
  color: var(--color-text-secondary);
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-primary-500);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.btn-retry {
  padding: var(--spacing-sm) var(--spacing-lg);
  background: var(--color-primary-500);
  color: var(--color-background);
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.btn-retry:hover {
  background: var(--color-primary-600);
}

.dashboard-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-md) var(--spacing-lg);
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
}

.header-info {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
}

.header-info h1 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text);
}

.header-actions {
  display: flex;
  gap: var(--spacing-sm);
}

.btn-sm {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.875rem;
}

.dashboard-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
