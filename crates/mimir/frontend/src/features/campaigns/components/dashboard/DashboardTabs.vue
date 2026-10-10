<template>
  <nav class="dashboard-tabs" v-if="!hideTabBar">
    <div class="tabs-container">
      <button
        v-for="tab in dashboardTabs"
        :key="tab.id"
        :class="['tab-button', { active: activeTab === tab.id }]"
        @click="onTabClick(tab.id)"
      >
        <component :is="TAB_ICONS[tab.icon] ?? Circle" class="tab-icon" :size="16" aria-hidden="true" />
        <span class="tab-label">{{ tab.label }}</span>
      </button>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { computed, type Component } from 'vue'
import { Circle, FlaskConical, Folder, Globe, User, Users } from '@lucide/vue'
import { useRoute } from 'vue-router'
import { useDashboardState, dashboardTabs, type DashboardTab } from '../../composables/useDashboardState'

/** Icon of each tab (the `icon` names in useDashboardState). */
const TAB_ICONS: Record<string, Component> = {
  globe: Globe,
  folder: Folder,
  users: Users,
  user: User,
  flask: FlaskConical,
}

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
  /* Sits in the dashboard header row, which draws the surface and border. */
  display: flex;
}

.tabs-container {
  display: flex;
  gap: var(--spacing-xs, 4px);
}

.tab-button {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm, 8px);
  padding: var(--spacing-md, 12px) var(--spacing-md, 12px);
  background: transparent;
  border: none;
  border-bottom: 3px solid transparent;
  color: var(--color-text-secondary);
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  margin-bottom: -1px;
}

.tab-button:hover {
  color: var(--color-text);
  background: var(--color-surface);
}

.tab-button.active {
  color: var(--color-primary-text);
  border-bottom-color: var(--color-primary-500);
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
