/**
 * Generic Homebrew Service Factory
 *
 * Creates type-safe CRUD services for any homebrew entity type,
 * eliminating duplication across items, monsters, and spells.
 */

import { invoke } from '@tauri-apps/api/core'
import type { ApiResponse } from '@/types/api'
import { dataEvents, type DataEventName } from '@/utils/dataEvents'

/** Common fields shared by all homebrew entities. */
export interface HomebrewBase {
  id: string
  campaign_id: string
  name: string
  data: string
  cloned_from_name: string | null
  cloned_from_source: string | null
  created_at: string
  updated_at: string
}

interface HomebrewServiceConfig {
  /** Tauri command suffix, e.g. "item", "monster", "spell" */
  commandSuffix: string
  /** Event prefix for dataEvents, e.g. "homebrew", "homebrew-monster" */
  eventPrefix: string
  /** Human-readable label for error messages */
  label: string
  /** Whether this entity has a getByName command */
  hasGetByName?: boolean
}

export interface HomebrewService<
  T extends HomebrewBase,
  TCreate,
  TUpdate
> {
  list(campaignId: string): Promise<T[]>
  get(id: string): Promise<T>
  getByName?(campaignId: string, name: string): Promise<T | null>
  create(request: TCreate): Promise<T>
  update(id: string, request: TUpdate): Promise<T>
  delete(id: string): Promise<void>
}

export function createHomebrewService<
  T extends HomebrewBase,
  TCreate,
  TUpdate
>(config: HomebrewServiceConfig): HomebrewService<T, TCreate, TUpdate> {
  const { commandSuffix: s, eventPrefix: ep, label } = config

  const service: HomebrewService<T, TCreate, TUpdate> = {
    async list(campaignId: string): Promise<T[]> {
      const response = await invoke<ApiResponse<T[]>>(`list_homebrew_${s}s`, { campaignId })
      if (response.success && response.data) return response.data
      throw new Error(response.error || `Failed to list homebrew ${label}s`)
    },

    async get(id: string): Promise<T> {
      const response = await invoke<ApiResponse<T>>(`get_homebrew_${s}`, { id })
      if (response.success && response.data) return response.data
      throw new Error(response.error || `Failed to get homebrew ${label} ${id}`)
    },

    async create(request: TCreate): Promise<T> {
      const response = await invoke<ApiResponse<T>>(`create_homebrew_${s}`, { input: request })
      if (response.success && response.data) {
        dataEvents.emit(`${ep}:created` as DataEventName, response.data as any)
        return response.data
      }
      throw new Error(response.error || `Failed to create homebrew ${label}`)
    },

    async update(id: string, request: TUpdate): Promise<T> {
      const response = await invoke<ApiResponse<T>>(`update_homebrew_${s}`, { id, input: request })
      if (response.success && response.data) {
        dataEvents.emit(`${ep}:updated` as DataEventName, response.data as any)
        return response.data
      }
      throw new Error(response.error || `Failed to update homebrew ${label} ${id}`)
    },

    async delete(id: string): Promise<void> {
      const response = await invoke<ApiResponse<boolean>>(`delete_homebrew_${s}`, { id })
      if (response.success) {
        dataEvents.emit(`${ep}:deleted` as DataEventName, { id } as any)
        return
      }
      throw new Error(response.error || `Failed to delete homebrew ${label} ${id}`)
    }
  }

  if (config.hasGetByName) {
    service.getByName = async (campaignId: string, name: string): Promise<T | null> => {
      const response = await invoke<ApiResponse<T | null>>(`get_homebrew_${s}_by_name`, {
        campaignId,
        name
      })
      if (response.success) return response.data ?? null
      throw new Error(response.error || `Failed to get homebrew ${label} ${name}`)
    }
  }

  return service
}
