<template>
  <MainLayout>
    <div class="campaign-list-view">
      <div class="header">
        <h1 class="page-title">Campaigns</h1>
        <router-link to="/campaigns/new" class="btn btn-primary">
          New Campaign
        </router-link>
      </div>

      <div v-if="campaignStore.loading" class="loading">
        Loading campaigns...
      </div>

      <div v-else-if="campaignStore.error" class="error-message">
        {{ campaignStore.error }}
      </div>

      <EmptyState
        v-else-if="campaignStore.campaigns.length === 0"
        variant="campaigns"
        title="No campaigns yet"
        description="Create your first campaign to start your adventure"
      >
        <template #action>
          <router-link to="/campaigns/new" class="btn btn-primary">
            Create Campaign
          </router-link>
        </template>
      </EmptyState>

      <div v-else class="campaign-grid">
        <div
          v-for="campaign in campaignStore.campaigns"
          :key="campaign.id"
          class="card-interactive campaign-card"
          @click="selectCampaign(campaign.id)"
        >
          <h3 class="campaign-name">{{ campaign.name }}</h3>
          <div class="campaign-meta">
            <span class="campaign-status" :class="`status-${campaign.status}`">
              {{ campaign.status }}
            </span>
            <span class="campaign-date">
              Created {{ formatDate(campaign.created_at) }}
            </span>
          </div>
          <p class="campaign-path">{{ campaign.directory_path }}</p>
        </div>
      </div>
    </div>
  </MainLayout>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import MainLayout from '../../../shared/components/layout/MainLayout.vue'
import EmptyState from '../../../shared/components/ui/EmptyState.vue'
import { useCampaignStore } from '../../../stores/campaigns'

const router = useRouter()
const campaignStore = useCampaignStore()

onMounted(async () => {
  await campaignStore.fetchCampaigns()
})

const selectCampaign = (id: number) => {
  router.push(`/campaigns/${id}`)
}

const formatDate = (dateString: string) => {
  const date = new Date(dateString)
  return date.toLocaleDateString()
}
</script>

<style scoped>
.campaign-list-view {
  @apply space-y-6;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.page-title {
  font-size: 2rem;
  font-weight: 700;
  color: var(--color-text);
}

.loading {
  text-align: center;
  padding: var(--spacing-xl) 0;
  color: var(--color-text-secondary);
}

.campaign-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: var(--spacing-lg);
}

/* Campaign card content styles - base styling from .card-interactive */

.campaign-name {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: var(--spacing-sm);
}

.campaign-meta {
  display: flex;
  gap: var(--spacing-md);
  align-items: center;
  margin-bottom: var(--spacing-sm);
}

.campaign-status {
  font-size: 0.875rem;
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-sm);
  font-weight: 500;
}

.status-planning {
  background-color: var(--color-surface-variant);
  color: var(--color-status-planning);
  border: 1px solid var(--color-border);
}

.status-active {
  background-color: var(--color-surface-variant);
  color: var(--color-status-active);
  border: 1px solid var(--color-status-active);
}

.status-completed {
  background-color: var(--color-surface-variant);
  color: var(--color-status-completed);
  border: 1px solid var(--color-border);
}

.campaign-date {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.campaign-path {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.error-message {
  padding: var(--spacing-md);
  background-color: var(--color-error) / 0.1;
  border: 1px solid var(--color-error) / 0.2;
  border-radius: var(--radius-md);
  color: var(--color-error);
}

/* Theme-specific overrides no longer needed - using theme variables */
</style>