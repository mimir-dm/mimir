import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useCampaignStore } from '../campaigns'
import { invoke } from '@tauri-apps/api/core'
import type { Campaign } from '../../types/api'

// Get the mocked invoke function
const mockInvoke = vi.mocked(invoke)

// Helper to create mock campaign
const createMockCampaign = (overrides: Partial<Campaign> = {}): Campaign => ({
  id: 1,
  name: 'Test Campaign',
  status: 'concept',
  directory_path: '/campaigns/test',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
  archived_at: null,
  ...overrides
})

describe('useCampaignStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('has empty campaigns array', () => {
      const store = useCampaignStore()
      expect(store.campaigns).toEqual([])
    })

    it('has empty archived campaigns array', () => {
      const store = useCampaignStore()
      expect(store.archivedCampaigns).toEqual([])
    })

    it('has null current campaign', () => {
      const store = useCampaignStore()
      expect(store.currentCampaign).toBeNull()
    })

    it('is not loading initially', () => {
      const store = useCampaignStore()
      expect(store.loading).toBe(false)
    })

    it('has no error initially', () => {
      const store = useCampaignStore()
      expect(store.error).toBeNull()
    })
  })

  describe('fetchCampaigns', () => {
    it('fetches campaigns successfully', async () => {
      const mockCampaigns = [
        createMockCampaign({ id: 1, name: 'Campaign 1' }),
        createMockCampaign({ id: 2, name: 'Campaign 2' })
      ]
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockCampaigns })

      const store = useCampaignStore()
      await store.fetchCampaigns()

      expect(mockInvoke).toHaveBeenCalledWith('list_campaigns')
      expect(store.campaigns).toEqual(mockCampaigns)
      expect(store.loading).toBe(false)
      expect(store.error).toBeNull()
    })

    it('sets loading state during fetch', async () => {
      mockInvoke.mockImplementation(() => new Promise(() => {})) // Never resolves

      const store = useCampaignStore()
      const promise = store.fetchCampaigns()

      expect(store.loading).toBe(true)

      // Clean up - just ignore the promise
    })

    it('handles API error response', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false, error: 'Database error' })

      const store = useCampaignStore()
      await store.fetchCampaigns()

      expect(store.campaigns).toEqual([])
      expect(store.error).toBe('Database error')
      expect(store.loading).toBe(false)
    })

    it('handles exception during fetch', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network failure'))

      const store = useCampaignStore()
      await store.fetchCampaigns()

      expect(store.campaigns).toEqual([])
      expect(store.error).toBe('Network failure')
      expect(store.loading).toBe(false)
    })

    it('clears error before fetching', async () => {
      const store = useCampaignStore()
      store.error = 'Previous error'

      mockInvoke.mockResolvedValueOnce({ success: true, data: [] })
      await store.fetchCampaigns()

      expect(store.error).toBeNull()
    })
  })

  describe('getCampaign', () => {
    it('fetches single campaign successfully', async () => {
      const mockCampaign = createMockCampaign({ id: 1 })
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockCampaign })

      const store = useCampaignStore()
      const result = await store.getCampaign(1)

      expect(mockInvoke).toHaveBeenCalledWith('get_campaign', { id: 1 })
      expect(result).toEqual(mockCampaign)
      expect(store.currentCampaign).toEqual(mockCampaign)
    })

    it('returns null on failure', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false, error: 'Not found' })

      const store = useCampaignStore()
      const result = await store.getCampaign(999)

      expect(result).toBeNull()
      expect(store.error).toBe('Not found')
    })
  })

  describe('createCampaign', () => {
    it('creates campaign and adds to list', async () => {
      const newCampaign = createMockCampaign({ id: 1, name: 'New Campaign' })
      mockInvoke.mockResolvedValueOnce({ success: true, data: newCampaign })

      const store = useCampaignStore()
      const result = await store.createCampaign({
        name: 'New Campaign',
        directory_path: '/campaigns/new'
      })

      expect(mockInvoke).toHaveBeenCalledWith('create_campaign', {
        request: { name: 'New Campaign', directory_path: '/campaigns/new' }
      })
      expect(result).toEqual(newCampaign)
      expect(store.campaigns).toContainEqual(newCampaign)
    })

    it('returns null on failure', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false, error: 'Validation error' })

      const store = useCampaignStore()
      const result = await store.createCampaign({ name: '', directory_path: '' })

      expect(result).toBeNull()
      expect(store.error).toBe('Validation error')
    })
  })

  describe('updateCampaignStatus', () => {
    it('updates campaign status in list', async () => {
      const store = useCampaignStore()
      store.campaigns = [createMockCampaign({ id: 1, status: 'concept' })]

      const updatedCampaign = createMockCampaign({ id: 1, status: 'active' })
      mockInvoke.mockResolvedValueOnce({ success: true, data: updatedCampaign })

      const result = await store.updateCampaignStatus(1, 'active')

      expect(mockInvoke).toHaveBeenCalledWith('update_campaign_status', { id: 1, status: 'active' })
      expect(result).toEqual(updatedCampaign)
      expect(store.campaigns[0].status).toBe('active')
    })

    it('updates currentCampaign if matching', async () => {
      const store = useCampaignStore()
      store.currentCampaign = createMockCampaign({ id: 1, status: 'concept' })
      store.campaigns = [store.currentCampaign]

      const updatedCampaign = createMockCampaign({ id: 1, status: 'active' })
      mockInvoke.mockResolvedValueOnce({ success: true, data: updatedCampaign })

      await store.updateCampaignStatus(1, 'active')

      expect(store.currentCampaign?.status).toBe('active')
    })
  })

  describe('archiveCampaign', () => {
    it('moves campaign from active to archived', async () => {
      const store = useCampaignStore()
      const campaign = createMockCampaign({ id: 1 })
      store.campaigns = [campaign]

      const archivedCampaign = { ...campaign, archived_at: '2024-01-02T00:00:00Z' }
      mockInvoke.mockResolvedValueOnce({ success: true, data: archivedCampaign })

      const result = await store.archiveCampaign(1)

      expect(mockInvoke).toHaveBeenCalledWith('archive_campaign', { campaignId: 1 })
      expect(result).toBe(true)
      expect(store.campaigns).toHaveLength(0)
      expect(store.archivedCampaigns).toContainEqual(archivedCampaign)
    })

    it('clears currentCampaign if archived', async () => {
      const store = useCampaignStore()
      const campaign = createMockCampaign({ id: 1 })
      store.campaigns = [campaign]
      store.currentCampaign = campaign

      mockInvoke.mockResolvedValueOnce({ success: true, data: campaign })
      await store.archiveCampaign(1)

      expect(store.currentCampaign).toBeNull()
    })

    it('returns false on failure', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false, error: 'Cannot archive' })

      const store = useCampaignStore()
      const result = await store.archiveCampaign(1)

      expect(result).toBe(false)
      expect(store.error).toBe('Cannot archive')
    })
  })

  describe('unarchiveCampaign', () => {
    it('moves campaign from archived to active', async () => {
      const store = useCampaignStore()
      const archivedCampaign = createMockCampaign({ id: 1, archived_at: '2024-01-02T00:00:00Z' })
      store.archivedCampaigns = [archivedCampaign]

      const unarchivedCampaign = { ...archivedCampaign, archived_at: null }
      mockInvoke.mockResolvedValueOnce({ success: true, data: unarchivedCampaign })

      const result = await store.unarchiveCampaign(1)

      expect(mockInvoke).toHaveBeenCalledWith('unarchive_campaign', { campaignId: 1 })
      expect(result).toBe(true)
      expect(store.archivedCampaigns).toHaveLength(0)
      expect(store.campaigns).toContainEqual(unarchivedCampaign)
    })
  })

  describe('deleteCampaign', () => {
    it('removes campaign from archived list', async () => {
      const store = useCampaignStore()
      const campaign = createMockCampaign({ id: 1 })
      store.archivedCampaigns = [campaign]

      mockInvoke.mockResolvedValueOnce({ success: true })

      const result = await store.deleteCampaign(1, false)

      expect(mockInvoke).toHaveBeenCalledWith('delete_campaign', {
        request: { campaign_id: 1, delete_files: false }
      })
      expect(result).toBe(true)
      expect(store.archivedCampaigns).toHaveLength(0)
    })

    it('clears currentCampaign if deleted', async () => {
      const store = useCampaignStore()
      const campaign = createMockCampaign({ id: 1 })
      store.currentCampaign = campaign
      store.archivedCampaigns = [campaign]

      mockInvoke.mockResolvedValueOnce({ success: true })
      await store.deleteCampaign(1)

      expect(store.currentCampaign).toBeNull()
    })

    it('passes deleteFiles flag', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true })

      const store = useCampaignStore()
      await store.deleteCampaign(1, true)

      expect(mockInvoke).toHaveBeenCalledWith('delete_campaign', {
        request: { campaign_id: 1, delete_files: true }
      })
    })
  })

  describe('fetchArchivedCampaigns', () => {
    it('fetches archived campaigns', async () => {
      const archivedCampaigns = [
        createMockCampaign({ id: 1, archived_at: '2024-01-02T00:00:00Z' })
      ]
      mockInvoke.mockResolvedValueOnce({ success: true, data: archivedCampaigns })

      const store = useCampaignStore()
      await store.fetchArchivedCampaigns()

      expect(mockInvoke).toHaveBeenCalledWith('list_archived_campaigns')
      expect(store.archivedCampaigns).toEqual(archivedCampaigns)
    })
  })

  describe('exportCampaign', () => {
    it('exports campaign and returns path info', async () => {
      const exportResult = { archive_path: '/exports/campaign.tar.gz', file_name: 'campaign.tar.gz' }
      mockInvoke.mockResolvedValueOnce({ success: true, data: exportResult })

      const store = useCampaignStore()
      const result = await store.exportCampaign(1, '/exports')

      expect(mockInvoke).toHaveBeenCalledWith('export_campaign_archive', {
        campaignId: 1,
        outputDirectory: '/exports'
      })
      expect(result).toEqual(exportResult)
    })

    it('returns null on export failure', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false, error: 'Export failed' })

      const store = useCampaignStore()
      const result = await store.exportCampaign(1, '/exports')

      expect(result).toBeNull()
      expect(store.error).toBe('Export failed')
    })
  })

  describe('previewArchive', () => {
    it('returns archive preview data', async () => {
      const previewData = {
        campaign_name: 'Test Campaign',
        file_count: 10,
        asset_count: 5,
        catalog_references: [],
        mimir_version: '0.2.0',
        created_at: '2024-01-01T00:00:00Z'
      }
      mockInvoke.mockResolvedValueOnce({ success: true, data: previewData })

      const store = useCampaignStore()
      const result = await store.previewArchive('/path/to/archive.tar.gz')

      expect(mockInvoke).toHaveBeenCalledWith('preview_campaign_archive', {
        archivePath: '/path/to/archive.tar.gz'
      })
      expect(result).toEqual(previewData)
    })
  })

  describe('importCampaign', () => {
    it('imports campaign and adds to list', async () => {
      const importedCampaign = createMockCampaign({ id: 99, name: 'Imported Campaign' })
      mockInvoke.mockResolvedValueOnce({ success: true, data: importedCampaign })

      const store = useCampaignStore()
      const result = await store.importCampaign(
        '/path/to/archive.tar.gz',
        'Imported Campaign',
        '/campaigns'
      )

      expect(mockInvoke).toHaveBeenCalledWith('import_campaign_archive', {
        request: {
          archive_path: '/path/to/archive.tar.gz',
          campaign_name: 'Imported Campaign',
          campaigns_directory: '/campaigns'
        }
      })
      expect(result).toEqual(importedCampaign)
      expect(store.campaigns).toContainEqual(importedCampaign)
    })
  })

  describe('getArchiveExtension', () => {
    it('returns extension from backend', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true, data: '.custom.tar.gz' })

      const store = useCampaignStore()
      const result = await store.getArchiveExtension()

      expect(result).toBe('.custom.tar.gz')
    })

    it('returns default extension on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Failed'))

      const store = useCampaignStore()
      const result = await store.getArchiveExtension()

      expect(result).toBe('.mimir-campaign.tar.gz')
    })
  })
})
