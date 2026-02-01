import { ref, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

export type DashboardTab = 'campaign' | 'modules' | 'npcs' | 'pcs' | 'homebrew'

export interface DashboardTabConfig {
  id: DashboardTab
  label: string
  icon: string
  route: string
}

export const dashboardTabs: DashboardTabConfig[] = [
  { id: 'campaign', label: 'Campaign', icon: 'globe', route: 'campaign' },
  { id: 'modules', label: 'Modules', icon: 'folder', route: 'modules' },
  { id: 'npcs', label: 'NPCs', icon: 'users', route: 'npcs' },
  { id: 'pcs', label: 'PCs', icon: 'user', route: 'pcs' },
  { id: 'homebrew', label: 'Homebrew', icon: 'flask', route: 'homebrew' }
]

const STORAGE_KEY_PREFIX = 'mimir-dashboard-tab-'

export function useDashboardState(campaignId: string | number) {
  const route = useRoute()
  const router = useRouter()

  // Get saved tab from localStorage or default to 'campaign'
  const getStoredTab = (): DashboardTab => {
    const stored = localStorage.getItem(`${STORAGE_KEY_PREFIX}${campaignId}`)
    if (stored && dashboardTabs.some(t => t.id === stored)) {
      return stored as DashboardTab
    }
    return 'campaign'
  }

  // Active tab derived from route
  const activeTab = computed((): DashboardTab => {
    const path = route.path
    for (const tab of dashboardTabs) {
      if (path.includes(`/dashboard/${tab.route}`)) {
        return tab.id
      }
    }
    return getStoredTab()
  })

  // Selected module for detail pane
  const selectedModuleId = computed((): string | null => {
    const moduleId = route.params.moduleId
    return moduleId ? String(moduleId) : null
  })

  // Check if module detail pane is open
  const isModuleDetailOpen = computed(() => {
    return selectedModuleId.value !== null && route.path.includes('/modules/')
  })

  // Check if in play mode (full-screen, hides tabs)
  const isPlayMode = computed(() => {
    return route.path.includes('/play')
  })

  // Save tab preference when it changes
  watch(activeTab, (newTab) => {
    localStorage.setItem(`${STORAGE_KEY_PREFIX}${campaignId}`, newTab)
  })

  // Navigate to a specific tab
  function setTab(tabId: DashboardTab) {
    const basePath = `/campaigns/${campaignId}/dashboard`
    router.push(`${basePath}/${tabId}`)
  }

  // Navigate to module detail
  function selectModule(moduleId: string | number) {
    router.push(`/campaigns/${campaignId}/dashboard/modules/${moduleId}`)
  }

  // Close module detail pane
  function closeModuleDetail() {
    router.push(`/campaigns/${campaignId}/dashboard/modules`)
  }

  // Start play session for a module
  function startSession(moduleId: string | number) {
    router.push(`/campaigns/${campaignId}/dashboard/modules/${moduleId}/play`)
  }

  // End play session (return to modules)
  function endSession() {
    router.push(`/campaigns/${campaignId}/dashboard/modules`)
  }

  // Get tab config by id
  function getTabConfig(tabId: DashboardTab): DashboardTabConfig | undefined {
    return dashboardTabs.find(t => t.id === tabId)
  }

  return {
    // State
    activeTab,
    selectedModuleId,
    isModuleDetailOpen,
    isPlayMode,

    // Tab configs
    dashboardTabs,
    getTabConfig,

    // Actions
    setTab,
    selectModule,
    closeModuleDetail,
    startSession,
    endSession
  }
}
