import { invoke } from '@tauri-apps/api/core'

export interface FeatSummary {
  name: string
  source: string
  page?: number
  prerequisites?: string
  brief?: string
}

export interface FeatFilters {
  query?: string
  sources?: string[]
  has_prerequisites?: boolean
}

export interface Feat {
  name: string
  source: string
  page?: number
  srd?: boolean
  entries: any[]
  prerequisite?: any[]
  ability?: any[]
  skill_proficiencies?: any[]
  language_proficiencies?: any[]
  tool_proficiencies?: any[]
  weapon_proficiencies?: any[]
  armor_proficiencies?: any[]
  saving_throw_proficiencies?: any[]
  expertise?: any[]
  resist?: any[]
  immune?: any[]
  senses?: any[]
  additional_spells?: any[]
  other_sources?: any[]
}

export function useFeats() {
  async function searchFeats(params: FeatFilters = {}): Promise<FeatSummary[]> {
    try {
      const response = await invoke<{ success: boolean; data?: FeatSummary[]; error?: string }>('search_feats', {
        filter: {
          name: params.query || null,
          sources: params.sources?.length ? params.sources : null,
        },
        limit: 100,
        offset: 0
      })
      if (response.success && response.data) {
        return response.data
      }
      return []
    } catch (e) {
      console.error('Failed to search feats:', e)
      return []
    }
  }

  async function getFeatDetails(name: string, source: string): Promise<Feat | null> {
    try {
      const response = await invoke<{ success: boolean; data?: Feat; error?: string }>('get_feat_by_name', { name, source })
      if (response.success && response.data) {
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to get feat details:', e)
      return null
    }
  }

  return {
    searchFeats,
    getFeatDetails,
  }
}
