export const DEFAULT_GRID_SIZE_PX = 70

export interface UvttRoot {
  format: number
  resolution: {
    map_origin: { x: number; y: number }
    map_size: { x: number; y: number }
    pixels_per_grid: number
  }
  line_of_sight: Array<unknown>
  portals: Array<unknown>
  lights: Array<unknown>
  image: string
}

/**
 * Wrap a plain image into a minimal UVTT payload.
 * The returned value is base64(JSON), matching create_map's uvtt_data_base64.
 */
export function buildUvttBase64FromImage(
  imageBase64: string,
  widthPx: number,
  heightPx: number,
  gridSizePx = DEFAULT_GRID_SIZE_PX
): string {
  const width = Math.max(1, Math.floor(widthPx))
  const height = Math.max(1, Math.floor(heightPx))
  const ppg = Math.max(1, Math.floor(gridSizePx))

  const uvtt: UvttRoot = {
    format: 0.3,
    resolution: {
      map_origin: { x: 0, y: 0 },
      map_size: {
        x: Number((width / ppg).toFixed(4)),
        y: Number((height / ppg).toFixed(4))
      },
      pixels_per_grid: ppg
    },
    line_of_sight: [],
    portals: [],
    lights: [],
    image: imageBase64
  }

  return btoa(JSON.stringify(uvtt))
}

export function isUvttFilename(filename: string): boolean {
  const lower = filename.toLowerCase()
  return lower.endsWith('.dd2vtt') || lower.endsWith('.uvtt')
}

export function uvttFilenameForUpload(filename: string): string {
  if (isUvttFilename(filename)) return filename
  return filename.replace(/\.[^/.]+$/, '') + '.uvtt'
}
