import { vi } from 'vitest'

export function mockInvoke(responses: Record<string, any>) {
  const invoke = vi.fn(async (command: string, args?: any) => {
    if (command in responses) {
      const response = responses[command]
      if (typeof response === 'function') {
        return response(args)
      }
      return response
    }
    throw new Error(`Unmocked command: ${command}`)
  })
  
  return invoke
}

export function createMockModule(overrides: any = {}) {
  return {
    id: 1,
    name: 'Test Module',
    campaign_id: 1,
    module_type: 'standard',
    status: 'planning',
    session_count: 0,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    module_number: 1,
    expected_sessions: 4,
    actual_sessions: 0,
    sessions_planned: 0,
    sessions_completed: 0,
    started_at: null,
    completed_at: null,
    ...overrides
  }
}

export function createMockDocument(overrides: any = {}) {
  return {
    id: 1,
    title: 'Test Document',
    content: 'Test content',
    campaign_id: 1,
    module_id: null,
    session_id: null,
    template_id: 'test-template',
    document_type: 'note',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    completed_at: null,
    ...overrides
  }
}

export function createMockSession(overrides: any = {}) {
  return {
    id: 1,
    module_id: 1,
    name: 'Test Session',
    status: 'planned',
    session_number: 1,
    created_at: '2024-01-01T00:00:00Z',
    ...overrides
  }
}

export function createMockCampaign(overrides: any = {}) {
  return {
    id: 1,
    name: 'Test Campaign',
    description: 'Test Description',
    system: 'D&D 5e',
    status: 'active',
    directory_path: '/test/path',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    ...overrides
  }
}