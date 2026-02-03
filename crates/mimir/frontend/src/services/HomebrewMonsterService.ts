/**
 * Homebrew Monster Service
 *
 * CRUD operations for campaign homebrew monsters via Tauri commands.
 */

import { createHomebrewService, type HomebrewBase } from './createHomebrewService'

export interface HomebrewMonster extends HomebrewBase {
  cr: string | null
  creature_type: string | null
  size: string | null
}

export interface CreateHomebrewMonsterRequest {
  campaign_id: string
  name: string
  cr?: string
  creature_type?: string
  size?: string
  data: string
  cloned_from_name?: string
  cloned_from_source?: string
}

export interface UpdateHomebrewMonsterRequest {
  name?: string
  cr?: string | null
  creature_type?: string | null
  size?: string | null
  data?: string
}

export const HomebrewMonsterService = createHomebrewService<
  HomebrewMonster,
  CreateHomebrewMonsterRequest,
  UpdateHomebrewMonsterRequest
>({
  commandSuffix: 'monster',
  eventPrefix: 'homebrew-monster',
  label: 'monster',
})
