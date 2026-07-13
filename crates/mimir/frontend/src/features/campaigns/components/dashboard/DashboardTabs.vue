<template>
  <nav class="dashboard-tabs" v-if="!hideTabBar">
    <div class="tabs-container">
      <button
        v-for="tab in dashboardTabs"
        :key="tab.id"
        :class="['tab-button', { active: activeTab === tab.id }]"
        @click="onTabClick(tab.id)"
      >
        <svg
          class="tab-icon"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <template v-if="tab.icon === 'globe'">
            <circle cx="12" cy="12" r="10" />
            <line x1="2" y1="12" x2="22" y2="12" />
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
          </template>
          <template v-else-if="tab.icon === 'folder'">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </template>
          <template v-else-if="tab.icon === 'users'">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
            <circle cx="9" cy="7" r="4" />
            <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
            <path d="M16 3.13a4 4 0 0 1 0 7.75" />
          </template>
          <template v-else-if="tab.icon === 'user'">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
            <circle cx="12" cy="7" r="4" />
          </template>
          <template v-else-if="tab.icon === 'flask'">
            <path d="M10 2v6L4.5 18.5A2 2 0 0 0 6.24 21.5h11.52a2 2 0 0 0 1.74-3L14 8V2" />
            <line x1="8.5" y1="2" x2="15.5" y2="2" />
            <line x1="7" y1="14" x2="17" y2="14" />
          </template>
        </svg>
        <span class="tab-label">{{ tab.label }}</span>
      </button>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useDashboardState, dashboardTabs, type DashboardTab } from '../../composables/useDashboardState'

const props = defineProps<{
  campaignId: string | number
}>()

const route = useRoute()
const { activeTab, setTab } = useDashboardState(props.campaignId)

// Hide tab bar on full-screen routes
const hideTabBar = computed(() => {
  return route.meta.hideTabBar === true
})

function onTabClick(tabId: DashboardTab) {
  setTab(tabId)
}
</script>

<style scoped>
.dashboard-tabs {
  display: flex;
  background: var(--color-surface, #1a1a1a);
  border-bottom: 1px solid var(--color-border, #333);
  padding: 0 var(--spacing-lg, 16px);
}

.tabs-container {
  display: flex;
  gap: var(--spacing-xs, 4px);
}

.tab-button {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm, 8px);
  padding: var(--spacing-md, 12px) var(--spacing-lg, 16px);
  background: transparent;
  border: none;
  border-bottom: 3px solid transparent;
  color: var(--color-text-muted, #888);
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  margin-bottom: -1px;
}

.tab-button:hover {
  color: var(--color-text, #e0e0e0);
  background: var(--color-base-200, #242424);
}

.tab-button.active {
  color: var(--color-primary, #4a9eff);
  border-bottom-color: var(--color-primary, #4a9eff);
}

.tab-icon {
  flex-shrink: 0;
  opacity: 0.8;
}

.tab-label {
  font-weight: 500;
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .tabs-container {
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
    width: 100%;
  }

  .tab-button {
    flex-shrink: 0;
    padding: var(--spacing-sm, 8px) var(--spacing-md, 12px);
    font-size: 0.85rem;
  }

  .tab-label {
    display: none;
  }

  .tab-icon {
    width: 20px;
    height: 20px;
  }
}
</style>
