import { ref, computed, type Ref, type ComputedRef } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { DEFAULT_SYSTEM_PROMPT } from '@/constants/defaultSystemPrompt'
import { useSharedContextStore } from '../sharedContext'
import type { ChatMessage } from './types'

export interface ModelInfo {
  model: string
  contextLength: number
  defaultMaxTokens: number
  architecture: string
}

export interface SystemMessageConfig {
  baseInstructions?: string
  contextEnabled?: boolean
  tools?: string[]
  customInstructions?: string
  temperature?: number
  maxTokens?: number
  llmEndpoint?: string
}

interface TokensState {
  modelInfo: Ref<ModelInfo | null>
  totalTokensUsed: Ref<number>
  maxResponseTokens: Ref<number>
  systemConfig: Ref<SystemMessageConfig>
}

interface TokensComputed {
  conversationTokens: ComputedRef<number>
  contextUsagePercentage: ComputedRef<number>
}

interface TokensActions {
  initializeTokens: () => Promise<void>
  setMaxResponseTokens: (tokens: number) => void
  updateSystemConfig: (config: Partial<SystemMessageConfig>) => void
  toggleContext: () => void
  setSystemInstructions: (instructions: string) => void
  setCustomInstructions: (instructions: string) => void
  resetToDefaultPrompt: () => void
  setLlmEndpoint: (endpoint: string) => void
  buildSystemMessage: () => ChatMessage
  saveSystemConfig: () => void
  loadSystemConfig: () => void
}

export function createTokensStore(getMessages: () => ChatMessage[]): TokensState & TokensComputed & TokensActions {
  // State
  const modelInfo = ref<ModelInfo | null>(null)
  const totalTokensUsed = ref(0)
  const maxResponseTokens = ref(16384)
  const systemConfig = ref<SystemMessageConfig>({
    baseInstructions: DEFAULT_SYSTEM_PROMPT,
    contextEnabled: true,
    tools: [],
    customInstructions: '',
    temperature: 0.3,  // Lower temperature for more deterministic tool calling
    maxTokens: 16384,  // Increased to allow room for thinking blocks and tool calls
    llmEndpoint: 'http://localhost:11434'  // Default Ollama endpoint
  })

  // Computed
  const conversationTokens = computed(() => {
    return getMessages().reduce((total, msg) => {
      return total + (msg.tokenUsage?.total || 0)
    }, 0)
  })

  const contextUsagePercentage = computed(() => {
    if (!modelInfo.value) return 0
    return (conversationTokens.value / modelInfo.value.contextLength) * 100
  })

  // Actions
  const initializeTokens = async () => {
    try {
      // Get model info
      const info = await invoke<ModelInfo>('get_model_context_info')
      modelInfo.value = info
      maxResponseTokens.value = info.defaultMaxTokens

      // Load system configuration
      loadSystemConfig()
    } catch (err) {
      console.error('Failed to initialize tokens:', err)
      throw err
    }
  }

  const buildSystemMessage = (): ChatMessage => {
    const contextStore = useSharedContextStore()
    const parts: string[] = []

    // Base instructions
    if (systemConfig.value.baseInstructions) {
      parts.push(systemConfig.value.baseInstructions)
    }

    // Add current context if enabled - send the full raw context as JSON
    if (systemConfig.value.contextEnabled) {
      const fullContext = {
        campaign: contextStore.campaign,
        module: contextStore.module,
        session: contextStore.session,
        reference: contextStore.reference,
        windows: Array.from(contextStore.windows.values()),
        recentActions: contextStore.recentActions.slice(0, 5), // Last 5 actions
        contextUsage: contextStore.contextUsage
      }

      parts.push('Current Application Context:')
      parts.push('```json')
      parts.push(JSON.stringify(fullContext, null, 2))
      parts.push('```')
    }

    // Add tools information if any
    if (systemConfig.value.tools && systemConfig.value.tools.length > 0) {
      parts.push(`Available tools: ${systemConfig.value.tools.join(', ')}`)
    }

    // Add custom instructions
    if (systemConfig.value.customInstructions) {
      parts.push(systemConfig.value.customInstructions)
    }

    return {
      id: 'system',
      role: 'system',
      content: parts.join('\n\n'),
      timestamp: Date.now()
    }
  }

  const setMaxResponseTokens = (tokens: number) => {
    maxResponseTokens.value = Math.min(
      tokens,
      modelInfo.value?.defaultMaxTokens || 2048
    )
    systemConfig.value.maxTokens = maxResponseTokens.value
  }

  const updateSystemConfig = (config: Partial<SystemMessageConfig>) => {
    systemConfig.value = { ...systemConfig.value, ...config }
    saveSystemConfig()
  }

  const toggleContext = () => {
    systemConfig.value.contextEnabled = !systemConfig.value.contextEnabled
    saveSystemConfig()
  }

  const setSystemInstructions = (instructions: string) => {
    systemConfig.value.baseInstructions = instructions
    saveSystemConfig()
  }

  const setCustomInstructions = (instructions: string) => {
    systemConfig.value.customInstructions = instructions
    saveSystemConfig()
  }

  const resetToDefaultPrompt = () => {
    systemConfig.value.baseInstructions = DEFAULT_SYSTEM_PROMPT
    saveSystemConfig()
  }

  const setLlmEndpoint = (endpoint: string) => {
    systemConfig.value.llmEndpoint = endpoint
    saveSystemConfig()
  }

  const saveSystemConfig = () => {
    localStorage.setItem('chat_system_config', JSON.stringify(systemConfig.value))
  }

  const loadSystemConfig = () => {
    const saved = localStorage.getItem('chat_system_config')
    if (saved) {
      try {
        systemConfig.value = JSON.parse(saved)
      } catch (e) {
        console.error('Failed to load system config:', e)
      }
    }
  }

  return {
    // State
    modelInfo,
    totalTokensUsed,
    maxResponseTokens,
    systemConfig,

    // Computed
    conversationTokens,
    contextUsagePercentage,

    // Actions
    initializeTokens,
    setMaxResponseTokens,
    updateSystemConfig,
    toggleContext,
    setSystemInstructions,
    setCustomInstructions,
    resetToDefaultPrompt,
    setLlmEndpoint,
    buildSystemMessage,
    saveSystemConfig,
    loadSystemConfig
  }
}
