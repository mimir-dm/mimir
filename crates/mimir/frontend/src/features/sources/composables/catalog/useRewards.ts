import { invoke } from '@tauri-apps/api/core'

export interface Reward {
  name: string
  source: string
  page?: number
  reward_type?: string
  entries?: any[]
  prerequisite?: any[]
  additional_spells?: any[]
  duration?: string
  srd?: boolean
  basic_rules?: boolean
  has_fluff?: boolean
  has_fluff_images?: boolean
}

export interface RewardSummary {
  name: string
  source: string
  reward_type: string
  description: string
  has_prerequisites: boolean
}

export interface RewardFilters {
  query?: string
  sources?: string[]
  reward_types?: string[]
  has_prerequisites?: boolean
}

export function useRewards() {
  async function initializeRewardCatalog() {
    try {
      await invoke('initialize_reward_catalog')
    } catch (e) {
      // Initialization failed silently
    }
  }

  async function searchRewards(filters: RewardFilters = {}): Promise<RewardSummary[]> {
    try {
      const results = await invoke<RewardSummary[]>('search_rewards', {
        query: filters.query || null,
        sources: filters.sources || null,
        reward_types: filters.reward_types || null,
        has_prerequisites: filters.has_prerequisites
      })
      return results || []
    } catch (e) {
      return []
    }
  }

  async function getRewardDetails(name: string, source: string): Promise<Reward | null> {
    try {
      const details = await invoke<Reward>('get_reward_details', { name, source })
      return details
    } catch (e) {
      return null
    }
  }

  async function getRewardTypes(): Promise<string[]> {
    try {
      const types = await invoke<string[]>('get_reward_types')
      return types || []
    } catch (e) {
      return []
    }
  }

  async function getRewardSources(): Promise<string[]> {
    try {
      const sources = await invoke<string[]>('get_reward_sources')
      return sources || []
    } catch (e) {
      return []
    }
  }

  return {
    initializeRewardCatalog,
    searchRewards,
    getRewardDetails,
    getRewardTypes,
    getRewardSources,
  }
}
