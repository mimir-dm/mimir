// Message-related types
export interface ChatMessage {
  id: string
  role: 'user' | 'assistant' | 'system' | 'tool'
  content: string
  timestamp: number
  tokenUsage?: {
    prompt: number
    completion: number
    total: number
  }
  isIntermediate?: boolean
  iteration?: number
  toolName?: string
  toolCalls?: string[]
  success?: boolean
  /** Required for tool result messages - identifies which tool call this responds to */
  tool_call_id?: string
}

export interface ChatResponseWithUsage {
  content: string
  prompt_tokens: number
  completion_tokens: number
  total_tokens: number
}

// Tool confirmation types
export interface ActionDescription {
  title: string
  description: string
  changes: ChangeDetail
}

export type ChangeDetail =
  | FileEditDetail
  | FileWriteDetail
  | FileReadDetail
  | GenericDetail

export interface FileEditDetail {
  type: 'FileEdit'
  file_path: string
  edits: LineEdit[]
  total_lines_affected: number
  total_lines_in_file: number
}

export interface FileWriteDetail {
  type: 'FileWrite'
  file_path: string
  content_length: number
  diff_preview?: DiffPreview
  content_preview?: string
}

export interface FileReadDetail {
  type: 'FileRead'
  file_path: string
  file_size: number
}

export interface GenericDetail {
  type: 'Generic'
  items: string[]
}

export interface LineEdit {
  operation: 'replace' | 'insert' | 'delete'
  start_line: number
  end_line: number
  old_content: string[]
  new_content: string[]
  context_before: string[]
  context_after: string[]
}

export interface DiffPreview {
  added_lines: number
  removed_lines: number
  preview: string
}

export interface ToolConfirmationRequest {
  id: string
  tool_name: string
  action: ActionDescription
}

export interface PendingConfirmation {
  request: ToolConfirmationRequest
  status: 'pending' | 'confirmed' | 'rejected'
  messageId?: string
}

// Event types
export interface IntermediateMessage {
  role: string
  content: string
  tool_calls: string[]
  iteration: number
  session_id?: string
}

export interface ToolResultMessage {
  tool_name: string
  result: string
  success: boolean
  iteration: number
  session_id?: string
  /** The ID of the tool call this result responds to */
  tool_call_id: string
}

// Todo types
export interface TodoItem {
  content: string
  status: 'pending' | 'in_progress' | 'completed'
  activeForm: string
}
