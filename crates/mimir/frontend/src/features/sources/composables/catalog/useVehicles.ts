import { invoke } from '@tauri-apps/api/core'

export interface Vehicle {
  name: string
  source: string
  vehicle_type?: string
  size?: string
  page?: number
  cap_crew?: number
  cap_passenger?: number
  cap_cargo?: number
  ac?: number
  hp?: number
  speed?: any
  pace?: number
  dimensions?: string[]
  immune?: string[]
  resist?: string[]
  vulnerable?: string[]
  terrain?: string[]
  weapon?: any[]
  entries?: any[]
}

export interface VehicleSummary {
  name: string
  source: string
  vehicle_type?: string
  size?: string
  cap_crew?: number
  cap_passenger?: number
  capacity: string
  terrain?: string[]
  pace?: number
  speed?: string
}

export interface VehicleFilters {
  query?: string
  types?: string[]
  sources?: string[]
  terrains?: string[]
  sizes?: string[]
}

export function useVehicles() {
  async function initializeVehicleCatalog() {
    // No-op: database-backed vehicle catalog doesn't need initialization
  }

  async function searchVehicles(filters: VehicleFilters = {}): Promise<VehicleSummary[]> {
    try {
      const results = await invoke<VehicleSummary[]>('search_vehicles', {
        filters: {
          name: filters.query || null,
          sources: filters.sources || null,
          vehicle_types: filters.types || null,
          terrains: filters.terrains || null,
          sizes: filters.sizes || null
        }
      })
      return results || []
    } catch (e) {
      return []
    }
  }

  async function getVehicleDetails(name: string, source: string): Promise<Vehicle | null> {
    try {
      const details = await invoke<Vehicle>('get_vehicle_details', { vehicleName: name, vehicleSource: source })
      return details
    } catch (e) {
      return null
    }
  }

  async function getVehicleTypes(): Promise<string[]> {
    try {
      const types = await invoke<string[]>('get_vehicle_types')
      return types || []
    } catch (e) {
      return []
    }
  }

  async function getVehicleTerrains(): Promise<string[]> {
    try {
      const terrains = await invoke<string[]>('get_vehicle_terrains')
      return terrains || []
    } catch (e) {
      return []
    }
  }

  async function getVehicleSizes(): Promise<string[]> {
    try {
      const sizes = await invoke<string[]>('get_vehicle_sizes')
      return sizes || []
    } catch (e) {
      return []
    }
  }

  async function getVehicleSources(): Promise<string[]> {
    try {
      const sources = await invoke<[string, number][]>('get_vehicle_statistics')
      return (sources || []).map(([source, _count]) => source)
    } catch (e) {
      return []
    }
  }

  return {
    initializeVehicleCatalog,
    searchVehicles,
    getVehicleDetails,
    getVehicleTypes,
    getVehicleTerrains,
    getVehicleSizes,
    getVehicleSources,
  }
}
