/**
 * Board Configuration Service
 * 
 * This service manages board configurations and stage metadata.
 * It interfaces with the backend to get board definitions and provides
 * a centralized way to access stage requirements and document templates.
 */

import { invoke } from '@tauri-apps/api/core'

export interface StageMetadata {
  displayName: string
  description: string
  completionMessage?: string
  transitionPrompt?: string
  helpText?: string
}

export interface StageDefinition {
  key: string
  displayName: string
  description: string
  requiredDocuments: string[]
  optionalDocuments: string[]
  metadata: StageMetadata
}

export interface BoardConfiguration {
  boardType: string
  stages: StageDefinition[]
  transitions: Record<string, string[]> // from stage -> allowed to stages
}

export interface DocumentMetadata {
  templateId: string
  title: string
  description: string
  category: 'required' | 'optional'
  tips?: string[]
}

class BoardConfigurationService {
  private boardConfigs: Map<string, BoardConfiguration> = new Map()
  
  /**
   * Cache a board configuration
   * Useful when configuration is already available from props
   */
  cacheBoard(config: BoardConfiguration): void {
    this.boardConfigs.set(config.boardType, config)
  }
  
  /**
   * Document metadata that enriches the template IDs from board config
   * This could eventually come from the backend
   */
  private documentMetadata: Record<string, Omit<DocumentMetadata, 'templateId' | 'category'>> = {
    'campaign_pitch': {
      title: 'Campaign Pitch',
      description: 'Your compelling one-page pitch for the campaign',
      tips: [
        'Keep it concise - one page maximum',
        'Focus on what makes your campaign unique',
        'Include the core conflict and player role'
      ]
    },
    'starting_scenario': {
      title: 'Starting Scenario',
      description: 'The opening situation that brings the party together',
      tips: [
        'Make it immediately engaging',
        'Give each character a reason to be involved',
        'Set the tone for the campaign'
      ]
    },
    'world_primer': {
      title: 'World Primer',
      description: 'Essential information about your campaign setting',
      tips: [
        'Focus on what players need to know immediately',
        'Save deep lore for later discovery',
        'Include practical information like currencies and customs'
      ]
    },
    'character_guidelines': {
      title: 'Character Guidelines',
      description: 'Rules and suggestions for character creation',
      tips: [
        'Be clear about any restrictions',
        'Suggest character concepts that fit the campaign',
        'Include any house rules for character creation'
      ]
    },
    'table_expectations': {
      title: 'Table Expectations',
      description: 'Gameplay style, safety tools, and social contract',
      tips: [
        'Discuss scheduling and attendance expectations',
        'Cover safety tools and boundaries',
        'Set tone and content expectations'
      ]
    },
    'character_integration': {
      title: 'Character Integration',
      description: 'How player characters connect to the world and story',
      tips: [
        'Find connections between character backstories',
        'Link characters to campaign themes',
        'Create shared history between characters'
      ]
    },
    'campaign_bible': {
      title: 'Campaign Bible',
      description: 'Comprehensive reference for your campaign world',
      tips: [
        'Organize information for easy reference',
        'Include maps and visual aids',
        'Track ongoing plots and mysteries'
      ]
    },
    'major_npc_tracker': {
      title: 'Major NPC Tracker (Deprecated)',
      description: 'Important non-player characters and their relationships. Consider using the Character Wizard to create NPCs with full character sheets instead.',
      tips: [
        'TIP: Use the Character Wizard to create NPCs with full character sheets',
        'NPCs created via the wizard can have ability scores, skills, and equipment',
        'Legacy: Include motivations and goals for narrative-only NPCs',
        'Legacy: Track relationships between NPCs'
      ]
    },
    'safety_tools': {
      title: 'Safety Tools',
      description: 'Safety mechanisms and content boundaries for the table',
      tips: [
        'Choose appropriate safety tools for your group',
        'Discuss implementation clearly',
        'Review and adjust as needed'
      ]
    },
    'house_rules': {
      title: 'House Rules',
      description: 'Custom rules and modifications for your campaign',
      tips: [
        'Document all rule changes clearly',
        'Explain the reasoning behind changes',
        'Be open to adjusting based on play experience'
      ]
    },
    'player_secrets': {
      title: 'Player Secrets',
      description: 'Hidden information and personal plots for each character',
      tips: [
        'Coordinate secrets between players when appropriate',
        'Plan reveal moments for maximum impact',
        'Ensure secrets enhance rather than disrupt the game'
      ]
    },
    'faction_overview': {
      title: 'Faction Overview',
      description: 'Major organizations and their goals in your world',
      tips: [
        'Define clear goals and methods for each faction',
        'Create conflicts between factions',
        'Consider how players might interact with each faction'
      ]
    },
    'play_notes': {
      title: 'Play Notes',
      description: 'Record of events and decisions from game sessions',
      tips: [
        'Note important player decisions',
        'Track unresolved plot threads',
        'Record memorable moments and quotes'
      ]
    },
    'player_handouts': {
      title: 'Player Handouts',
      description: 'In-game documents and materials for players',
      tips: [
        'Make handouts visually interesting',
        'Use handouts to convey lore naturally',
        'Consider physical props for important documents'
      ]
    }
  }

