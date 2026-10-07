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
        <!-- One row: campaign name, tabs, actions (MIMIR-T-0689) -->
        <header class="dashboard-header">
          <h1 class="dashboard-title" :title="campaign.name">{{ campaign.name }}</h1>
          <DashboardTabs :campaign-id="id" class="dashboard-header-tabs" />
          <div class="header-actions">
            <button @click="showSourcesDialog = true" class="btn btn-secondary btn-sm">
              Sources
            </button>
            <button @click="showPdfDialog = true" class="btn btn-secondary btn-sm">
              PDF
            </button>
            <button @click="showExportDialog = true" class="btn btn-secondary btn-sm">
              Export Archive
            </button>
          </div>
        </header>

        <!-- Tab Content (nested router-view) -->
        <main class="dashboard-content">
          <router-view
            :campaign="campaign"
            :documents="documents"
            @refresh="loadCampaign"
          />
        </main>
      </template>

      <!-- PDF Export Dialog -->
      <CampaignExportDialog
        :visible="showPdfDialog"
        :campaign-id="id"
        :campaign-name="campaign?.name"
        @close="showPdfDialog = false"
      />

      <!-- Archive Export Dialog -->
      <CampaignArchiveExportDialog
        :visible="showExportDialog"
        :campaign="campaign"
        @close="showExportDialog = false"
      />

      <!-- Campaign Sources Modal -->
      <CampaignSourcesModal
        :visible="showSourcesDialog"
        :campaign-id="id"
        @close="showSourcesDialog = false"
        @saved="campaignStore.refreshCampaignSources()"
      />
    </div>
  </MainLayout>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, provide } from 'vue'
import { DocumentService } from '@/services/DocumentService'
import { useApiCall } from '@/composables/useApiCall'
import MainLayout from '@/shared/components/layout/MainLayout.vue'
import DashboardTabs from '../components/dashboard/DashboardTabs.vue'
import CampaignArchiveExportDialog from '@/components/campaigns/CampaignArchiveExportDialog.vue'
import CampaignExportDialog from '@/components/print/CampaignExportDialog.vue'
import CampaignSourcesModal from '@/components/campaigns/CampaignSourcesModal.vue'
import { useCampaignStore } from '@/stores/campaigns'
import type { Campaign } from '@/types'

const props = defineProps<{
  id: string
}>()

const campaignStore = useCampaignStore()

// Local state
const campaign = ref<Campaign | null>(null)
const documents = ref<any[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const showExportDialog = ref(false)
const showPdfDialog = ref(false)
const showSourcesDialog = ref(false)

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
  align-items: stretch;
  column-gap: var(--spacing-md);
  padding: 0 var(--spacing-md);
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
}

.dashboard-title {
  align-self: center;
  /* The only part that gives way: a long name ends in an ellipsis. */
  flex: 0 1 auto;
  min-width: 0;
  max-width: 32ch;
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text);
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.dashboard-header-tabs {
  flex: 0 0 auto;
}

.header-actions {
  align-self: center;
  flex: 0 0 auto;
  display: flex;
  gap: var(--spacing-sm);
  margin-left: auto;
  padding: var(--spacing-sm) 0;
}

/* Narrow windows: the row wraps (actions on their own line) instead of
   squeezing the tabs. */
@media (max-width: 1024px) {
  .dashboard-header {
    flex-wrap: wrap;
  }

  .dashboard-header-tabs {
    flex: 1 1 auto;
    min-width: 0;
  }
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
