// Centralized domain types to eliminate duplication across the codebase

// Document types - used in modules, campaigns, and board views
export interface Document {
  id: string | number
  title: string
  content?: string
  phase?: 'draft' | 'review' | 'published' | 'archived'
  documentType?: 'vision' | 'strategy' | 'initiative' | 'task' | 'adr'
  parentId?: string
  createdAt?: Date
  updatedAt?: Date
  blockedBy?: string[]
  exitCriteria?: ExitCriterion[]
  // Additional fields for compatibility with different components
  campaign_id?: number
  module_id?: number | null
  template_id?: string
  document_type?: string
  parent_id?: string
  file_path?: string
  file_type?: string
  is_user_created?: boolean
  created_at?: string
  updated_at?: string
  completed_at?: string | null
  blocked_by?: string[]
  exit_criteria?: ExitCriterion[]
}

export interface ExitCriterion {
  id: string
  title: string
  completed: boolean
  notes?: string
}

// Board configuration types - used in multiple board views
export interface BoardConfig {
  columns: BoardColumn[]
  defaultColumn?: string
  transitions?: BoardTransition[]
  documentTypes?: BoardDocumentType[]
  document_types?: BoardDocumentType[]
  stages?: BoardStage[]
}

export interface BoardStage {
  key: string
  displayName?: string
  display_name?: string
  title?: string
  required_documents?: string[]
  optional_documents?: string[]
  no_completion_required_documents?: string[]
  templates?: DocumentTemplate[]
  transition_prompt?: string
}

export interface BoardColumn {
  id: string
  title: string
  color?: string
  order: number
  phases?: string[]
}

export interface BoardTransition {
  from: string
  to: string
  label?: string
  requiresValidation?: boolean
  requires_validation?: boolean
}

export interface BoardDocumentType {
  type: string
  columns: string[]
  defaultColumn?: string
  default_column?: string
}

// Module types
export interface Module {
  id: number
  campaign_id: number
  name: string
  module_number: number
  description?: string
  expected_sessions: number
  completed_sessions: number
  actual_sessions?: number
  module_type: 'standard' | 'oneshot' | 'campaign'
  status?: string
  phase: 'planning' | 'development' | 'ready' | 'active' | 'completed'
  documents?: Document[]
  created_at: string
  updated_at: string
}

// Campaign types
export interface Campaign {
  id: number
  name: string
  description?: string
  system: string
  phase: 'planning' | 'active' | 'completed' | 'archived'
  documents?: Document[]
  modules?: Module[]
  created_at: string
  updated_at: string
}

// Stage types for module/campaign progression
export type Stage = 'planning' | 'development' | 'ready' | 'active' | 'completed'

export interface StageInfo {
  title: string
  subtitle: string
  color: string
  phase: string
  description?: string
}

// Template types for document creation
export interface DocumentTemplate {
  templateId: string
  title: string
  description?: string
  documentType: string
  required?: boolean
  defaultContent?: string
}