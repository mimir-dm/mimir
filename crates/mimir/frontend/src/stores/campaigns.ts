import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ApiResponse, Campaign, CreateCampaignRequest, UpdateCampaignRequest } from '../types/api'
import { dataEvents } from '@/shared/utils/dataEvents'

export const useCampaignStore = defineStore('campaigns', () => {
  const campaigns = ref<Campaign[]>([])
  const archivedCampaigns = ref<Campaign[]>([])
  const currentCampaign = ref<Campaign | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Fetch all active campaigns
  const fetchCampaigns = async () => {
    loading.value = true
    error.value = null

    try {
      console.log('Fetching campaigns...')
      const response = await invoke<ApiResponse<Campaign[]>>('list_campaigns')
      console.log('Campaign response:', response)

      if (response.success && response.data) {
        campaigns.value = response.data
        console.log('Campaigns loaded successfully:', response.data.length, 'campaigns')
      } else {
        console.error('Campaign fetch failed:', response.error)
        error.value = response.error || 'Failed to fetch campaigns'
      }
    } catch (e) {
      console.error('Campaign fetch exception:', e)
      error.value = e instanceof Error ? e.message : 'Unknown error occurred'
    } finally {
      loading.value = false
    }
  }

  // Get campaign by ID
  const getCampaign = async (id: string) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<Campaign>>('get_campaign', { id })
      if (response.success && response.data) {
        currentCampaign.value = response.data
        return response.data
      } else {
        error.value = response.error || 'Failed to fetch campaign'
        return null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Unknown error occurred'
      return null
    } finally {
      loading.value = false
    }
  }

  // Create new campaign
  const createCampaign = async (request: CreateCampaignRequest) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<Campaign>>('create_campaign', { request })
      if (response.success && response.data) {
        campaigns.value.push(response.data)
        dataEvents.emit('campaign:created', { campaignId: response.data.id })
        return response.data
      } else {
        error.value = response.error || 'Failed to create campaign'
        return null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Unknown error occurred'
      return null
    } finally {
      loading.value = false
    }
  }

  // Update campaign
  const updateCampaign = async (id: string, request: UpdateCampaignRequest) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<Campaign>>('update_campaign', { id, request })
      if (response.success && response.data) {
        // Update local state
        const index = campaigns.value.findIndex(c => c.id === id)
        if (index !== -1) {
          campaigns.value[index] = response.data
        }
        if (currentCampaign.value?.id === id) {
          currentCampaign.value = response.data
        }
        dataEvents.emit('campaign:updated', { campaignId: id })
        return response.data
      } else {
        error.value = response.error || 'Failed to update campaign'
        return null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Unknown error occurred'
      return null
    } finally {
      loading.value = false
    }
  }

  // Fetch archived campaigns
  const fetchArchivedCampaigns = async () => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<Campaign[]>>('list_archived_campaigns')
      if (response.success && response.data) {
        archivedCampaigns.value = response.data
      } else {
        error.value = response.error || 'Failed to fetch archived campaigns'
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Unknown error occurred'
    } finally {
      loading.value = false
    }
  }

  // Archive campaign
  const archiveCampaign = async (id: string) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<Campaign>>('archive_campaign', { campaignId: id })
      if (response.success && response.data) {
        // Remove from active campaigns
        campaigns.value = campaigns.value.filter(c => c.id !== id)
        // Add to archived campaigns
        archivedCampaigns.value.push(response.data)
        if (currentCampaign.value?.id === id) {
          currentCampaign.value = null
        }
        return true
      } else {
        error.value = response.error || 'Failed to archive campaign'
        return false
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Unknown error occurred'
      return false
    } finally {
      loading.value = false
    }
  }

  // Unarchive campaign
  const unarchiveCampaign = async (id: string) => {
    loading.value = true
    error.value = null

    try {
      console.log('Calling unarchive_campaign with:', { campaignId: id })
      const response = await invoke<ApiResponse<Campaign>>('unarchive_campaign', { campaignId: id })
      console.log('Unarchive response:', response)
      if (response.success && response.data) {
        // Remove from archived campaigns
        archivedCampaigns.value = archivedCampaigns.value.filter(c => c.id !== id)
        // Add to active campaigns
        campaigns.value.push(response.data)
        return true
      } else {
        console.error('Unarchive failed with response:', response)
        error.value = response.error || 'Failed to unarchive campaign'
        return false
      }
    } catch (e) {
      console.error('Unarchive exception:', e)
      error.value = e instanceof Error ? e.message : 'Unknown error occurred'
      return false
    } finally {
      loading.value = false
    }
  }

  // Delete campaign permanently
  const deleteCampaign = async (id: string, deleteFiles = false) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<void>>('delete_campaign', {
        request: { campaign_id: id, delete_files: deleteFiles }
      })
      if (response.success) {
        // Remove from both lists (could be in either)
        campaigns.value = campaigns.value.filter(c => c.id !== id)
        archivedCampaigns.value = archivedCampaigns.value.filter(c => c.id !== id)
        if (currentCampaign.value?.id === id) {
          currentCampaign.value = null
        }
        dataEvents.emit('campaign:deleted', { campaignId: id })
        return true
      } else {
        error.value = response.error || 'Failed to delete campaign'
        return false
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Unknown error occurred'
      return false
    } finally {
      loading.value = false
    }
  }

  // Set current campaign
  const setCurrentCampaign = (campaign: Campaign | null) => {
    currentCampaign.value = campaign
  }

  // Clear error
  const clearError = () => {
    error.value = null
  }

  return {
    campaigns,
    archivedCampaigns,
    currentCampaign,
    loading,
    error,
    fetchCampaigns,
    fetchArchivedCampaigns,
    getCampaign,
    createCampaign,
    updateCampaign,
    archiveCampaign,
    unarchiveCampaign,
    deleteCampaign,
    setCurrentCampaign,
    clearError
  }
})
