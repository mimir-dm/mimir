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
      const results = await invoke<FeatSummary[]>('search_feats', {
        query: params.query,
        sources: params.sources,
        has_prerequisites: params.has_prerequisites
      })
      return results || []
    } catch (e) {
      return []
    }
  }

  async function getFeatDetails(name: string, source: string): Promise<Feat | null> {
    try {
      const feat = await invoke<Feat>('get_feat_details', { name, source })
      return feat
    } catch (e) {
      return null
    }
  }

  return {
    searchFeats,
    getFeatDetails,
  }
}
