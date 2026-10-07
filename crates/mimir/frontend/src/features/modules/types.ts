/**
 * Module dashboard entities shown in the details inspector (MIMIR-T-0693).
 */

/** A trap type used on the module's maps. */
export interface ModuleTrap {
  id: string
  name: string
  /** Catalog source (e.g. "DMG"). */
  source: string
  /** How many of this trap type across all maps. */
  count: number
}

/** A point of interest on the module's maps. */
export interface ModulePoi {
  id: string
  name: string
  description: string | null
  icon: string
  color: string | null
  visible: number
  grid_x: number
  grid_y: number
  /** How many of this POI type across all maps. */
  count: number
}
