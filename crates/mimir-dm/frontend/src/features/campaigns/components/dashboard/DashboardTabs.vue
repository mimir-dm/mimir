<template>
  <nav class="dashboard-tabs" v-if="!hideTabBar">
    <div class="tabs-container">
      <button
        v-for="tab in dashboardTabs"
        :key="tab.id"
        :class="['tab-button', { active: activeTab === tab.id }]"
        @click="onTabClick(tab.id)"
      >
        <span class="tab-icon">{{ getIcon(tab.icon) }}</span>
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
const { activeTab, setTab, isPlayMode } = useDashboardState(props.campaignId)

// Hide tab bar in play mode
const hideTabBar = computed(() => {
  return isPlayMode.value || route.meta.hideTabBar === true
})

function onTabClick(tabId: DashboardTab) {
  setTab(tabId)
}

// Simple icon mapping (using emoji for now, can be replaced with icon library)
function getIcon(iconName: string): string {
  const icons: Record<string, string> = {
    home: '~',
    folder: '+',
    user: '@',
    users: '@@',
    globe: '*',
    play: '>'
  }
  return icons[iconName] || ''
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
  font-size: 1rem;
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
    font-size: 1.2rem;
  }
}
</style>
