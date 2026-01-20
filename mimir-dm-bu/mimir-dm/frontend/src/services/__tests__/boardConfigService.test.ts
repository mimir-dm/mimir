import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { boardConfigService, type BoardConfiguration, type StageDefinition } from '../boardConfigService'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

const mockInvoke = vi.mocked(invoke)

// Helper to create a mock board configuration
function createMockBoardConfig(overrides: Partial<BoardConfiguration> = {}): BoardConfiguration {
  return {
    boardType: 'campaign',
    stages: [
      {
        key: 'concept',
        displayName: 'Concept',
        description: 'Initial campaign concept',
        requiredDocuments: ['campaign_pitch'],
        optionalDocuments: ['world_primer'],
        metadata: {
          displayName: 'Concept',
          description: 'Initial campaign concept',
          completionMessage: 'Concept complete!',
          transitionPrompt: 'Ready to design?',
          helpText: 'Define your campaign concept'
        }
      },
      {
        key: 'design',
        displayName: 'Design',
        description: 'Campaign design phase',
        requiredDocuments: ['starting_scenario', 'character_guidelines'],
        optionalDocuments: ['table_expectations'],
        metadata: {
          displayName: 'Design',
          description: 'Campaign design phase'
        }
      },
      {
        key: 'active',
        displayName: 'Active',
        description: 'Campaign in progress',
        requiredDocuments: [],
        optionalDocuments: ['play_notes'],
        metadata: {
          displayName: 'Active',
          description: 'Campaign in progress'
        }
      }
    ],
    transitions: {
      'concept': ['design'],
      'design': ['active'],
      'active': []
    },
    ...overrides
  }
}

// Helper to create backend response format
function createBackendResponse(config: BoardConfiguration) {
  return {
    board_type: config.boardType,
    stages: config.stages.map(stage => ({
      key: stage.key,
      display_name: stage.displayName,
      description: stage.description,
      required_documents: stage.requiredDocuments,
      optional_documents: stage.optionalDocuments,
      completion_message: stage.metadata.completionMessage,
      transition_prompt: stage.metadata.transitionPrompt,
      help_text: stage.metadata.helpText
    })),
    transitions: config.transitions
  }
}

