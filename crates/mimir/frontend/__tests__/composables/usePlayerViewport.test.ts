/**
 * Tests for usePlayerViewport composable.
 *
 * Tests pan/zoom state management, transform computation,
 * mouse interaction handlers, zoom clamping, and reset behavior.
 * This composable is pure logic with no Tauri dependencies.
 */

import { describe, it, expect } from 'vitest'
import { usePlayerViewport } from '@/composables/map/usePlayerViewport'

describe('usePlayerViewport', () => {
  describe('initial state', () => {
    it('starts with zero pan and 1x zoom', () => {
      const viewport = usePlayerViewport()
      expect(viewport.panX.value).toBe(0)
      expect(viewport.panY.value).toBe(0)
      expect(viewport.zoom.value).toBe(1)
    })

    it('starts with panning disabled', () => {
      const viewport = usePlayerViewport()
      expect(viewport.isPanning.value).toBe(false)
    })

    it('starts with display scale of 1', () => {
      const viewport = usePlayerViewport()
      expect(viewport.displayScale.value).toBe(1)
    })

    it('starts with zero image dimensions', () => {
      const viewport = usePlayerViewport()
      expect(viewport.imageNaturalWidth.value).toBe(0)
      expect(viewport.imageNaturalHeight.value).toBe(0)
    })
  })

  describe('transform computation', () => {
    it('produces identity-like transform at defaults', () => {
      const viewport = usePlayerViewport()
      const t = viewport.transform.value
      expect(t.transform).toBe('translate(0px, 0px) scale(1)')
      expect(t.transformOrigin).toBe('center center')
    })

    it('reflects pan offset in transform', () => {
      const viewport = usePlayerViewport()
      viewport.panX.value = 50
      viewport.panY.value = -30
      const t = viewport.transform.value
      expect(t.transform).toBe('translate(50px, -30px) scale(1)')
    })

    it('reflects zoom in transform', () => {
      const viewport = usePlayerViewport()
      viewport.zoom.value = 2
      const t = viewport.transform.value
      expect(t.transform).toBe('translate(0px, 0px) scale(2)')
    })

    it('combines displayScale and zoom in transform', () => {
      const viewport = usePlayerViewport()
      viewport.displayScale.value = 0.5
      viewport.zoom.value = 2
      const t = viewport.transform.value
      // finalScale = displayScale * zoom = 0.5 * 2 = 1
      expect(t.transform).toBe('translate(0px, 0px) scale(1)')
    })

    it('combines pan, displayScale, and zoom', () => {
      const viewport = usePlayerViewport()
      viewport.panX.value = 100
      viewport.panY.value = -50
      viewport.displayScale.value = 0.75
      viewport.zoom.value = 1.5
      const t = viewport.transform.value
      const expectedScale = 0.75 * 1.5 // 1.125
      expect(t.transform).toBe(`translate(100px, -50px) scale(${expectedScale})`)
    })
  })

  describe('panning', () => {
    it('mousedown starts panning on left button', () => {
      const viewport = usePlayerViewport()
      const event = {
        button: 0,
        clientX: 100,
        clientY: 200,
        preventDefault: () => {},
      } as MouseEvent
      viewport.handleMouseDown(event)
      expect(viewport.isPanning.value).toBe(true)
    })

    it('mousedown on non-left button does not start panning', () => {
      const viewport = usePlayerViewport()
      const event = {
        button: 2, // right click
        clientX: 100,
        clientY: 200,
        preventDefault: () => {},
      } as MouseEvent
      viewport.handleMouseDown(event)
      expect(viewport.isPanning.value).toBe(false)
    })

    it('mousemove updates pan when panning', () => {
      const viewport = usePlayerViewport()

      // Start panning at (100, 200)
      viewport.handleMouseDown({
        button: 0,
        clientX: 100,
        clientY: 200,
        preventDefault: () => {},
      } as MouseEvent)

      // Move to (150, 230) — delta is (50, 30)
      viewport.handleMouseMove({
        clientX: 150,
        clientY: 230,
      } as MouseEvent)

      expect(viewport.panX.value).toBe(50)
      expect(viewport.panY.value).toBe(30)
    })

    it('mousemove does nothing when not panning', () => {
      const viewport = usePlayerViewport()
      viewport.handleMouseMove({
        clientX: 150,
        clientY: 230,
      } as MouseEvent)
      expect(viewport.panX.value).toBe(0)
      expect(viewport.panY.value).toBe(0)
    })

    it('mouseup stops panning', () => {
      const viewport = usePlayerViewport()
      viewport.handleMouseDown({
        button: 0,
        clientX: 100,
        clientY: 200,
        preventDefault: () => {},
      } as MouseEvent)
      expect(viewport.isPanning.value).toBe(true)
      viewport.handleMouseUp()
      expect(viewport.isPanning.value).toBe(false)
    })
  })

  describe('zooming', () => {
    it('scroll down decreases zoom', () => {
      const viewport = usePlayerViewport()
      viewport.handleWheel({
        deltaY: 100, // scroll down
        clientX: 0,
        clientY: 0,
        currentTarget: null,
      } as unknown as WheelEvent)
      expect(viewport.zoom.value).toBeLessThan(1)
    })

    it('scroll up increases zoom', () => {
      const viewport = usePlayerViewport()
      viewport.handleWheel({
        deltaY: -100, // scroll up
        clientX: 0,
        clientY: 0,
        currentTarget: null,
      } as unknown as WheelEvent)
      expect(viewport.zoom.value).toBeGreaterThan(1)
    })

    it('clamps zoom to minimum', () => {
      const viewport = usePlayerViewport({ minZoom: 0.5 })
      // Zoom way out
      for (let i = 0; i < 100; i++) {
        viewport.handleWheel({
          deltaY: 100,
          clientX: 0,
          clientY: 0,
          currentTarget: null,
        } as unknown as WheelEvent)
      }
      expect(viewport.zoom.value).toBeGreaterThanOrEqual(0.5)
    })

    it('clamps zoom to maximum', () => {
      const viewport = usePlayerViewport({ maxZoom: 3 })
      // Zoom way in
      for (let i = 0; i < 100; i++) {
        viewport.handleWheel({
          deltaY: -100,
          clientX: 0,
          clientY: 0,
          currentTarget: null,
        } as unknown as WheelEvent)
      }
      expect(viewport.zoom.value).toBeLessThanOrEqual(3)
    })

    it('uses custom zoom step', () => {
      const viewport = usePlayerViewport({ zoomStep: 0.5 })
      const initialZoom = viewport.zoom.value
      viewport.handleWheel({
        deltaY: -100,
        clientX: 0,
        clientY: 0,
        currentTarget: null,
      } as unknown as WheelEvent)
      // With step 0.5, zoom increases by factor of 1.5
      expect(viewport.zoom.value).toBeCloseTo(initialZoom * 1.5, 5)
    })
  })

  describe('reset', () => {
    it('resets pan and zoom to defaults', () => {
      const viewport = usePlayerViewport()
      viewport.panX.value = 100
      viewport.panY.value = -50
      viewport.zoom.value = 2.5

      viewport.reset()

      expect(viewport.panX.value).toBe(0)
      expect(viewport.panY.value).toBe(0)
      expect(viewport.zoom.value).toBe(1)
    })

    it('does not reset displayScale', () => {
      const viewport = usePlayerViewport()
      viewport.displayScale.value = 0.75
      viewport.reset()
      // displayScale is set by updateDisplayScale, not reset
      expect(viewport.displayScale.value).toBe(0.75)
    })
  })

  describe('updateDisplayScale', () => {
    it('does nothing with null image', () => {
      const viewport = usePlayerViewport()
      viewport.updateDisplayScale(null)
      expect(viewport.displayScale.value).toBe(1)
    })

    it('does nothing with zero-dimension image', () => {
      const viewport = usePlayerViewport()
      const img = { naturalWidth: 0, naturalHeight: 0 } as HTMLImageElement
      viewport.updateDisplayScale(img)
      expect(viewport.displayScale.value).toBe(1)
    })

    it('stores natural dimensions', () => {
      const viewport = usePlayerViewport()
      const img = { naturalWidth: 1920, naturalHeight: 1080 } as HTMLImageElement
      viewport.updateDisplayScale(img)
      expect(viewport.imageNaturalWidth.value).toBe(1920)
      expect(viewport.imageNaturalHeight.value).toBe(1080)
    })
  })

  describe('custom options', () => {
    it('uses default options when none provided', () => {
      const viewport = usePlayerViewport()
      // Default minZoom = 0.25 — zoom way out
      for (let i = 0; i < 200; i++) {
        viewport.handleWheel({
          deltaY: 100,
          clientX: 0,
          clientY: 0,
          currentTarget: null,
        } as unknown as WheelEvent)
      }
      expect(viewport.zoom.value).toBeGreaterThanOrEqual(0.25)
      expect(viewport.zoom.value).toBeLessThan(0.3)
    })

    it('accepts all options together', () => {
      const viewport = usePlayerViewport({
        minZoom: 0.1,
        maxZoom: 10,
        zoomStep: 0.2,
      })
      // Just verify it creates without error and defaults are overridden
      expect(viewport.zoom.value).toBe(1)
    })
  })
})
