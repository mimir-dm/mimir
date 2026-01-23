import { ref, type Ref } from 'vue'
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
  reward_type?: string
}

export interface RewardFilters {
  query?: string
  sources?: string[]
  reward_type?: string
}

export function useRewards() {
  const isRewardsInitialized = ref(true)
  const isLoading = ref(false)
  const error: Ref<string | null> = ref(null)
  const rewards = ref<RewardSummary[]>([])

  async function initializeRewardCatalog() {
    // No initialization needed for DB-backed catalog
  }

  async function searchRewards(filters: RewardFilters = {}): Promise<RewardSummary[]> {
    try {
      isLoading.value = true
      error.value = null

      // Transform to backend RewardFilter format
      const backendFilter = {
        name_contains: filters.query || null,
        sources: filters.sources ?? null,
        reward_type: filters.reward_type || null,
      }

      const response = await invoke<{ success: boolean; data?: RewardSummary[]; error?: string }>('search_rewards', {
        filter: backendFilter,
        limit: 10000,
        offset: 0
      })

      if (response.success && response.data) {
        rewards.value = response.data
        return response.data
      } else {
        error.value = response.error || 'Search failed'
        return []
      }
    } catch (e) {
      error.value = `Search failed: ${e}`
      return []
    } finally {
      isLoading.value = false
    }
  }

  async function getRewardDetails(name: string, source: string): Promise<Reward | null> {
    try {
      const response = await invoke<{ success: boolean; data?: Reward; error?: string }>('get_reward_by_name', {
        name,
        source
      })
      if (response.success && response.data) {
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to get reward details:', e)
      return null
    }
  }

  return {
    isRewardsInitialized,
    isLoading,
    error,
    rewards,
    initializeRewardCatalog,
    searchRewards,
    getRewardDetails,
  }
}
