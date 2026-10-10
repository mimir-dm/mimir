<template>
  <MainLayout>
    <div class="home-view">
      <!-- Ambient background effect -->
      <div class="ambient-bg"></div>

      <section class="hero">
        <div class="hero-image-container">
          <img
            :src="skullImage"
            alt="Mimir - Mystical Skull"
            class="hero-image"
          />
        </div>
        <h1 class="hero-title">
          <span class="title-main">Mimir</span>
          <span class="title-divider">·</span>
          <span class="title-sub">Campaign Assistant</span>
        </h1>
        <p class="hero-tagline">Your arcane companion for D&D 5e campaign management</p>

        <nav class="hero-actions" aria-label="Get started">
          <router-link
            v-if="lastCampaign"
            :to="`/campaigns/${lastCampaign.id}/dashboard`"
            class="btn btn-primary btn-lg hero-action"
            data-testid="home-continue"
            @click="rememberCampaign(lastCampaign.id)"
          >
            <Play class="icon-em" aria-hidden="true" />
            <span class="hero-action-label" :title="lastCampaign.name">Continue {{ lastCampaign.name }}</span>
          </router-link>
          <router-link
            to="/campaigns/new"
            :class="['btn', 'btn-lg', 'hero-action', lastCampaign ? 'btn-secondary' : 'btn-primary']"
            data-testid="home-create"
          >
            <Plus class="icon-em" aria-hidden="true" />
            Create campaign
          </router-link>
          <button type="button" class="btn btn-secondary btn-lg hero-action" data-testid="home-import" @click="showImportDialog = true">
            <Download class="icon-em" aria-hidden="true" />
            Import campaign
          </button>
        </nav>
      </section>

      <CampaignArchiveImportDialog
        :visible="showImportDialog"
        @close="showImportDialog = false"
        @imported="openImported"
      />
    </div>
  </MainLayout>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { Download, Play, Plus } from '@lucide/vue'
import CampaignArchiveImportDialog from '@/components/campaigns/CampaignArchiveImportDialog.vue'
import { useCampaignStore } from '@/stores/campaigns'
import type { Campaign } from '@/types/api'
import MainLayout from '../shared/components/layout/MainLayout.vue'
import { useThemeStore } from '../stores/theme'
import lightMimir from '../assets/images/themes/light/mimir.png'
import darkMimir from '../assets/images/themes/dark/mimir.png'
import hyperMimir from '../assets/images/themes/hyper/mimir.png'

const themeStore = useThemeStore()
const router = useRouter()
const campaignStore = useCampaignStore()
const { campaigns, currentCampaign } = storeToRefs(campaignStore)
const showImportDialog = ref(false)

// The campaign selected last, if it still exists and is not archived. The
// header's CampaignSelector sets the store's current campaign (reactive) and
// stores the id; on a first start it may do so after Home has loaded.
const lastCampaign = computed<Campaign | null>(() => {
  const id = currentCampaign.value?.id ?? localStorage.getItem('selectedCampaignId')
  return campaigns.value.find((c) => c.id === id && !c.archived_at) ?? null
})

function rememberCampaign(id: string) {
  localStorage.setItem('selectedCampaignId', id)
}

async function openImported(campaign: Campaign) {
  showImportDialog.value = false
  rememberCampaign(campaign.id)
  await campaignStore.fetchCampaigns()
  router.push(`/campaigns/${campaign.id}/dashboard`)
}

onMounted(() => {
  campaignStore.fetchCampaigns()
})

// Dynamically select skull image based on current theme
const skullImage = computed(() => {
  switch (themeStore.currentTheme) {
    case 'dark':
      return darkMimir
    case 'hyper':
      return hyperMimir
    default:
      return lightMimir
  }
})
</script>

<style scoped>
.home-view {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  position: relative;
  overflow: hidden;
}

/* Ambient background with subtle radial gradient */
.ambient-bg {
  position: absolute;
  inset: 0;
  background: radial-gradient(
    ellipse 80% 60% at 50% 40%,
    var(--color-primary-500) 0%,
    transparent 70%
  );
  opacity: 0.04;
  pointer-events: none;
  animation: ambient-pulse 8s ease-in-out infinite;
}

@keyframes ambient-pulse {
  0%, 100% {
    opacity: 0.04;
    transform: scale(1);
  }
  50% {
    opacity: 0.07;
    transform: scale(1.05);
  }
}

.hero {
  text-align: center;
  overflow: visible;
  position: relative;
  z-index: 1;
}

.hero-image-container {
  display: flex;
  justify-content: center;
  align-items: center;
  animation: float 6s ease-in-out infinite;
  overflow: visible;
}

.hero-image {
  width: 400px;
  height: 400px;
  object-fit: contain;
  display: block;
  padding: 60px;
  margin: -60px;
  box-sizing: content-box;
  animation: fade-in 1s ease-out;
}

@keyframes float {
  0%, 100% {
    transform: translateY(0px);
  }
  50% {
    transform: translateY(-20px);
  }
}

@keyframes fade-in {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.hero-title {
  font-family: var(--font-display);
  font-size: 2.5rem;
  font-weight: 600;
  color: var(--color-text);
  margin-top: var(--spacing-xl);
  letter-spacing: 0.08em;
  text-align: center;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  animation: title-fade-in 1s ease-out 0.3s both;
}

@keyframes title-fade-in {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.title-main {
  font-weight: 700;
}

.title-divider {
  opacity: 0.4;
  font-weight: 300;
}

.title-sub {
  font-weight: 400;
  opacity: 0.85;
}

.hero-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: var(--spacing-md);
  margin-top: var(--spacing-xl);
  animation: title-fade-in 1s ease-out 0.7s both;
}

.hero-action {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-sm);
  text-decoration: none;
  max-width: 28rem;
}

.hero-action-label {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.hero-tagline {
  font-size: 1rem;
  color: var(--color-text-secondary);
  margin-top: var(--spacing-md);
  font-weight: 400;
  letter-spacing: 0.02em;
  animation: title-fade-in 1s ease-out 0.5s both;
}
</style>