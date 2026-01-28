<template>
  <div class="campaign-selector">
    <div class="selector-dropdown" :class="{ 'is-open': isOpen }" @click="toggleDropdown">
      <div class="selector-current">
        <span v-if="selectedCampaign" class="campaign-name">
          {{ selectedCampaign.name }}
        </span>
        <span v-else class="no-selection">
          Select a campaign...
        </span>
        <svg class="dropdown-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="6 9 12 15 18 9"></polyline>
        </svg>
      </div>

      <div v-if="isOpen" class="dropdown-menu" @click.stop>
        <div v-if="isLoading" class="dropdown-loading">
          Loading campaigns...
        </div>

        <div v-else-if="error" class="dropdown-error">
          {{ error }}
        </div>

        <div v-else-if="campaigns.length === 0" class="dropdown-empty">
          <p>No campaigns found</p>
          <router-link to="/campaigns/new" class="create-link" @click="closeDropdown">
            Create your first campaign
          </router-link>
        </div>

        <div v-else class="dropdown-options">
          <div
            v-for="campaign in campaigns"
            :key="campaign.id"
            class="dropdown-option"
            :class="{ 'is-selected': campaign.id === selectedCampaignId }"
            @click="selectCampaign(campaign.id)"
          >
            <div class="option-content">
              <div class="option-name">{{ campaign.name }}</div>
              <div class="option-status">{{ getCampaignStatus(campaign) }}</div>
            </div>
            <svg v-if="campaign.id === selectedCampaignId" class="check-icon" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
          </div>

          <div class="dropdown-divider"></div>

          <router-link
            to="/campaigns/new"
            class="dropdown-action"
            @click="closeDropdown"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="12" y1="5" x2="12" y2="19"></line>
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
            Create New Campaign
          </router-link>

          <button
            class="dropdown-action"
            @click="openImportDialog"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
              <polyline points="7 10 12 15 17 10"></polyline>
              <line x1="12" y1="15" x2="12" y2="3"></line>
            </svg>
            Import Campaign
          </button>
        </div>
      </div>
    </div>

    <!-- Import Dialog -->
    <CampaignArchiveImportDialog
      :visible="showImportDialog"
      @close="showImportDialog = false"
      @imported="handleCampaignImported"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useCampaignStore } from '@/stores/campaigns'
import { storeToRefs } from 'pinia'
import CampaignArchiveImportDialog from '@/components/campaigns/CampaignArchiveImportDialog.vue'
import type { Campaign } from '@/types/api'

const router = useRouter()
const campaignStore = useCampaignStore()
const { campaigns, loading, error } = storeToRefs(campaignStore)

// Local state for dropdown and selection
const selectedCampaignId = ref<string | null>(null)
const isOpen = ref(false)
const showImportDialog = ref(false)

// Computed properties
const isLoading = computed(() => loading.value)
const selectedCampaign = computed(() => {
  if (!selectedCampaignId.value) return null
  return campaigns.value.find(c => c.id === selectedCampaignId.value) || null
})

// Helper to derive status from campaign data
function getCampaignStatus(campaign: Campaign): string {
  if (campaign.archived_at) return 'archived'
  return 'active'
}

function toggleDropdown() {
  if (!isOpen.value) {
    // Reload campaigns when opening dropdown
    campaignStore.fetchCampaigns()
  }
  isOpen.value = !isOpen.value
}

function closeDropdown() {
  isOpen.value = false
}

function openImportDialog() {
  closeDropdown()
  showImportDialog.value = true
}

function handleCampaignImported(campaign: Campaign) {
  // Refresh the campaign list and select the imported campaign
  campaignStore.fetchCampaigns()
  selectCampaign(campaign.id)
}

async function selectCampaign(campaignId: string) {
  selectedCampaignId.value = campaignId
  // Also update the current campaign in the store
  await campaignStore.getCampaign(campaignId)
  closeDropdown()

  // Persist selection to localStorage
  localStorage.setItem('selectedCampaignId', campaignId)

  // Navigate to the campaign dashboard
  router.push(`/campaigns/${campaignId}/dashboard`)
}

// Close dropdown when clicking outside
function handleClickOutside(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (!target.closest('.campaign-selector')) {
    closeDropdown()
  }
}

