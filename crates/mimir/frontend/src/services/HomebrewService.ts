/**
 * Homebrew Item Service
 *
 * CRUD operations for campaign homebrew items via Tauri commands.
 */

import { createHomebrewService, type HomebrewBase } from './createHomebrewService'

export interface HomebrewItem extends HomebrewBase {
  item_type: string | null
  rarity: string | null
}

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
  eventPrefix: 'homebrew',
  label: 'item',
  hasGetByName: true,
})