describe('boardConfigService', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Clear cached boards by fetching a fresh instance
    // Since it's a singleton, we need to clear its internal state
    // We'll do this by caching an empty config and removing it
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('fetchBoardConfig', () => {
    it('fetches and transforms board configuration from backend', async () => {
      const mockConfig = createMockBoardConfig()
      const backendResponse = createBackendResponse(mockConfig)
      mockInvoke.mockResolvedValueOnce(backendResponse)

      const result = await boardConfigService.fetchBoardConfig('campaign')

      expect(mockInvoke).toHaveBeenCalledWith('get_board_configuration', { boardType: 'campaign' })
      expect(result.boardType).toBe('campaign')
      expect(result.stages).toHaveLength(3)
      expect(result.stages[0].key).toBe('concept')
      expect(result.stages[0].displayName).toBe('Concept')
      expect(result.stages[0].requiredDocuments).toEqual(['campaign_pitch'])
    })

    it('handles API response wrapper with data property', async () => {
      const mockConfig = createMockBoardConfig()
      const backendResponse = createBackendResponse(mockConfig)
      mockInvoke.mockResolvedValueOnce({ data: backendResponse })

      const result = await boardConfigService.fetchBoardConfig('campaign')

      expect(result.boardType).toBe('campaign')
    })

    it('caches configuration after fetching', async () => {
      const mockConfig = createMockBoardConfig()
      const backendResponse = createBackendResponse(mockConfig)
      mockInvoke.mockResolvedValueOnce(backendResponse)

      await boardConfigService.fetchBoardConfig('campaign')
      const cached = boardConfigService.getBoardConfig('campaign')

      expect(cached).toBeDefined()
      expect(cached?.boardType).toBe('campaign')
    })

    it('throws error on fetch failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network error'))

      await expect(boardConfigService.fetchBoardConfig('campaign'))
        .rejects.toThrow('Failed to fetch board configuration for campaign')
    })

    it('transforms snake_case to camelCase in response', async () => {
      const backendResponse = {
        board_type: 'module',
        stages: [{
          key: 'planning',
          display_name: 'Planning',
          description: 'Module planning',
          required_documents: ['module_outline'],
          optional_documents: [],
          completion_message: 'Planning done',
          transition_prompt: 'Start design?',
          help_text: 'Plan your module'
        }],
        transitions: { 'planning': ['design'] }
      }
      mockInvoke.mockResolvedValueOnce(backendResponse)

      const result = await boardConfigService.fetchBoardConfig('module')

      expect(result.stages[0].displayName).toBe('Planning')
      expect(result.stages[0].requiredDocuments).toEqual(['module_outline'])
      expect(result.stages[0].metadata.completionMessage).toBe('Planning done')
      expect(result.stages[0].metadata.transitionPrompt).toBe('Start design?')
      expect(result.stages[0].metadata.helpText).toBe('Plan your module')
    })

    it('handles missing optional fields in backend response', async () => {
      const backendResponse = {
        board_type: 'simple',
        stages: [{
          key: 'start',
          display_name: 'Start',
          description: 'Starting phase'
          // No required_documents, optional_documents, completion_message, etc.
        }],
        transitions: {}
      }
      mockInvoke.mockResolvedValueOnce(backendResponse)

      const result = await boardConfigService.fetchBoardConfig('simple')

      expect(result.stages[0].requiredDocuments).toEqual([])
      expect(result.stages[0].optionalDocuments).toEqual([])
    })
  })

  describe('cacheBoard', () => {
    it('caches a board configuration', () => {
      const config = createMockBoardConfig({ boardType: 'test-board' })

      boardConfigService.cacheBoard(config)
      const cached = boardConfigService.getBoardConfig('test-board')

      expect(cached).toEqual(config)
    })

    it('overwrites existing cached configuration', () => {
      const config1 = createMockBoardConfig({ boardType: 'overwrite-test' })
      const config2 = createMockBoardConfig({
        boardType: 'overwrite-test',
        stages: [{ key: 'new', displayName: 'New', description: 'New stage', requiredDocuments: [], optionalDocuments: [], metadata: { displayName: 'New', description: 'New stage' } }]
      })

      boardConfigService.cacheBoard(config1)
      boardConfigService.cacheBoard(config2)
      const cached = boardConfigService.getBoardConfig('overwrite-test')

      expect(cached?.stages).toHaveLength(1)
      expect(cached?.stages[0].key).toBe('new')
    })
  })

  describe('getBoardConfig', () => {
    it('returns undefined for non-cached board type', () => {
      const result = boardConfigService.getBoardConfig('non-existent-board')

      expect(result).toBeUndefined()
    })

    it('returns cached configuration', () => {
      const config = createMockBoardConfig({ boardType: 'cached-board' })
      boardConfigService.cacheBoard(config)

      const result = boardConfigService.getBoardConfig('cached-board')

      expect(result).toEqual(config)
    })
  })

  describe('getStageDefinition', () => {
    beforeEach(() => {
      const config = createMockBoardConfig({ boardType: 'stage-test' })
      boardConfigService.cacheBoard(config)
    })

    it('returns stage definition for valid stage key', () => {
      const stage = boardConfigService.getStageDefinition('stage-test', 'concept')

      expect(stage).toBeDefined()
      expect(stage?.key).toBe('concept')
      expect(stage?.displayName).toBe('Concept')
    })

    it('returns undefined for invalid stage key', () => {
      const stage = boardConfigService.getStageDefinition('stage-test', 'invalid-stage')

      expect(stage).toBeUndefined()
    })

    it('returns undefined for non-cached board type', () => {
      const stage = boardConfigService.getStageDefinition('non-existent', 'concept')

      expect(stage).toBeUndefined()
    })
  })

  describe('getStageDocuments', () => {
    beforeEach(() => {
      const config = createMockBoardConfig({ boardType: 'doc-test' })
      boardConfigService.cacheBoard(config)
    })

    it('returns documents with metadata for known document types', () => {
      const docs = boardConfigService.getStageDocuments('doc-test', 'concept')

      expect(docs).toHaveLength(2) // 1 required + 1 optional

      const campaignPitch = docs.find(d => d.templateId === 'campaign_pitch')
      expect(campaignPitch).toBeDefined()
      expect(campaignPitch?.category).toBe('required')
      expect(campaignPitch?.title).toBe('Campaign Pitch')
      expect(campaignPitch?.description).toContain('one-page pitch')
      expect(campaignPitch?.tips).toBeDefined()
      expect(campaignPitch?.tips?.length).toBeGreaterThan(0)
    })

    it('returns optional documents with correct category', () => {
      const docs = boardConfigService.getStageDocuments('doc-test', 'concept')

      const worldPrimer = docs.find(d => d.templateId === 'world_primer')
      expect(worldPrimer).toBeDefined()
      expect(worldPrimer?.category).toBe('optional')
    })

    it('returns fallback metadata for unknown document types', () => {
      const config: BoardConfiguration = {
        boardType: 'unknown-docs',
        stages: [{
          key: 'test',
          displayName: 'Test',
          description: 'Test stage',
          requiredDocuments: ['unknown_doc_type'],
          optionalDocuments: ['another_unknown'],
          metadata: { displayName: 'Test', description: 'Test stage' }
        }],
        transitions: {}
      }
      boardConfigService.cacheBoard(config)

      const docs = boardConfigService.getStageDocuments('unknown-docs', 'test')

      expect(docs).toHaveLength(2)
      expect(docs[0].templateId).toBe('unknown_doc_type')
      expect(docs[0].title).toBe('Unknown Doc Type') // Formatted from ID
      expect(docs[0].description).toBe('Document: unknown_doc_type')
    })

    it('returns empty array for non-existent stage', () => {
      const docs = boardConfigService.getStageDocuments('doc-test', 'non-existent')

      expect(docs).toEqual([])
    })

    it('returns empty array for non-cached board', () => {
      const docs = boardConfigService.getStageDocuments('non-existent-board', 'concept')

      expect(docs).toEqual([])
    })
  })

  describe('canTransition', () => {
    beforeEach(() => {
      const config = createMockBoardConfig({ boardType: 'transition-test' })
      boardConfigService.cacheBoard(config)
    })

    it('returns true for valid transition', () => {
      const result = boardConfigService.canTransition('transition-test', 'concept', 'design')

      expect(result).toBe(true)
    })

    it('returns false for invalid transition', () => {
      const result = boardConfigService.canTransition('transition-test', 'concept', 'active')

      expect(result).toBe(false)
    })

    it('returns false for transition from non-existent stage', () => {
      const result = boardConfigService.canTransition('transition-test', 'invalid', 'design')

      expect(result).toBe(false)
    })

    it('returns false for non-cached board', () => {
      const result = boardConfigService.canTransition('non-existent', 'concept', 'design')

      expect(result).toBe(false)
    })

    it('returns false when transitioning to same stage', () => {
      const result = boardConfigService.canTransition('transition-test', 'concept', 'concept')

      expect(result).toBe(false)
    })
  })

  describe('getNextStage', () => {
    beforeEach(() => {
      const config = createMockBoardConfig({ boardType: 'next-stage-test' })
      boardConfigService.cacheBoard(config)
    })

    it('returns next stage in progression', () => {
      const next = boardConfigService.getNextStage('next-stage-test', 'concept')

      expect(next).toBe('design')
    })

    it('returns undefined for last stage', () => {
      const next = boardConfigService.getNextStage('next-stage-test', 'active')

      expect(next).toBeUndefined()
    })

    it('returns undefined for non-existent stage', () => {
      const next = boardConfigService.getNextStage('next-stage-test', 'invalid')

      expect(next).toBeUndefined()
    })

    it('returns undefined for non-cached board', () => {
      const next = boardConfigService.getNextStage('non-existent', 'concept')

      expect(next).toBeUndefined()
    })
  })

  describe('calculateStageCompletion', () => {
    const stage: StageDefinition = {
      key: 'test',
      displayName: 'Test',
      description: 'Test stage',
      requiredDocuments: ['doc1', 'doc2', 'doc3'],
      optionalDocuments: [],
      metadata: { displayName: 'Test', description: 'Test stage' }
    }

    it('calculates 0% completion when no documents completed', () => {
      const completedDocs = new Set<string>()

      const result = boardConfigService.calculateStageCompletion(stage, completedDocs)

      expect(result.completed).toBe(0)
      expect(result.total).toBe(3)
      expect(result.percentage).toBe(0)
      expect(result.missingDocuments).toEqual(['doc1', 'doc2', 'doc3'])
      expect(result.isComplete).toBe(false)
    })

    it('calculates partial completion correctly', () => {
      const completedDocs = new Set(['doc1', 'doc2'])

      const result = boardConfigService.calculateStageCompletion(stage, completedDocs)

      expect(result.completed).toBe(2)
      expect(result.total).toBe(3)
      expect(result.percentage).toBe(67) // Rounded
      expect(result.missingDocuments).toEqual(['doc3'])
      expect(result.isComplete).toBe(false)
    })

    it('calculates 100% completion when all documents completed', () => {
      const completedDocs = new Set(['doc1', 'doc2', 'doc3'])

      const result = boardConfigService.calculateStageCompletion(stage, completedDocs)

      expect(result.completed).toBe(3)
      expect(result.total).toBe(3)
      expect(result.percentage).toBe(100)
      expect(result.missingDocuments).toEqual([])
      expect(result.isComplete).toBe(true)
    })

    it('handles stage with no required documents', () => {
      const emptyStage: StageDefinition = {
        ...stage,
        requiredDocuments: []
      }
      const completedDocs = new Set<string>()

      const result = boardConfigService.calculateStageCompletion(emptyStage, completedDocs)

      expect(result.completed).toBe(0)
      expect(result.total).toBe(0)
      expect(result.percentage).toBe(0)
      expect(result.missingDocuments).toEqual([])
      expect(result.isComplete).toBe(false) // Not complete because no documents required
    })

    it('ignores extra completed documents not in required list', () => {
      const completedDocs = new Set(['doc1', 'doc2', 'doc3', 'extra_doc', 'another_extra'])

      const result = boardConfigService.calculateStageCompletion(stage, completedDocs)

      expect(result.completed).toBe(3)
      expect(result.total).toBe(3)
      expect(result.percentage).toBe(100)
    })
  })

  describe('document metadata', () => {
    beforeEach(() => {
      const config = createMockBoardConfig({
        boardType: 'metadata-test',
        stages: [{
          key: 'full',
          displayName: 'Full',
          description: 'All documents',
          requiredDocuments: [
            'campaign_pitch',
            'starting_scenario',
            'world_primer',
            'character_guidelines',
            'table_expectations',
            'character_integration',
            'campaign_bible',
            'major_npc_tracker',
            'safety_tools',
            'house_rules',
            'player_secrets',
            'faction_overview',
            'play_notes',
            'player_handouts'
          ],
          optionalDocuments: [],
          metadata: { displayName: 'Full', description: 'All documents' }
        }]
      })
      boardConfigService.cacheBoard(config)
    })

    it('provides metadata for all known document types', () => {
      const docs = boardConfigService.getStageDocuments('metadata-test', 'full')

      expect(docs).toHaveLength(14)

      // Check each document has proper metadata
      const expectedDocs = [
        { id: 'campaign_pitch', title: 'Campaign Pitch' },
        { id: 'starting_scenario', title: 'Starting Scenario' },
        { id: 'world_primer', title: 'World Primer' },
        { id: 'character_guidelines', title: 'Character Guidelines' },
        { id: 'table_expectations', title: 'Table Expectations' },
        { id: 'character_integration', title: 'Character Integration' },
        { id: 'campaign_bible', title: 'Campaign Bible' },
        { id: 'major_npc_tracker', title: 'Major NPC Tracker (Deprecated)' },
        { id: 'safety_tools', title: 'Safety Tools' },
        { id: 'house_rules', title: 'House Rules' },
        { id: 'player_secrets', title: 'Player Secrets' },
        { id: 'faction_overview', title: 'Faction Overview' },
        { id: 'play_notes', title: 'Play Notes' },
        { id: 'player_handouts', title: 'Player Handouts' }
      ]

      for (const expected of expectedDocs) {
        const doc = docs.find(d => d.templateId === expected.id)
        expect(doc, `Document ${expected.id} should exist`).toBeDefined()
        expect(doc?.title).toBe(expected.title)
        expect(doc?.description).toBeTruthy()
      }
    })

    it('includes tips for documents that have them', () => {
      const docs = boardConfigService.getStageDocuments('metadata-test', 'full')

      const campaignPitch = docs.find(d => d.templateId === 'campaign_pitch')
      expect(campaignPitch?.tips).toBeDefined()
      expect(campaignPitch?.tips?.length).toBeGreaterThan(0)
      expect(campaignPitch?.tips?.[0]).toContain('concise')
    })
  })
})
