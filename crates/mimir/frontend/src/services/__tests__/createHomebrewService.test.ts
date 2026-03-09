import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createHomebrewService } from '../createHomebrewService'
import type { HomebrewBase } from '../createHomebrewService'
import { dataEvents } from '@/utils/dataEvents'

vi.mock('@tauri-apps/api/core')

interface TestMonster extends HomebrewBase {
  cr: string | null
  creature_type: string | null
  size: string | null
}

interface CreateTestMonster {
  campaign_id: string
  name: string
  data: string
  cr?: string
}

interface UpdateTestMonster {
  name?: string
  data?: string
  cr?: string | null
}

const mockMonster: TestMonster = {
  id: 'test-id-1',
  campaign_id: 'campaign-1',
  name: 'Frost Colossus',
  data: '{"name":"Frost Colossus"}',
  cr: '20',
  creature_type: 'elemental',
  size: 'G',
  cloned_from_name: null,
  cloned_from_source: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
}

describe('createHomebrewService', () => {
  let invoke: ReturnType<typeof vi.fn>
  let service: ReturnType<typeof createHomebrewService<TestMonster, CreateTestMonster, UpdateTestMonster>>

  beforeEach(async () => {
    vi.clearAllMocks()
    dataEvents.clear()

    const { invoke: mockInvoke } = await import('@tauri-apps/api/core')
    invoke = mockInvoke as any

    service = createHomebrewService<TestMonster, CreateTestMonster, UpdateTestMonster>({
      commandSuffix: 'monster',
      eventPrefix: 'homebrew-monster',
      label: 'monster',
    })
  })

  describe('list', () => {
    it('should call list_homebrew_monsters and return data', async () => {
      invoke.mockResolvedValueOnce({ success: true, data: [mockMonster] })

      const result = await service.list('campaign-1')

      expect(invoke).toHaveBeenCalledWith('list_homebrew_monsters', { campaignId: 'campaign-1' })
      expect(result).toEqual([mockMonster])
    })

    it('should return empty array when no items', async () => {
      invoke.mockResolvedValueOnce({ success: true, data: [] })

      const result = await service.list('campaign-1')
      expect(result).toEqual([])
    })

    it('should throw on error response', async () => {
      invoke.mockResolvedValueOnce({ success: false, error: 'DB error' })

      await expect(service.list('campaign-1')).rejects.toThrow('DB error')
    })

    it('should throw default message when no error provided', async () => {
      invoke.mockResolvedValueOnce({ success: false })

      await expect(service.list('campaign-1')).rejects.toThrow('Failed to list homebrew monsters')
    })
  })

  describe('get', () => {
    it('should call get_homebrew_monster and return data', async () => {
      invoke.mockResolvedValueOnce({ success: true, data: mockMonster })

      const result = await service.get('test-id-1')

      expect(invoke).toHaveBeenCalledWith('get_homebrew_monster', { id: 'test-id-1' })
      expect(result).toEqual(mockMonster)
    })

    it('should throw on error response', async () => {
      invoke.mockResolvedValueOnce({ success: false, error: 'Not found' })

      await expect(service.get('bad-id')).rejects.toThrow('Not found')
    })
  })

  describe('create', () => {
    it('should call create_homebrew_monster and return created entity', async () => {
      invoke.mockResolvedValueOnce({ success: true, data: mockMonster })

      const input: CreateTestMonster = {
        campaign_id: 'campaign-1',
        name: 'Frost Colossus',
        data: '{}',
        cr: '20',
      }
      const result = await service.create(input)

      expect(invoke).toHaveBeenCalledWith('create_homebrew_monster', { input })
      expect(result).toEqual(mockMonster)
    })

    it('should emit created event on success', async () => {
      invoke.mockResolvedValueOnce({ success: true, data: mockMonster })

      const listener = vi.fn()
      dataEvents.on('homebrew-monster:created', listener)

      await service.create({
        campaign_id: 'campaign-1',
        name: 'Test',
        data: '{}',
      })

      expect(listener).toHaveBeenCalledTimes(1)
    })

    it('should throw on error response without emitting', async () => {
      invoke.mockResolvedValueOnce({ success: false, error: 'Invalid data' })

      const listener = vi.fn()
      dataEvents.on('homebrew-monster:created', listener)

      await expect(
        service.create({ campaign_id: 'c', name: 'x', data: '{}' })
      ).rejects.toThrow('Invalid data')

      expect(listener).not.toHaveBeenCalled()
    })
  })

  describe('update', () => {
    it('should call update_homebrew_monster and return updated entity', async () => {
      const updated = { ...mockMonster, name: 'Ice Titan' }
      invoke.mockResolvedValueOnce({ success: true, data: updated })

      const result = await service.update('test-id-1', { name: 'Ice Titan' })

      expect(invoke).toHaveBeenCalledWith('update_homebrew_monster', {
        id: 'test-id-1',
        input: { name: 'Ice Titan' },
      })
      expect(result.name).toBe('Ice Titan')
    })

    it('should emit updated event on success', async () => {
      invoke.mockResolvedValueOnce({ success: true, data: mockMonster })

      const listener = vi.fn()
      dataEvents.on('homebrew-monster:updated', listener)

      await service.update('test-id-1', { name: 'Updated' })

      expect(listener).toHaveBeenCalledTimes(1)
    })

    it('should throw on error response', async () => {
      invoke.mockResolvedValueOnce({ success: false, error: 'Not found' })

      await expect(service.update('bad-id', { name: 'x' })).rejects.toThrow('Not found')
    })
  })

  describe('delete', () => {
    it('should call delete_homebrew_monster', async () => {
      invoke.mockResolvedValueOnce({ success: true, data: true })

      await service.delete('test-id-1')

      expect(invoke).toHaveBeenCalledWith('delete_homebrew_monster', { id: 'test-id-1' })
    })

    it('should emit deleted event on success', async () => {
      invoke.mockResolvedValueOnce({ success: true, data: true })

      const listener = vi.fn()
      dataEvents.on('homebrew-monster:deleted', listener)

      await service.delete('test-id-1')

      expect(listener).toHaveBeenCalledTimes(1)
      expect(listener).toHaveBeenCalledWith({ id: 'test-id-1' })
    })

    it('should throw on error response', async () => {
      invoke.mockResolvedValueOnce({ success: false, error: 'Not found' })

      await expect(service.delete('bad-id')).rejects.toThrow('Not found')
    })
  })

  describe('command naming', () => {
    it('should use correct command suffix for items', async () => {
      const itemService = createHomebrewService({
        commandSuffix: 'item',
        eventPrefix: 'homebrew-item',
        label: 'item',
      })

      invoke.mockResolvedValueOnce({ success: true, data: [] })
      await itemService.list('campaign-1')
      expect(invoke).toHaveBeenCalledWith('list_homebrew_items', { campaignId: 'campaign-1' })
    })

    it('should use correct command suffix for spells', async () => {
      const spellService = createHomebrewService({
        commandSuffix: 'spell',
        eventPrefix: 'homebrew-spell',
        label: 'spell',
      })

      invoke.mockResolvedValueOnce({ success: true, data: [] })
      await spellService.list('campaign-1')
      expect(invoke).toHaveBeenCalledWith('list_homebrew_spells', { campaignId: 'campaign-1' })
    })
  })
})
