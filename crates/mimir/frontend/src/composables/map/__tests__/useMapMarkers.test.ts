/**
 * Tests for useMapMarkers composable.
 *
 * Tests trap/POI state management, visibility toggling, context menu operations,
 * icon mapping, and marker clearing.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'
import { useMapMarkers, type MapTrap, type MapPoi } from '../useMapMarkers'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  emit: vi.fn(),
}))

const mockInvoke = vi.mocked(invoke)

function makeTrap(overrides: Partial<MapTrap> = {}): MapTrap {
  return {
    id: 'trap-1',
    map_id: 'map-1',
    grid_x: 5,
    grid_y: 3,
    name: 'Pit Trap',
    description: 'A hidden pit',
    trigger_description: 'Pressure plate',
    effect_description: '2d6 falling damage',
    dc: 15,
    visible: 0,
    created_at: '2024-01-01',
    updated_at: '2024-01-01',
    ...overrides,
  }
}

function makePoi(overrides: Partial<MapPoi> = {}): MapPoi {
  return {
    id: 'poi-1',
    map_id: 'map-1',
    grid_x: 7,
    grid_y: 2,
    name: 'Treasure Chest',
    description: 'A locked chest',
    icon: 'chest',
    color: '#gold',
    visible: 1,
    created_at: '2024-01-01',
    updated_at: '2024-01-01',
    ...overrides,
  }
}

function createMarkers(mapIdValue = 'map-1') {
  return useMapMarkers({
    mapId: computed(() => mapIdValue),
    gridSizePx: computed(() => 70),
    isDisplayOpen: ref(false),
  })
}

describe('useMapMarkers', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('starts with empty traps and POIs', () => {
      const markers = createMarkers()
      expect(markers.mapTraps.value).toEqual([])
      expect(markers.mapPois.value).toEqual([])
    })

    it('starts with no selections', () => {
      const markers = createMarkers()
      expect(markers.selectedTrapId.value).toBeNull()
      expect(markers.selectedPoiId.value).toBeNull()
    })

    it('starts with context menu hidden', () => {
      const markers = createMarkers()
      expect(markers.poiContextMenu.value.visible).toBe(false)
    })
  })

  describe('loadMapTraps', () => {
    it('loads traps from backend', async () => {
      const traps = [makeTrap(), makeTrap({ id: 'trap-2', name: 'Arrow Trap' })]
      mockInvoke.mockResolvedValueOnce({ success: true, data: traps })

      const markers = createMarkers()
      await markers.loadMapTraps()

      expect(markers.mapTraps.value).toHaveLength(2)
      expect(mockInvoke).toHaveBeenCalledWith('list_map_traps', { mapId: 'map-1' })
    })

    it('clears traps on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network error'))

      const markers = createMarkers()
      markers.mapTraps.value = [makeTrap()]
      await markers.loadMapTraps()

      expect(markers.mapTraps.value).toEqual([])
    })

    it('accepts explicit map ID', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true, data: [] })

      const markers = createMarkers()
      await markers.loadMapTraps('other-map')

      expect(mockInvoke).toHaveBeenCalledWith('list_map_traps', { mapId: 'other-map' })
    })
  })

  describe('loadMapPois', () => {
    it('loads POIs from backend', async () => {
      const pois = [makePoi()]
      mockInvoke.mockResolvedValueOnce({ success: true, data: pois })

      const markers = createMarkers()
      await markers.loadMapPois()

      expect(markers.mapPois.value).toHaveLength(1)
    })
  })

  describe('sendMarkersToDisplay', () => {
    it('does nothing when display is closed', async () => {
      const markers = createMarkers()
      await markers.sendMarkersToDisplay()

      expect(emit).not.toHaveBeenCalled()
    })

    it('sends only visible markers when display is open', async () => {
      const markers = useMapMarkers({
        mapId: computed(() => 'map-1'),
        gridSizePx: computed(() => 70),
        isDisplayOpen: ref(true),
      })

      markers.mapTraps.value = [
        makeTrap({ id: 't1', visible: 1 }),
        makeTrap({ id: 't2', visible: 0 }),
      ]
      markers.mapPois.value = [
        makePoi({ id: 'p1', visible: 1 }),
        makePoi({ id: 'p2', visible: 0 }),
      ]

      await markers.sendMarkersToDisplay()

      expect(emit).toHaveBeenCalledWith('player-display:markers-update', expect.objectContaining({
        mapId: 'map-1',
        gridSizePx: 70,
      }))

      const call = vi.mocked(emit).mock.calls[0]
      const payload = call[1] as any
      expect(payload.traps).toHaveLength(1)
      expect(payload.traps[0].id).toBe('t1')
      expect(payload.pois).toHaveLength(1)
      expect(payload.pois[0].id).toBe('p1')
    })
  })

  describe('clearMarkers', () => {
    it('clears all marker state', () => {
      const markers = createMarkers()
      markers.mapTraps.value = [makeTrap()]
      markers.mapPois.value = [makePoi()]
      markers.selectedTrapId.value = 'trap-1'
      markers.selectedPoiId.value = 'poi-1'

      markers.clearMarkers()

      expect(markers.mapTraps.value).toEqual([])
      expect(markers.mapPois.value).toEqual([])
      expect(markers.selectedTrapId.value).toBeNull()
      expect(markers.selectedPoiId.value).toBeNull()
    })
  })

  describe('POI context menu', () => {
    it('showPoiContextMenuAt sets context menu state', () => {
      const markers = createMarkers()
      const poi = makePoi()
      const event = { clientX: 200, clientY: 300 } as MouseEvent

      markers.showPoiContextMenuAt(event, poi)

      expect(markers.poiContextMenu.value.visible).toBe(true)
      expect(markers.poiContextMenu.value.x).toBe(200)
      expect(markers.poiContextMenu.value.y).toBe(300)
      expect(markers.poiContextMenu.value.poi).toEqual(poi)
      expect(markers.selectedPoiId.value).toBe('poi-1')
    })

    it('closePoiContextMenu hides menu', () => {
      const markers = createMarkers()
      markers.poiContextMenu.value.visible = true

      markers.closePoiContextMenu()

      expect(markers.poiContextMenu.value.visible).toBe(false)
    })

    it('openPoiEditModal sets edit state from context menu', () => {
      const markers = createMarkers()
      const poi = makePoi()
      markers.poiContextMenu.value = { visible: true, x: 0, y: 0, poi }

      markers.openPoiEditModal()

      expect(markers.showPoiEditModal.value).toBe(true)
      expect(markers.poiToEdit.value).toEqual(poi)
      expect(markers.poiContextMenu.value.visible).toBe(false)
    })

    it('closePoiEditModal clears edit state', () => {
      const markers = createMarkers()
      markers.showPoiEditModal.value = true
      markers.poiToEdit.value = makePoi()

      markers.closePoiEditModal()

      expect(markers.showPoiEditModal.value).toBe(false)
      expect(markers.poiToEdit.value).toBeNull()
    })
  })

  describe('handlePoiSaved', () => {
    it('updates POI in local list', () => {
      const markers = createMarkers()
      const original = makePoi({ id: 'poi-1', name: 'Old Name' })
      markers.mapPois.value = [original]

      const updated = makePoi({ id: 'poi-1', name: 'New Name' })
      markers.handlePoiSaved(updated)

      expect(markers.mapPois.value[0].name).toBe('New Name')
      expect(markers.showPoiEditModal.value).toBe(false)
    })
  })

  describe('getPoiIcon', () => {
    it('maps known icon names to emoji', () => {
      const markers = createMarkers()
      expect(markers.getPoiIcon('chest')).toBe('\uD83D\uDCE6')
      expect(markers.getPoiIcon('skull')).toBe('\uD83D\uDC80')
      expect(markers.getPoiIcon('door')).toBe('\uD83D\uDEAA')
      expect(markers.getPoiIcon('star')).toBe('\u2B50')
    })

    it('returns default pin for unknown icons', () => {
      const markers = createMarkers()
      expect(markers.getPoiIcon('unknown')).toBe('\uD83D\uDCCD')
    })
  })
})
