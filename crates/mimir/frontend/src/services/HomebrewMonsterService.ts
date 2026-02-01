/**
 * Homebrew Monster Service
 *
 * Provides access to campaign homebrew monster CRUD operations via Tauri commands.
 */

import { invoke } from '@tauri-apps/api/core'
import type { ApiResponse } from '@/types/api'
import { dataEvents } from '@/utils/dataEvents'

// =============================================================================
// Types
// =============================================================================

export interface HomebrewMonster {
  id: string
  campaign_id: string
  name: string
  cr: string | null
  creature_type: string | null
  size: string | null
  data: string
  cloned_from_name: string | null
  cloned_from_source: string | null
  created_at: string
  updated_at: string
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

// =============================================================================
// Service
// =============================================================================

class HomebrewMonsterServiceClass {
  async list(campaignId: string): Promise<HomebrewMonster[]> {
    const response = await invoke<ApiResponse<HomebrewMonster[]>>('list_homebrew_monsters', {
      campaignId
    })

    if (response.success && response.data) {
      return response.data
    }

    throw new Error(response.error || 'Failed to list homebrew monsters')
  }

  async get(id: string): Promise<HomebrewMonster> {
    const response = await invoke<ApiResponse<HomebrewMonster>>('get_homebrew_monster', { id })

    if (response.success && response.data) {
      return response.data
    }

    throw new Error(response.error || `Failed to get homebrew monster ${id}`)
  }

  async create(request: CreateHomebrewMonsterRequest): Promise<HomebrewMonster> {
    const response = await invoke<ApiResponse<HomebrewMonster>>('create_homebrew_monster', {
      input: request
    })

    if (response.success && response.data) {
      dataEvents.emit('homebrew-monster:created', response.data)
      return response.data
    }

    throw new Error(response.error || 'Failed to create homebrew monster')
  }

  async update(id: string, request: UpdateHomebrewMonsterRequest): Promise<HomebrewMonster> {
    const response = await invoke<ApiResponse<HomebrewMonster>>('update_homebrew_monster', {
      id,
      input: request
    })

    if (response.success && response.data) {
      dataEvents.emit('homebrew-monster:updated', response.data)
      return response.data
    }

    throw new Error(response.error || `Failed to update homebrew monster ${id}`)
  }

  async delete(id: string): Promise<void> {
    const response = await invoke<ApiResponse<boolean>>('delete_homebrew_monster', { id })

    if (response.success) {
      dataEvents.emit('homebrew-monster:deleted', { id })
      return
    }

    throw new Error(response.error || `Failed to delete homebrew monster ${id}`)
  }
}

export const HomebrewMonsterService = new HomebrewMonsterServiceClass()
