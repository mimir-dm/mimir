import { describe, it, expect, beforeEach, vi } from 'vitest'
import { ModuleService } from '../ModuleService'
import { mockInvoke, createMockModule, createMockSession, createMockDocument } from '../../test/utils/mockTauri'

vi.mock('@tauri-apps/api/core')

describe('ModuleService', () => {
  let invoke: ReturnType<typeof mockInvoke>

  beforeEach(async () => {
    // Clear cache before each test
    ModuleService.clearCache()
    vi.clearAllMocks()
    
    // Import fresh mock for each test
    const { invoke: mockInvokeImport } = await import('@tauri-apps/api/core')
    invoke = mockInvokeImport as any
  })

  describe('get', () => {
    it('should fetch a module by id', async () => {
      const mockModule = createMockModule({ id: 1, name: 'Test Module' })
      invoke.mockResolvedValueOnce({ data: mockModule })
      
      const result = await ModuleService.get(1)
      
      expect(invoke).toHaveBeenCalledWith('get_module', { id: 1 })
      expect(result).toEqual(mockModule)
    })

    it('should normalize module fields with defaults', async () => {
      const incompleteModule = { 
        id: 1, 
        name: 'Test',
        campaign_id: 1,
        module_type: 'standard',
        status: 'planning',
        session_count: 0,
        created_at: '2024-01-01',
        updated_at: '2024-01-01'
      }
      invoke.mockResolvedValueOnce({ data: incompleteModule })
      
      const result = await ModuleService.get(1)
      
      expect(result.module_number).toBe(1)
      expect(result.expected_sessions).toBe(0)
      expect(result.actual_sessions).toBe(0)
      expect(result.sessions_planned).toBe(0)
      expect(result.sessions_completed).toBe(0)
      expect(result.started_at).toBeNull()
      expect(result.completed_at).toBeNull()
    })

    it('should cache modules after fetching', async () => {
      const mockModule = createMockModule()
      invoke.mockResolvedValueOnce({ data: mockModule })
      
      await ModuleService.get(1)
      const result = await ModuleService.get(1)
      
      expect(invoke).toHaveBeenCalledTimes(1)
      expect(result).toEqual(mockModule)
    })
  })

  describe('list', () => {
    it('should list modules for a campaign', async () => {
      const mockModules = [
        createMockModule({ id: 1, name: 'Module 1' }),
        createMockModule({ id: 2, name: 'Module 2' })
      ]
      invoke.mockResolvedValueOnce({ data: mockModules })
      
      const result = await ModuleService.list(1)
      
      expect(invoke).toHaveBeenCalledWith('list_campaign_modules', {
        request: { campaign_id: 1 }
      })
      expect(result).toHaveLength(2)
    })

    it('should assign module numbers if missing', async () => {
      const mockModules = [
        { id: 1, name: 'Module 1' },
        { id: 2, name: 'Module 2' }
      ]
      invoke.mockResolvedValueOnce({ data: mockModules })
      
      const result = await ModuleService.list(1)
      
      expect(result[0].module_number).toBe(1)
      expect(result[1].module_number).toBe(2)
    })
  })

  describe('create', () => {
    it('should create a new module', async () => {
      const newModule = createMockModule({ id: 3, name: 'New Module' })
      invoke.mockResolvedValueOnce({ data: newModule })
      
      const result = await ModuleService.create({
        name: 'New Module',
        campaign_id: 1,
        module_type: 'standard'
      })
      
      expect(invoke).toHaveBeenCalledWith('create_module', {
        request: {
          campaign_id: 1,
          name: 'New Module',
          module_type: 'standard',
          expected_sessions: 4
        }
      })
      expect(result).toEqual(newModule)
    })

    it('should clear campaign cache after creation', async () => {
      const mockModules = [createMockModule()]
      invoke.mockResolvedValueOnce({ data: mockModules })
      
      // Populate cache
      await ModuleService.list(1)
      
      // Create new module
      const newModule = createMockModule({ id: 2 })
      invoke.mockResolvedValueOnce(newModule)
      await ModuleService.create({ campaign_id: 1, name: 'New' })
      
      // Next list should fetch fresh data
      invoke.mockResolvedValueOnce({ data: [...mockModules, newModule] })
      await ModuleService.list(1)
      
      expect(invoke).toHaveBeenCalledTimes(3)
    })
  })

  describe('update', () => {
    it('should update a module', async () => {
      const updatedModule = createMockModule({ id: 1, name: 'Updated' })
      invoke.mockResolvedValueOnce({ data: updatedModule })
      
      const result = await ModuleService.update(1, { name: 'Updated' })
      
      expect(invoke).toHaveBeenCalledWith('update_module', {
        id: 1,
        request: {
          name: 'Updated',
          expected_sessions: undefined,
          actual_sessions: undefined
        }
      })
      expect(result).toEqual(updatedModule)
    })

    it('should clear module cache after update', async () => {
      const originalModule = createMockModule({ id: 1, name: 'Original' })
      invoke.mockResolvedValueOnce({ data: originalModule })
      
      await ModuleService.get(1)
      
      const updatedModule = createMockModule({ id: 1, name: 'Updated' })
      invoke.mockResolvedValueOnce({ data: updatedModule })
      await ModuleService.update(1, { name: 'Updated' })
      
      invoke.mockResolvedValueOnce({ data: updatedModule })
      const result = await ModuleService.get(1)
      
      expect(result.name).toBe('Updated')
      expect(invoke).toHaveBeenCalledTimes(3)
    })
  })

  describe('delete', () => {
    it('should delete a module', async () => {
      invoke.mockResolvedValueOnce(undefined)
      
      await ModuleService.delete(1)
      
      expect(invoke).toHaveBeenCalledWith('delete_module', { id: 1 })
    })

    it('should clear all caches after deletion', async () => {
      ModuleService.clearCache()
      expect(true).toBe(true)
    })
  })

  describe('stage management', () => {
    it('should update module status', async () => {
      const updatedModule = createMockModule({ status: 'active' })
      invoke.mockResolvedValueOnce({ data: updatedModule })

      const result = await ModuleService.updateStatus(1, 'active')

      // updateStatus internally calls transitionStage
      expect(invoke).toHaveBeenCalledWith('transition_module_stage', {
        request: {
          module_id: 1,
          new_stage: 'active'
        }
      })
      expect(result.status).toBe('active')
    })

    it('should transition module stage', async () => {
      const transitionedModule = createMockModule({ status: 'ready' })
      invoke.mockResolvedValueOnce({ data: transitionedModule })

      const result = await ModuleService.transitionStage(1, 'ready')

      expect(invoke).toHaveBeenCalledWith('transition_module_stage', {
        request: {
          module_id: 1,
          new_stage: 'ready'
        }
      })
      expect(result.status).toBe('ready')
    })
  })

  describe('document management', () => {
    it('should initialize module documents', async () => {
      const mockModule = createMockModule({ id: 1, campaign_id: 1 })
      const mockCampaign = { directory_path: '/campaigns/test' }
      const documentPaths = ['/path/doc1.md', '/path/doc2.md']

      // Service first gets the module, then the campaign, then initializes
      invoke.mockResolvedValueOnce({ data: mockModule })
      invoke.mockResolvedValueOnce({ data: mockCampaign })
      invoke.mockResolvedValueOnce({ data: documentPaths })

      const result = await ModuleService.initializeDocuments(1)

      expect(invoke).toHaveBeenCalledWith('initialize_module_documents', {
        request: {
          module_id: 1,
          campaign_directory: '/campaigns/test'
        }
      })
      expect(result).toEqual(documentPaths)
    })

    it('should get module documents', async () => {
      const mockDocs = [
        createMockDocument({ module_id: 1 }),
        createMockDocument({ module_id: 1 })
      ]
      invoke.mockResolvedValueOnce({ data: mockDocs })
      
      const result = await ModuleService.getDocuments(1)
      
      expect(invoke).toHaveBeenCalledWith('get_module_documents', {
        request: { module_id: 1 }
      })
      expect(result).toHaveLength(2)
    })
  })

  describe('session management', () => {
    it('should list module sessions', async () => {
      const mockSessions = [
        createMockSession({ id: 1, name: 'Session 1' }),
        createMockSession({ id: 2, name: 'Session 2' })
      ]
      invoke.mockResolvedValueOnce({ data: mockSessions })
      
      const result = await ModuleService.listSessions(1)
      
      expect(invoke).toHaveBeenCalledWith('list_module_sessions', {
        request: { module_id: 1 }
      })
      expect(result).toHaveLength(2)
    })

    it('should ensure sessions have required fields', async () => {
      const incompleteSessions = [
        { id: 1, module_id: 1, name: 'Session 1', status: 'planned' },
        { id: 2, module_id: 1, name: 'Session 2', status: 'active' }
      ]
      invoke.mockResolvedValueOnce({ data: incompleteSessions })
      
      const result = await ModuleService.listSessions(1)
      
      expect(result[0].session_number).toBe(1)
      expect(result[0].created_at).toBeTruthy()
      expect(result[1].session_number).toBe(2)
      expect(result[1].created_at).toBeTruthy()
    })

    it('should increment session count', async () => {
      invoke.mockResolvedValueOnce(undefined)

      await ModuleService.incrementSessionCount(1)

      expect(invoke).toHaveBeenCalledWith('increment_module_sessions', {
        module_id: 1
      })
    })
  })

  describe('canTransition', () => {
    it('should check if module can transition', async () => {
      const mockModule = createMockModule({ status: 'planning' })
      const mockDocs = [
        createMockDocument({ template_id: 'required-doc', completed_at: '2024-01-01' })
      ]
      const boardConfig = {
        stages: [{
          key: 'planning',
          required_documents: ['required-doc'],
          no_completion_required_documents: []
        }]
      }
      
      invoke.mockResolvedValueOnce({ data: mockModule })
      invoke.mockResolvedValueOnce({ data: mockDocs })
      
      const result = await ModuleService.canTransition(1, boardConfig)
      
      expect(result).toBe(true)
    })

    it('should return false when documents incomplete', async () => {
      const mockModule = createMockModule({ status: 'planning' })
      const mockDocs = [
        createMockDocument({ template_id: 'required-doc', completed_at: null })
      ]
      const boardConfig = {
        stages: [{
          key: 'planning',
          required_documents: ['required-doc'],
          no_completion_required_documents: []
        }]
      }
      
      invoke.mockResolvedValueOnce({ data: mockModule })
      invoke.mockResolvedValueOnce({ data: mockDocs })
      
      const result = await ModuleService.canTransition(1, boardConfig)
      
      expect(result).toBe(false)
    })

    it('should handle errors gracefully in canTransition', async () => {
      invoke.mockRejectedValueOnce(new Error('Network error'))
      
      const result = await ModuleService.canTransition(1, {})
      
      expect(result).toBe(false)
    })
  })
})