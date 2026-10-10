<template>
  <section class="settings-panel" aria-labelledby="manage-campaigns-title" data-testid="manage-campaigns-panel">
    <header class="panel-header">
      <h2 id="manage-campaigns-title" class="content-title">Manage Campaigns</h2>
      <div class="modal-tabs">
        <button
          class="btn-tab"
          :class="{ 'btn-tab--active': activeTab === 'active' }"
          @click="activeTab = 'active'"
        >
          Active Campaigns
        </button>
        <button
          class="btn-tab"
          :class="{ 'btn-tab--active': activeTab === 'archived' }"
          @click="activeTab = 'archived'"
        >
          Archived Campaigns
        </button>
      </div>
    </header>

    <!-- Active Campaigns Tab -->
    <div v-if="activeTab === 'active'">
      <div v-if="isLoading" class="loading-message">
        Loading campaigns...
      </div>

      <EmptyState
        v-else-if="activeCampaigns.length === 0"
        variant="campaigns"
        title="No active campaigns"
        description="Create a new campaign to get started"
      />

      <div v-else class="campaign-list">
        <div v-for="campaign in activeCampaigns" :key="campaign.id" class="campaign-item">
          <div class="campaign-info">
            <div class="campaign-name">{{ campaign.name }}</div>
            <div class="campaign-meta">
              <span class="campaign-status">{{ getCampaignStatus(campaign) }}</span>
              <span class="campaign-activity">
                Last updated: {{ formatDate(campaign.updated_at) }}
              </span>
            </div>
          </div>
          <button
            @click="handleArchiveCampaign(campaign)"
            class="archive-button"
            title="Archive campaign"
          >
            Archive
          </button>
        </div>
      </div>
    </div>

    <!-- Archived Campaigns Tab -->
    <div v-if="activeTab === 'archived'">
      <div v-if="isLoading" class="loading-message">
        Loading archived campaigns...
      </div>

      <EmptyState
        v-else-if="archivedCampaigns.length === 0"
        variant="campaigns"
        title="No archived campaigns"
        description="Archived campaigns will appear here"
      />

      <div v-else class="campaign-list">
        <div v-for="campaign in archivedCampaigns" :key="campaign.id" class="campaign-item archived">
          <div class="campaign-info">
            <div class="campaign-name">{{ campaign.name }}</div>
            <div class="campaign-meta">
              <span class="campaign-status">{{ getCampaignStatus(campaign) }}</span>
              <span class="campaign-activity">
                Archived: {{ formatDate(campaign.archived_at!) }}
              </span>
            </div>
          </div>
          <div class="campaign-actions">
            <button
              @click="handleUnarchiveCampaign(campaign)"
              class="unarchive-button"
              title="Restore campaign"
            >
              Restore
            </button>
            <button
              @click="handleDeleteCampaign(campaign)"
              class="delete-button"
              title="Delete campaign permanently"
            >
              Delete
            </button>
          </div>
        </div>
      </div>
    </div>
    <!-- Delete Confirmation Modal -->
    <AppModal
      :visible="showDeleteModal"
      title="Delete Campaign"
      size="sm"
      :stack-index="1"
      @close="cancelDelete"
    >
      <p>Are you sure you want to permanently delete "<strong>{{ campaignToDelete?.name }}</strong>"?</p>
      <p class="warning-text">This action cannot be undone.</p>

      <div v-if="deleteError" class="error-message">
        {{ deleteError }}
      </div>

      <div class="delete-options">
        <label class="checkbox-label">
          <input
            type="checkbox"
            v-model="deleteFiles"
          />
          Also delete all campaign files and directories
        </label>
      </div>

      <template #footer>
        <button @click="cancelDelete" class="btn btn-secondary">
          Cancel
        </button>
        <button @click="confirmDelete" class="btn btn-danger">
          Delete Campaign
        </button>
      </template>
    </AppModal>
  </section>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useCampaignStore } from '@/stores/campaigns'
import AppModal from '@/components/shared/AppModal.vue'
import EmptyState from '@/shared/components/ui/EmptyState.vue'
import type { Campaign } from '@/types/api'

/**
 * Manage Campaigns, a section of Settings (MIMIR-T-0692; it was a modal).
 */

const campaignStore = useCampaignStore()
const activeTab = ref<'active' | 'archived'>('active')
const showDeleteModal = ref(false)
const campaignToDelete = ref<Campaign | null>(null)
const deleteFiles = ref(false)
const deleteError = ref<string | null>(null)

const activeCampaigns = computed(() => campaignStore.campaigns)
const archivedCampaigns = computed(() => campaignStore.archivedCampaigns)
const isLoading = computed(() => campaignStore.loading)

onMounted(loadCampaigns)

// Load campaigns when switching tabs
watch(activeTab, loadCampaigns)