onMounted(async () => {
  // Load campaigns first
  await campaignStore.fetchCampaigns()

  // Then initialize from localStorage
  const storedId = localStorage.getItem('selectedCampaignId')
  if (storedId) {
    selectedCampaignId.value = storedId
    // Load the current campaign to ensure it's in the store
    await campaignStore.getCampaign(storedId)
  } else if (campaigns.value.length > 0) {
    // If no stored selection but we have campaigns, select the first one
    selectedCampaignId.value = campaigns.value[0].id
    localStorage.setItem('selectedCampaignId', campaigns.value[0].id)
    await campaignStore.getCampaign(campaigns.value[0].id)
  }

  // Add click outside listener
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

// Watch for route changes to update campaign context
watch(() => router.currentRoute.value, (route) => {
  // Only update if we're on a campaign route
  if (route.path.startsWith('/campaigns/') && route.params.id) {
    const id = route.params.id as string
    if (id !== selectedCampaignId.value) {
      // Just update the selection, don't navigate again
      selectedCampaignId.value = id
      campaignStore.getCampaign(id)
      localStorage.setItem('selectedCampaignId', id)
    }
  }
  // If we're on a module route, don't change the selection
  // The selection should persist from localStorage
}, { immediate: true })
</script>

<style scoped>
.campaign-selector {
  display: flex;
  align-items: center;
}

.selector-dropdown {
  position: relative;
  min-width: 200px;
}

.selector-current {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-sm) var(--spacing-md);
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.selector-dropdown:hover .selector-current {
  border-color: var(--color-primary-300);
}

.selector-dropdown.is-open .selector-current {
  border-color: var(--color-primary-500);
  box-shadow: 0 0 0 3px var(--color-primary-100);
}

.theme-dark .selector-dropdown.is-open .selector-current {
  box-shadow: 0 0 0 3px var(--color-primary-900);
}

.campaign-name {
  font-weight: 500;
  color: var(--color-text);
}

.no-selection {
  color: var(--color-text-secondary);
  font-style: italic;
}

.dropdown-icon {
  color: var(--color-text-secondary);
  transition: transform var(--transition-fast);
}

.selector-dropdown.is-open .dropdown-icon {
  transform: rotate(180deg);
}

.dropdown-menu {
  position: absolute;
  top: calc(100% + var(--spacing-xs));
  left: 0;
  right: 0;
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  max-height: 320px;
  overflow-y: auto;
  z-index: 1000;
}

.dropdown-loading,
.dropdown-error,
.dropdown-empty {
  padding: var(--spacing-lg);
  text-align: center;
  color: var(--color-text-secondary);
}

.dropdown-error {
  color: var(--color-error);
}

.dropdown-empty p {
  margin: 0 0 var(--spacing-md) 0;
}

.create-link {
  color: var(--color-primary-600);
  text-decoration: none;
  font-weight: 500;
}

.create-link:hover {
  text-decoration: underline;
}

.dropdown-options {
  padding: var(--spacing-xs);
}

.dropdown-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background-color var(--transition-fast);
}

.dropdown-option:hover {
  background-color: var(--color-surface-variant);
}

.dropdown-option.is-selected {
  background-color: var(--color-primary-50);
  color: var(--color-primary-700);
}

.theme-dark .dropdown-option.is-selected {
  background-color: var(--color-primary-900);
  color: var(--color-primary-300);
}

.option-content {
  flex: 1;
}

.option-name {
  font-weight: 500;
}

.option-status {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  text-transform: capitalize;
}

.check-icon {
  color: var(--color-primary-600);
}

.dropdown-divider {
  height: 1px;
  background-color: var(--color-border);
  margin: var(--spacing-xs) 0;
}

.dropdown-action {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-sm);
  color: var(--color-primary-600);
  text-decoration: none;
  font-weight: 500;
  transition: background-color var(--transition-fast);
  width: 100%;
  background: none;
  border: none;
  cursor: pointer;
  font-size: inherit;
  text-align: left;
}

.dropdown-action:hover {
  background-color: var(--color-primary-50);
}

.theme-dark .dropdown-action:hover {
  background-color: var(--color-primary-900);
}
</style>
