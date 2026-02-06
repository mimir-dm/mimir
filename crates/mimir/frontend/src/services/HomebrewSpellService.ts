/**
 * Homebrew Spell Service
 *
 * CRUD operations for campaign homebrew spells via Tauri commands.
 */

import { createHomebrewService } from './createHomebrewService'
import type { CampaignHomebrewSpell } from '@/types/generated/CampaignHomebrewSpell'

// Re-export the generated type as HomebrewSpell for backwards compatibility
export type HomebrewSpell = CampaignHomebrewSpell

export interface CreateHomebrewSpellRequest {
  campaign_id: string
  name: string
  level?: number
  school?: string
  data: string
  cloned_from_name?: string
  cloned_from_source?: string
}

export interface UpdateHomebrewSpellRequest {
  name?: string
  level?: number | null
  school?: string | null
  data?: string
}

export const HomebrewSpellService = createHomebrewService<
  HomebrewSpell,
  CreateHomebrewSpellRequest,
  UpdateHomebrewSpellRequest
>({
  commandSuffix: 'spell',
  eventPrefix: 'homebrew-spell',
  label: 'spell',
})
