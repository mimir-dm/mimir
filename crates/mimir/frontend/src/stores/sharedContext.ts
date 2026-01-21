import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { emit, listen } from '@tauri-apps/api/event'

export interface WindowState {
  id: string
  type: 'main' | 'reference' | 'chat' | 'debug'
  title: string
  focused: boolean
  route?: string
}

export interface UserAction {
  timestamp: number
  type: string
  description: string
  data?: any
}

export interface CampaignContext {
  id?: string
  name?: string
  currentStage?: string
  currentDocument?: string
  directory_path?: string
  modules?: Array<{
    id: string
    name: string
    status: string
  }>
}

export interface ModuleContext {
  id?: string
  name?: string
  campaignId?: string
  currentStage?: string
  sessions?: Array<{
    id: string
    name: string
    status: string
  }>
}

export interface SessionContext {
  id?: string
  name?: string
  moduleId?: string
  status?: string
  notes?: string
}

export interface ReferenceContext {
  activeTab?: 'reading' | 'catalog'
  // Reading mode context
  reading?: {
    currentBook?: string
    currentSection?: string
  }
  // Catalog mode context
  catalog?: {
    selectedCategory?: string
    selectedItems?: string[]
    searchQuery?: string
    selectedSources?: string[]
  }
}

export interface SharedContext {
  campaign?: CampaignContext
  module?: ModuleContext
  session?: SessionContext
  reference?: ReferenceContext
  windows: Map<string, WindowState>
  recentActions: UserAction[]
  contextUsage?: number
}

export const useSharedContextStore = defineStore('sharedContext', () => {
  // State
  const campaign = ref<CampaignContext>({})
  const module = ref<ModuleContext>({})
  const session = ref<SessionContext>({})
  const reference = ref<ReferenceContext>({})
  const windows = ref<Map<string, WindowState>>(new Map())
  const recentActions = ref<UserAction[]>([])
  const contextUsage = ref<number>(0)
  
  // Keep only last 10 actions
  const MAX_ACTIONS = 10
  
  // Computed
  const fullContext = computed<SharedContext>(() => ({
    campaign: campaign.value,
    module: module.value,
    session: session.value,
    reference: reference.value,
    windows: windows.value,
    recentActions: recentActions.value,
    contextUsage: contextUsage.value
  }))
  
  const hasActiveContext = computed(() => 
    Boolean(campaign.value?.id || module.value?.id || session.value?.id)
  )
  
  // Actions
  const updateCampaign = async (data: Partial<CampaignContext>) => {
    campaign.value = { ...campaign.value, ...data }
    await syncToBackend('campaign', campaign.value)
    addAction('campaign_update', `Updated campaign: ${data.name || data.id}`)
  }
  
  const updateModule = async (data: Partial<ModuleContext>) => {
    module.value = { ...module.value, ...data }
    await syncToBackend('module', module.value)
    addAction('module_update', `Updated module: ${data.name || data.id}`)
  }
  
  const updateSession = async (data: Partial<SessionContext>) => {
    session.value = { ...session.value, ...data }
    await syncToBackend('session', session.value)
    addAction('session_update', `Updated session: ${data.name || data.id}`)
  }
  
  const updateReference = async (data: Partial<ReferenceContext>) => {
    reference.value = { ...reference.value, ...data }
    await syncToBackend('reference', reference.value)
    // Track catalog search actions
    if (data.catalog?.searchQuery) {
      addAction('reference_search', `Searched: ${data.catalog.searchQuery}`)
    }
    // Track reading actions
    if (data.reading?.currentBook) {
      addAction('reference_reading', `Reading: ${data.reading.currentBook}`)
    }
  }
  
  const registerWindow = (window: WindowState) => {
    windows.value.set(window.id, window)
    syncToBackend('windows', Array.from(windows.value.values()))
  }
  
  const unregisterWindow = (windowId: string) => {
    windows.value.delete(windowId)
    syncToBackend('windows', Array.from(windows.value.values()))
  }
  
  const updateWindowState = (windowId: string, state: Partial<WindowState>) => {
    const window = windows.value.get(windowId)
    if (window) {
      windows.value.set(windowId, { ...window, ...state })
      syncToBackend('windows', Array.from(windows.value.values()))
    }
  }
  
  const addAction = (type: string, description: string, data?: any) => {
    const action: UserAction = {
      timestamp: Date.now(),
      type,
      description,
      data
    }
    
    recentActions.value = [
      action,
      ...recentActions.value.slice(0, MAX_ACTIONS - 1)
    ]
    
    syncToBackend('actions', recentActions.value)
  }
  
  const clearContext = async () => {
    campaign.value = {}
    module.value = {}
    session.value = {}
    reference.value = {}
    recentActions.value = []
    contextUsage.value = 0
    await invoke('clear_shared_context')
    addAction('context_cleared', 'Cleared all context')
  }
  
  // Backend sync
  const syncToBackend = async (contextType: string, data: any) => {
    try {
      await invoke('update_context', {
        windowId: getCurrentWindowId(),
        contextType,
        data: JSON.stringify(data)
      })
      
      // Emit event for other windows
      await emit('context-updated', {
        contextType,
        data
      })
    } catch (error) {
      console.error('Failed to sync context to backend:', error)
    }
  }
  
  const loadFullContext = async () => {
    try {
      const context = await invoke<string>('get_full_context')
      const parsed = JSON.parse(context) as SharedContext
      
      if (parsed.campaign) campaign.value = parsed.campaign
      if (parsed.module) module.value = parsed.module
      if (parsed.session) session.value = parsed.session
      if (parsed.reference) reference.value = parsed.reference
      if (parsed.recentActions) recentActions.value = parsed.recentActions
      if (parsed.contextUsage !== undefined) contextUsage.value = parsed.contextUsage
      
      // Reconstruct windows map
      if (parsed.windows) {
        windows.value = new Map(
          Object.entries(parsed.windows).map(([id, state]) => [id, state as WindowState])
        )
      }
    } catch (error) {
      console.error('Failed to load context from backend:', error)
    }
  }
  
  const updateContextUsage = (usage: number) => {
    contextUsage.value = usage
  }
  
  // Helper to get current window ID (will be implemented per window)
  const getCurrentWindowId = () => {
    // This will be set by each window on mount
    return (window as any).__TAURI_WINDOW_ID__ || 'main'
  }
  
  // Listen for context updates from other windows
  const startListening = async () => {
    await listen('context-updated', (event) => {
      const { contextType, data } = event.payload as any
      
      // Update local state without re-syncing to avoid loops
      switch (contextType) {
        case 'campaign':
          campaign.value = data
          break
        case 'module':
          module.value = data
          break
        case 'session':
          session.value = data
          break
        case 'reference':
          reference.value = data
          break
        case 'actions':
          recentActions.value = data
          break
      }
    })
  }
  
  // Initialize on store creation
  startListening()
  loadFullContext()
  
  return {
    // State
    campaign,
    module,
    session,
    reference,
    windows,
    recentActions,
    contextUsage,
    
    // Computed
    fullContext,
    hasActiveContext,
    
    // Actions
    updateCampaign,
    updateModule,
    updateSession,
    updateReference,
    registerWindow,
    unregisterWindow,
    updateWindowState,
    addAction,
    clearContext,
    loadFullContext,
    updateContextUsage
  }
})