import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ApiResponse, ArchiveCounts, ArchivePreview, Campaign, CreateCampaignRequest, UpdateCampaignRequest } from '../types/api'
import { dataEvents } from '@/utils/dataEvents'

export const useCampaignStore = defineStore('campaigns', () => {
  const campaigns = ref<Campaign[]>([])
  const archivedCampaigns = ref<Campaign[]>([])
  const currentCampaign = ref<Campaign | null>(null)
  const currentCampaignSources = ref<string[]>([])
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

  // Fetch campaign sources
  const fetchCampaignSources = async (campaignId: string) => {
    try {
      const response = await invoke<ApiResponse<string[]>>('list_campaign_sources', { campaignId })
      if (response.success && response.data) {
        currentCampaignSources.value = response.data
        return response.data
      }
      // If no sources configured, empty array means "allow all"
      currentCampaignSources.value = []
      return []
    } catch (e) {
      console.error('Failed to fetch campaign sources:', e)
      currentCampaignSources.value = []
      return []
    }
  }

  // Set current campaign (also loads sources)
  const setCurrentCampaign = async (campaign: Campaign | null) => {
    currentCampaign.value = campaign
    if (campaign) {
      await fetchCampaignSources(campaign.id)
    } else {
      currentCampaignSources.value = []
    }
  }

  // Refresh campaign sources (call after editing sources)
  const refreshCampaignSources = async () => {
    if (currentCampaign.value) {
      await fetchCampaignSources(currentCampaign.value.id)
    }
  }

  // Clear error
  const clearError = () => {
    error.value = null
  }

  // Export campaign archive
  const exportCampaign = async (campaignId: string, outputDirectory: string): Promise<{ archive_path: string; size_bytes: number } | null> => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<{ archive_path: string; size_bytes: number }>>('export_campaign', {
        campaignId,
        outputDir: outputDirectory
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

  // Import campaign archive
  const importCampaign = async (archivePath: string, newName?: string): Promise<Campaign | null> => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<{ campaign_id: string; campaign_name: string; counts: ArchiveCounts }>>('import_campaign', {
        archivePath,
        newName: newName || null
      })
      if (response.success && response.data) {
        // Refresh campaigns list to include the new one
        await fetchCampaigns()
        // Find and return the newly imported campaign
        const imported = campaigns.value.find(c => c.id === response.data!.campaign_id)
        return imported || null
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

  // Preview archive contents
  const previewArchive = async (archivePath: string): Promise<ArchivePreview | null> => {
    loading.value = true
    error.value = null

    try {
      const response = await invoke<ApiResponse<ArchivePreview>>('preview_archive', {
        archivePath
      })
      if (response.success && response.data) {
        return response.data
      } else {
        error.value = response.error || 'Failed to read archive'
        return null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Unknown error occurred'
      return null
    } finally {
      loading.value = false
    }
  }

  // Get archive file extension
  const getArchiveExtension = (): string => {
    return 'mimir-campaign.tar.gz'
  }

  return {
    campaigns,
    archivedCampaigns,
    currentCampaign,
    currentCampaignSources,
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
    fetchCampaignSources,
    refreshCampaignSources,
    clearError,
    exportCampaign,
    importCampaign,
    previewArchive,
    getArchiveExtension
  }
})
