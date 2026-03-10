/**
 * Tests for ModuleService
 *
 * Tests the module service layer that wraps Tauri invoke commands
 * for module CRUD operations.
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
  mockCommandError,
  expectCommandCalled,
  expectCommandCalledWith,
} from '@tests/helpers/mockInvoke'
import { ModuleService } from '@/services/ModuleService'

// ─── Factories ──────────────────────────────────────────────────────────────

function makeModule(overrides: Record<string, unknown> = {}) {
  return {
    id: 'mod-1',
    campaign_id: 'camp-1',
    name: 'The Goblin Hideout',
    description: 'A goblin-infested cave',
    module_number: 1,
    module_type: 'dungeon',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    ...overrides,
  }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('ModuleService', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  describe('list', () => {
    it('returns modules for a campaign', async () => {
      const modules = [makeModule(), makeModule({ id: 'mod-2', name: 'Module 2' })]
      mockCommand('list_modules', modules)

      const result = await ModuleService.list('camp-1')
      expect(result).toHaveLength(2)
      expect(result[0].name).toBe('The Goblin Hideout')
      expectCommandCalledWith('list_modules', { campaignId: 'camp-1' })
    })

    it('throws on error response', async () => {
      mockCommandError('list_modules', 'Database error')
      await expect(ModuleService.list('camp-1')).rejects.toThrow('Database error')
    })
  })

  describe('get', () => {
    it('returns a single module', async () => {
      mockCommand('get_module', makeModule())

      const result = await ModuleService.get('mod-1')
      expect(result.name).toBe('The Goblin Hideout')
      expectCommandCalledWith('get_module', { id: 'mod-1' })
    })

    it('throws on error response', async () => {
      mockCommandError('get_module', 'Not found')
      await expect(ModuleService.get('mod-1')).rejects.toThrow('Not found')
    })
  })

  describe('create', () => {
    it('creates a module and returns it', async () => {
      mockCommand('create_module', makeModule())

      const result = await ModuleService.create({
        campaign_id: 'camp-1',
        name: 'The Goblin Hideout',
        module_type: 'dungeon',
      })

      expect(result.name).toBe('The Goblin Hideout')
      expectCommandCalledWith('create_module', {
        request: {
          campaign_id: 'camp-1',
          name: 'The Goblin Hideout',
          module_type: 'dungeon',
        },
      })
    })

    it('throws on error response', async () => {
      mockCommandError('create_module', 'Duplicate name')
      await expect(
        ModuleService.create({ campaign_id: 'camp-1', name: 'Test' }),
      ).rejects.toThrow('Duplicate name')
    })
  })

  describe('update', () => {
    it('updates a module and returns it', async () => {
      mockCommand('update_module', makeModule({ name: 'Updated Name' }))

      const result = await ModuleService.update('mod-1', { name: 'Updated Name' })
      expect(result.name).toBe('Updated Name')
      expectCommandCalledWith('update_module', {
        id: 'mod-1',
        request: { name: 'Updated Name' },
      })
    })

    it('throws on error response', async () => {
      mockCommandError('update_module', 'Validation failed')
      await expect(
        ModuleService.update('mod-1', { name: '' }),
      ).rejects.toThrow('Validation failed')
    })
  })

  describe('delete', () => {
    it('deletes a module', async () => {
      mockCommand('delete_module', null)

      await ModuleService.delete('mod-1', 'camp-1')
      expectCommandCalledWith('delete_module', { id: 'mod-1' })
    })

    it('throws on error response', async () => {
      mockCommandError('delete_module', 'Module has active references')
      await expect(ModuleService.delete('mod-1')).rejects.toThrow('Module has active references')
    })
  })

  describe('reorder', () => {
    it('reorders a module to new position', async () => {
      const reordered = [
        makeModule({ id: 'mod-2', module_number: 1 }),
        makeModule({ id: 'mod-1', module_number: 2 }),
      ]
      mockCommand('reorder_module', reordered)

      const result = await ModuleService.reorder('mod-1', 2)
      expect(result).toHaveLength(2)
      expectCommandCalledWith('reorder_module', {
        moduleId: 'mod-1',
        newPosition: 2,
      })
    })

    it('throws on error response', async () => {
      mockCommandError('reorder_module', 'Invalid position')
      await expect(ModuleService.reorder('mod-1', 99)).rejects.toThrow('Invalid position')
    })
  })
})
