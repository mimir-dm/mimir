/**
 * Homebrew Item Service
 *
 * CRUD operations for campaign homebrew items via Tauri commands.
 */

import { createHomebrewService } from './createHomebrewService'
import type { CampaignHomebrewItem } from '@/types/generated/CampaignHomebrewItem'

// Re-export the generated type as HomebrewItem for backwards compatibility
export type HomebrewItem = CampaignHomebrewItem

export interface CreateHomebrewItemRequest {
  campaign_id: string
  name: string
  item_type?: string
  rarity?: string
  data: string
  cloned_from_name?: string
  cloned_from_source?: string
}

export interface UpdateHomebrewItemRequest {
  name?: string
  item_type?: string | null
  rarity?: string | null
  data?: string
}

export const HomebrewService = createHomebrewService<
  HomebrewItem,
  CreateHomebrewItemRequest,
  UpdateHomebrewItemRequest
>({
  commandSuffix: 'item',
  eventPrefix: 'homebrew-item',
  label: 'item',
  hasGetByName: true,
})
