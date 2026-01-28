/**
 * Module Service
 *
 * Provides access to module CRUD operations via Tauri commands.
 * Types match mimir-core Module model.
 */

import { invoke } from '@tauri-apps/api/core'
import type { ApiResponse, Module } from '@/types/api'
import { dataEvents } from '@/utils/dataEvents'

// =============================================================================
// Request Types
// =============================================================================

export interface CreateModuleRequest {
  campaign_id: string
  name: string
  description?: string
  module_type?: string
}

export interface UpdateModuleRequest {
  name?: string
  description?: string | null
}

// =============================================================================
// Module Service
// =============================================================================

class ModuleServiceClass {
  /**
   * List all modules for a campaign
   */
  async list(campaignId: string): Promise<Module[]> {
    const response = await invoke<ApiResponse<Module[]>>('list_modules', {
      campaignId
    })

    if (response.success && response.data) {
      return response.data
    }

    throw new Error(response.error || 'Failed to list modules')
  }

  /**
   * Get a module by ID
   */
  async get(id: string): Promise<Module> {
    const response = await invoke<ApiResponse<Module>>('get_module', { id })

    if (response.success && response.data) {
      return response.data
    }

    throw new Error(response.error || `Failed to get module ${id}`)
  }

  /**
   * Get a module by campaign ID and module number
   */
  async getByNumber(campaignId: string, moduleNumber: number): Promise<Module> {
    const response = await invoke<ApiResponse<Module>>('get_module_by_number', {
      campaignId,
      moduleNumber
    })

    if (response.success && response.data) {
      return response.data
    }

    throw new Error(response.error || `Failed to get module #${moduleNumber}`)
  }

  /**
   * Create a new module
   */
  async create(request: CreateModuleRequest): Promise<Module> {
    const response = await invoke<ApiResponse<Module>>('create_module', { request })

    if (response.success && response.data) {
      dataEvents.emit('module:created', {
        campaignId: request.campaign_id,
        moduleId: response.data.id
      })
      return response.data
    }

    throw new Error(response.error || 'Failed to create module')
  }

  /**
   * Update a module
   */
  async update(id: string, request: UpdateModuleRequest): Promise<Module> {
    const response = await invoke<ApiResponse<Module>>('update_module', {
      id,
      request
    })

    if (response.success && response.data) {
      dataEvents.emit('module:updated', { moduleId: id })
      return response.data
    }

    throw new Error(response.error || `Failed to update module ${id}`)
  }

  /**
   * Delete a module
   */
  async delete(id: string, campaignId?: string): Promise<void> {
    const response = await invoke<ApiResponse<void>>('delete_module', { id })

    if (response.success) {
      if (campaignId) {
        dataEvents.emit('module:deleted', { campaignId, moduleId: id })
      }
      return
    }

    throw new Error(response.error || `Failed to delete module ${id}`)
  }

  /**
   * Update module status/stage (stub - backend doesn't have stages)
   * @deprecated The new backend doesn't support module stages
   */
  async updateStatus(id: string, status: string): Promise<Module> {
    console.warn('ModuleService.updateStatus: Module stages are not supported in the new backend')
    // Return the current module without changes
    return this.get(id)
  }

  /**
   * Transition module to a new stage (stub - backend doesn't have stages)
   * @deprecated The new backend doesn't support module stages
   */
  async transitionStage(id: string, newStage: string): Promise<Module> {
    console.warn('ModuleService.transitionStage: Module stages are not supported in the new backend')
    // Return the current module without changes
    return this.get(id)
  }

  /**
   * Initialize documents for a module (stub - backend doesn't have template system)
   * @deprecated The new backend doesn't support document templates
   */
  async initializeDocuments(id: string): Promise<string[]> {
    console.warn('ModuleService.initializeDocuments: Document templates are not supported in the new backend')
    return []
  }
}

export const ModuleService = new ModuleServiceClass()