async function loadCampaigns() {
  if (activeTab.value === 'active') {
    await campaignStore.fetchCampaigns()
  } else {
    await campaignStore.fetchArchivedCampaigns()
  }
}

// Helper to derive status from campaign data
function getCampaignStatus(campaign: Campaign): string {
  if (campaign.archived_at) return 'Archived'
  return 'Active'
}

async function handleArchiveCampaign(campaign: Campaign) {
  const success = await campaignStore.archiveCampaign(campaign.id)
  if (success) {
    // Refresh both lists
    await campaignStore.fetchCampaigns()
    await campaignStore.fetchArchivedCampaigns()
  } else {
    // Show error in a non-system way - could be improved with toast notifications
    console.error(`Failed to archive campaign: ${campaignStore.error}`)
  }
}

async function handleUnarchiveCampaign(campaign: Campaign) {
  const success = await campaignStore.unarchiveCampaign(campaign.id)
  if (success) {
    // Refresh both lists
    await campaignStore.fetchCampaigns()
    await campaignStore.fetchArchivedCampaigns()
  } else {
    // Show error in a non-system way - could be improved with toast notifications
    console.error(`Failed to restore campaign: ${campaignStore.error}`)
  }
}

function handleDeleteCampaign(campaign: Campaign) {
  campaignToDelete.value = campaign
  deleteFiles.value = false
  showDeleteModal.value = true
}

async function confirmDelete() {
  if (!campaignToDelete.value) return

  deleteError.value = null
  const success = await campaignStore.deleteCampaign(campaignToDelete.value.id, deleteFiles.value)
  if (success) {
    showDeleteModal.value = false
    campaignToDelete.value = null
    deleteFiles.value = false
    // Refresh archived list
    await campaignStore.fetchArchivedCampaigns()
  } else {
    deleteError.value = campaignStore.error || 'Failed to delete campaign'
  }
}

function cancelDelete() {
  showDeleteModal.value = false
  campaignToDelete.value = null
  deleteFiles.value = false
  deleteError.value = null
}


function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleDateString()
}
</script>

<style scoped>
/* Section header and footer inside the Settings pane (MIMIR-T-0692). */
.panel-header {
  margin-bottom: var(--spacing-lg);
}

.panel-header .content-title {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 var(--spacing-sm) 0;
}

.panel-header .content-description {
  color: var(--color-text-secondary);
  line-height: 1.5;
  margin: 0;
}

/* Domain-specific styles */
.modal-tabs {
  display: flex;
  border-bottom: 1px solid var(--color-border);
}

.modal-tabs .btn-tab {
  flex: 1;
}

.loading-message {
  text-align: center;
  color: var(--color-text-secondary);
  padding: var(--spacing-xl) 0;
}

.campaign-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.campaign-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  transition: background-color var(--transition-fast);
}

.campaign-item:hover {
  background: var(--color-neutral-tint);
}

.campaign-item.archived {
  opacity: 0.8;
}

.campaign-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.campaign-name {
  font-weight: 500;
  color: var(--color-text);
  font-size: 1rem;
}

.campaign-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.campaign-status {
  font-weight: 500;
  color: var(--color-primary-text);
}

.campaign-actions {
  display: flex;
  gap: var(--spacing-sm);
}

.archive-button {
  background: var(--color-warning-tint);
  color: var(--color-warning-on-tint);
  border: 1px solid var(--color-warning-tint-border);
  border-radius: var(--radius-sm);
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.archive-button:hover {
  background: var(--color-warning-tint-strong);
  color: var(--color-warning-on-tint);
}

.unarchive-button {
  background: var(--color-success-tint);
  color: var(--color-success-on-tint);
  border: 1px solid var(--color-success-tint-border);
  border-radius: var(--radius-sm);
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.unarchive-button:hover {
  background: var(--color-success-tint-strong);
  color: var(--color-success-on-tint);
}

.delete-button {
  background: var(--color-error-tint);
  color: var(--color-error-text);
  border: 1px solid var(--color-error-tint-border);
  border-radius: var(--radius-sm);
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.delete-button:hover {
  background: var(--color-error-tint-strong);
  color: var(--color-error-on-tint);
}

.warning-text {
  color: var(--color-error-600);
  font-weight: 500;
  margin-top: var(--spacing-sm);
}

.error-message {
  background: var(--color-error-tint);
  color: var(--color-error-on-tint);
  border: 1px solid var(--color-error-tint-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-sm) var(--spacing-md);
  margin: var(--spacing-md) 0;
  font-size: 0.875rem;
}

.delete-options {
  margin: var(--spacing-lg) 0;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  cursor: pointer;
  color: var(--color-text);
}

.checkbox-label input[type="checkbox"] {
  cursor: pointer;
  margin: 0;
}
</style>
