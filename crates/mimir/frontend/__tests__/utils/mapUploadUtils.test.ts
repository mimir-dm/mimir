import { describe, expect, it } from 'vitest'
import {
  DEFAULT_GRID_SIZE_PX,
  buildUvttBase64FromImage,
  isUvttFilename,
  uvttFilenameForUpload
} from '@/features/campaigns/components/StageLanding/mapUploadUtils'

describe('mapUploadUtils', () => {
  it('wraps image base64 into valid UVTT JSON', () => {
    const imageBase64 = 'ZmFrZS1pbWFnZS1kYXRh'
    const encoded = buildUvttBase64FromImage(imageBase64, 1400, 700)
    const uvtt = JSON.parse(atob(encoded))

    expect(uvtt.image).toBe(imageBase64)
    expect(uvtt.resolution.pixels_per_grid).toBe(DEFAULT_GRID_SIZE_PX)
    expect(uvtt.resolution.map_size.x).toBe(20)
    expect(uvtt.resolution.map_size.y).toBe(10)
    expect(Array.isArray(uvtt.line_of_sight)).toBe(true)
    expect(Array.isArray(uvtt.portals)).toBe(true)
    expect(Array.isArray(uvtt.lights)).toBe(true)
  })

  it('detects UVTT filenames', () => {
    expect(isUvttFilename('map.uvtt')).toBe(true)
    expect(isUvttFilename('map.DD2VTT')).toBe(true)
    expect(isUvttFilename('map.png')).toBe(false)
  })

  it('normalizes image filename to uvtt for upload', () => {
    expect(uvttFilenameForUpload('battlemap.png')).toBe('battlemap.uvtt')
    expect(uvttFilenameForUpload('battlemap.uvtt')).toBe('battlemap.uvtt')
  })
})