  /**
   * Fetch board configuration from the backend
   */
  async fetchBoardConfig(boardType: string): Promise<BoardConfiguration> {
    try {
      const response = await invoke('get_board_configuration', { boardType })
      const apiResponse = response as any
      
      // Handle API response wrapper
      const config = apiResponse.data || apiResponse
      
      // Transform the backend response to our frontend format
      const boardConfig: BoardConfiguration = {
        boardType: config.board_type,
        stages: config.stages.map((stage: any) => ({
          key: stage.key,
          displayName: stage.display_name,
          description: stage.description,
          requiredDocuments: stage.required_documents || [],
          optionalDocuments: stage.optional_documents || [],
          metadata: {
            displayName: stage.display_name,
            description: stage.description,
            completionMessage: stage.completion_message,
            transitionPrompt: stage.transition_prompt,
            helpText: stage.help_text
          }
        })),
        transitions: config.transitions || {}
      }
      
      // Cache the configuration
      this.boardConfigs.set(boardType, boardConfig)

      return boardConfig
    } catch (error) {
      throw new Error(`Failed to fetch board configuration for ${boardType}: ${error}`)
    }
  }

  /**
   * Get cached board configuration
   */
  getBoardConfig(boardType: string): BoardConfiguration | undefined {
    return this.boardConfigs.get(boardType)
  }

  /**
   * Get stage definition from a board
   */
  getStageDefinition(boardType: string, stageKey: string): StageDefinition | undefined {
    const config = this.boardConfigs.get(boardType)
    if (!config) return undefined
    
    return config.stages.find((stage: StageDefinition) => stage.key === stageKey)
  }

  /**
   * Get documents for a specific stage with metadata
   */
  getStageDocuments(boardType: string, stageKey: string): DocumentMetadata[] {
    const stage = this.getStageDefinition(boardType, stageKey)
    if (!stage) return []
    
    const documents: DocumentMetadata[] = []
    
    // Add required documents
    for (const docId of stage.requiredDocuments) {
      const metadata = this.documentMetadata[docId]
      if (metadata) {
        documents.push({
          templateId: docId,
          category: 'required',
          ...metadata
        })
      } else {
        // Fallback for unknown document types
        documents.push({
          templateId: docId,
          category: 'required',
          title: this.formatDocumentId(docId),
          description: `Document: ${docId}`
        })
      }
    }
    
    // Add optional documents
    for (const docId of stage.optionalDocuments) {
      const metadata = this.documentMetadata[docId]
      if (metadata) {
        documents.push({
          templateId: docId,
          category: 'optional',
          ...metadata
        })
      } else {
        documents.push({
          templateId: docId,
          category: 'optional',
          title: this.formatDocumentId(docId),
          description: `Optional document: ${docId}`
        })
      }
    }
    
    return documents
  }

  /**
   * Check if a stage transition is allowed
   */
  canTransition(boardType: string, fromStage: string, toStage: string): boolean {
    const config = this.boardConfigs.get(boardType)
    if (!config) return false
    
    const allowedTransitions = config.transitions[fromStage]
    return allowedTransitions ? allowedTransitions.includes(toStage) : false
  }

  /**
   * Get the next stage in progression
   */
  getNextStage(boardType: string, currentStage: string): string | undefined {
    const config = this.boardConfigs.get(boardType)
    if (!config) return undefined
    
    const currentIndex = config.stages.findIndex((s: StageDefinition) => s.key === currentStage)
    if (currentIndex === -1 || currentIndex >= config.stages.length - 1) {
      return undefined
    }
    
    // Simply return the next stage in order
    // The backend already defines the valid progression order
    return config.stages[currentIndex + 1].key
  }

  /**
   * Calculate stage completion status
   */
  calculateStageCompletion(
    stage: StageDefinition,
    completedDocuments: Set<string>
  ): {
    completed: number
    total: number
    percentage: number
    missingDocuments: string[]
    isComplete: boolean
  } {
    const requiredDocs = stage.requiredDocuments
    const completed = requiredDocs.filter(docId => completedDocuments.has(docId))
    const missing = requiredDocs.filter(docId => !completedDocuments.has(docId))
    
    return {
      completed: completed.length,
      total: requiredDocs.length,
      percentage: requiredDocs.length > 0 
        ? Math.round((completed.length / requiredDocs.length) * 100)
        : 0,
      missingDocuments: missing,
      isComplete: missing.length === 0 && requiredDocs.length > 0
    }
  }

  /**
   * Format a document ID for display
   */
  private formatDocumentId(docId: string): string {
    return docId
      .replace(/_/g, ' ')
      .replace(/\b\w/g, l => l.toUpperCase())
  }
}

// Export a singleton instance
export const boardConfigService = new BoardConfigurationService()