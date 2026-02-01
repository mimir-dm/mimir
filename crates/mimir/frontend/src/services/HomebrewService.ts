/**
 * Homebrew Item Service
 *
 * Provides access to campaign homebrew item CRUD operations via Tauri commands.
 */

import { invoke } from '@tauri-apps/api/core'
import type { ApiResponse } from '@/types/api'
import { dataEvents } from '@/utils/dataEvents'

// =============================================================================
// Types
// =============================================================================

export interface HomebrewItem {
  id: string
  campaign_id: string
  name: string
  item_type: string | null
  rarity: string | null
  data: string
  cloned_from_name: string | null
  cloned_from_source: string | null
  created_at: string
  updated_at: string
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

// =============================================================================
// Service
// =============================================================================

class HomebrewServiceClass {
  async list(campaignId: string): Promise<HomebrewItem[]> {
    const response = await invoke<ApiResponse<HomebrewItem[]>>('list_homebrew_items', {
      campaignId
    })

    if (response.success && response.data) {
      return response.data
    }

    throw new Error(response.error || 'Failed to list homebrew items')
  }

  async get(id: string): Promise<HomebrewItem> {
    const response = await invoke<ApiResponse<HomebrewItem>>('get_homebrew_item', { id })

    if (response.success && response.data) {
      return response.data
    }

    throw new Error(response.error || `Failed to get homebrew item ${id}`)
  }

  async getByName(campaignId: string, name: string): Promise<HomebrewItem | null> {
    const response = await invoke<ApiResponse<HomebrewItem | null>>('get_homebrew_item_by_name', {
      campaignId,
      name
    })

    if (response.success) {
      return response.data ?? null
    }

    throw new Error(response.error || `Failed to get homebrew item ${name}`)
  }

  async create(request: CreateHomebrewItemRequest): Promise<HomebrewItem> {
    const response = await invoke<ApiResponse<HomebrewItem>>('create_homebrew_item', {
      input: request
    })

    if (response.success && response.data) {
      dataEvents.emit('homebrew:created', response.data)
      return response.data
    }

    throw new Error(response.error || 'Failed to create homebrew item')
  }

  async update(id: string, request: UpdateHomebrewItemRequest): Promise<HomebrewItem> {
    const response = await invoke<ApiResponse<HomebrewItem>>('update_homebrew_item', {
      id,
      input: request
    })

    if (response.success && response.data) {
      dataEvents.emit('homebrew:updated', response.data)
      return response.data
    }

    throw new Error(response.error || `Failed to update homebrew item ${id}`)
  }

  async delete(id: string): Promise<void> {
    const response = await invoke<ApiResponse<boolean>>('delete_homebrew_item', { id })

    if (response.success) {
      dataEvents.emit('homebrew:deleted', { id })
      return
    }

    throw new Error(response.error || `Failed to delete homebrew item ${id}`)
  }
}

export const HomebrewService = new HomebrewServiceClass()
