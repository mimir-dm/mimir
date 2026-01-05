import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ApiResponse, Campaign, NewCampaign } from '../types/api'

export const useCampaignStore = defineStore('campaigns', () => {
  const campaigns = ref<Campaign[]>([])
  const archivedCampaigns = ref<Campaign[]>([])
  const currentCampaign = ref<Campaign | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  
  // Fetch all campaigns
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
  const getCampaign = async (id: number) => {
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
  const createCampaign = async (data: NewCampaign) => {
    loading.value = true
    error.value = null
    
    try {
      const response = await invoke<ApiResponse<Campaign>>('create_campaign', { request: data })
      if (response.success && response.data) {
        campaigns.value.push(response.data)
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
  
  // Update campaign status
  const updateCampaignStatus = async (id: number, status: string) => {
    loading.value = true
    error.value = null
    
    try {
      const response = await invoke<ApiResponse<Campaign>>('update_campaign_status', { id, status })
      if (response.success && response.data) {
        // Update local state
        const index = campaigns.value.findIndex(c => c.id === id)
        if (index !== -1) {
          campaigns.value[index] = response.data
        }
        if (currentCampaign.value?.id === id) {
          currentCampaign.value = response.data
        }
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
  const archiveCampaign = async (id: number) => {
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
  const unarchiveCampaign = async (id: number) => {
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
  
  // Delete campaign (hard delete - only for archived campaigns)
  const deleteCampaign = async (id: number, deleteFiles = false) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<void>>('delete_campaign', {
        request: { campaign_id: id, delete_files: deleteFiles }
      })
      if (response.success) {
        // Remove from archived campaigns
        archivedCampaigns.value = archivedCampaigns.value.filter(c => c.id !== id)
        if (currentCampaign.value?.id === id) {
          currentCampaign.value = null
        }
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

  // Export campaign to archive
  const exportCampaign = async (campaignId: number, outputDirectory: string) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<{ archive_path: string; file_name: string }>>('export_campaign_archive', {
        campaignId,
        outputDirectory
      })
      if (response.success && response.data) {
        return response.data
      } else {
        error.value = response.error || 'Failed to export campaign'
        return null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Unknown error occurred'
      return null
    } finally {
      loading.value = false
    }
  }

  // Preview an archive before importing
  const previewArchive = async (archivePath: string) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<{
        campaign_name: string
        file_count: number
        asset_count: number
        catalog_references: Array<{ type: string; name: string; source: string }>
        mimir_version: string
        created_at: string
      }>>('preview_campaign_archive', { archivePath })
      if (response.success && response.data) {
        return response.data
      } else {
        error.value = response.error || 'Failed to preview archive'
        return null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Unknown error occurred'
      return null
    } finally {
      loading.value = false
    }
  }

  // Import campaign from archive
  const importCampaign = async (archivePath: string, campaignName: string, campaignsDirectory: string) => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<Campaign>>('import_campaign_archive', {
        request: {
          archive_path: archivePath,
          campaign_name: campaignName,
          campaigns_directory: campaignsDirectory
        }
      })
      if (response.success && response.data) {
        // Add the new campaign to the list
        campaigns.value.push(response.data)
        return response.data
      } else {
        error.value = response.error || 'Failed to import campaign'
        return null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Unknown error occurred'
      return null
    } finally {
      loading.value = false
    }
  }

  // Get the archive file extension
  const getArchiveExtension = async (): Promise<string> => {
    try {
      const response = await invoke<ApiResponse<string>>('get_campaign_archive_extension')
      if (response.success && response.data) {
        return response.data
      }
      return '.mimir-campaign.tar.gz'
    } catch {
      return '.mimir-campaign.tar.gz'
    }
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
    updateCampaignStatus,
    archiveCampaign,
    unarchiveCampaign,
    deleteCampaign,
    exportCampaign,
    previewArchive,
    importCampaign,
    getArchiveExtension
  }
})