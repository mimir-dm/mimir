/**
 * Homebrew Spell Service
 *
 * Provides access to campaign homebrew spell CRUD operations via Tauri commands.
 */

import { invoke } from '@tauri-apps/api/core'
import type { ApiResponse } from '@/types/api'
import { dataEvents } from '@/utils/dataEvents'

// =============================================================================
// Types
// =============================================================================

export interface HomebrewSpell {
  id: string
  campaign_id: string
  name: string
  level: number | null
  school: string | null
  data: string
  cloned_from_name: string | null
  cloned_from_source: string | null
  created_at: string
  updated_at: string
}

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

// =============================================================================
// Service
// =============================================================================

class HomebrewSpellServiceClass {
  async list(campaignId: string): Promise<HomebrewSpell[]> {
    const response = await invoke<ApiResponse<HomebrewSpell[]>>('list_homebrew_spells', {
      campaignId
    })

    if (response.success && response.data) {
      return response.data
    }

    throw new Error(response.error || 'Failed to list homebrew spells')
  }

  async get(id: string): Promise<HomebrewSpell> {
    const response = await invoke<ApiResponse<HomebrewSpell>>('get_homebrew_spell', { id })

    if (response.success && response.data) {
      return response.data
    }

    throw new Error(response.error || `Failed to get homebrew spell ${id}`)
  }

  async create(request: CreateHomebrewSpellRequest): Promise<HomebrewSpell> {
    const response = await invoke<ApiResponse<HomebrewSpell>>('create_homebrew_spell', {
      input: request
    })

    if (response.success && response.data) {
      dataEvents.emit('homebrew-spell:created', response.data)
      return response.data
    }

    throw new Error(response.error || 'Failed to create homebrew spell')
  }

  async update(id: string, request: UpdateHomebrewSpellRequest): Promise<HomebrewSpell> {
    const response = await invoke<ApiResponse<HomebrewSpell>>('update_homebrew_spell', {
      id,
      input: request
    })

    if (response.success && response.data) {
      dataEvents.emit('homebrew-spell:updated', response.data)
      return response.data
    }

    throw new Error(response.error || `Failed to update homebrew spell ${id}`)
  }

  async delete(id: string): Promise<void> {
    const response = await invoke<ApiResponse<boolean>>('delete_homebrew_spell', { id })

    if (response.success) {
      dataEvents.emit('homebrew-spell:deleted', { id })
      return
    }

    throw new Error(response.error || `Failed to delete homebrew spell ${id}`)
  }
}

export const HomebrewSpellService = new HomebrewSpellServiceClass()
