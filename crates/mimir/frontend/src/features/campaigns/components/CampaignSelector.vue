<template>
  <div class="campaign-selector">
    <div class="selector-dropdown" :class="{ 'is-open': isOpen }" @click="toggleDropdown">
      <div class="selector-current">
        <span v-if="selectedCampaign" class="campaign-name" :title="selectedCampaign.name">
          {{ selectedCampaign.name }}
        </span>
        <span v-else class="no-selection">
          Select a campaign...
        </span>
        <ChevronDown class="dropdown-icon" :size="20" aria-hidden="true" />
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
          <button
            class="dropdown-action empty-import-action"
            @click="openImportDialog"
          >
            <Download :size="16" aria-hidden="true" />
            Import Campaign
          </button>
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
            <Check v-if="campaign.id === selectedCampaignId" class="check-icon" :size="16" :stroke-width="3" aria-hidden="true" />
          </div>

          <div class="dropdown-divider"></div>

          <router-link
            to="/campaigns/new"
            class="dropdown-action"
            @click="closeDropdown"
          >
            <Plus :size="16" aria-hidden="true" />
            Create New Campaign
          </router-link>

          <button
            class="dropdown-action"
            @click="openImportDialog"
          >
            <Download :size="16" aria-hidden="true" />
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
import { Check, ChevronDown, Download, Plus } from '@lucide/vue'
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
  box-shadow: 0 0 0 3px var(--color-focus-ring);
}

/* One line; very long names end in an ellipsis (full name in the tooltip). */
.campaign-name {
  font-weight: 500;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 32ch;
}

.no-selection {
  color: var(--color-text-secondary);
  font-style: italic;
}

.dropdown-icon {
  flex-shrink: 0;
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
  color: var(--color-primary-text);
  text-decoration: none;
  font-weight: 500;
}

.create-link:hover {
  text-decoration: underline;
}

.empty-import-action {
  margin-top: var(--spacing-sm);
  justify-content: center;
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
  background-color: var(--color-primary-tint-subtle);
  color: var(--color-primary-on-tint);
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
  color: var(--color-primary-text);
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
  color: var(--color-primary-text);
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
  background-color: var(--color-primary-tint-subtle);
}
</style>
