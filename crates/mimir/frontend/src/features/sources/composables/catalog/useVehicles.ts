import { ref, type Ref } from 'vue'
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
}

export interface VehicleFilters {
  query?: string
  sources?: string[]
  vehicle_type?: string
}

export function useVehicles() {
  const isVehiclesInitialized = ref(true)
  const isLoading = ref(false)
  const error: Ref<string | null> = ref(null)
  const vehicles = ref<VehicleSummary[]>([])

  async function initializeVehicleCatalog() {
    // No initialization needed for DB-backed catalog
  }

  async function searchVehicles(filters: VehicleFilters = {}): Promise<VehicleSummary[]> {
    try {
      isLoading.value = true
      error.value = null

      // Transform to backend VehicleFilter format
      const backendFilter = {
        name_contains: filters.query || null,
        sources: filters.sources ?? null,
        vehicle_type: filters.vehicle_type || null,
      }

      const response = await invoke<{ success: boolean; data?: VehicleSummary[]; error?: string }>('search_vehicles', {
        filter: backendFilter,
        limit: 10000,
        offset: 0
      })

      if (response.success && response.data) {
        vehicles.value = response.data
        return response.data
      } else {
        error.value = response.error || 'Search failed'
        return []
      }
    } catch (e) {
      error.value = `Search failed: ${e}`
      return []
    } finally {
      isLoading.value = false
    }
  }

  async function getVehicleDetails(name: string, source: string): Promise<Vehicle | null> {
    try {
      const response = await invoke<{ success: boolean; data?: Vehicle; error?: string }>('get_vehicle_by_name', {
        name,
        source
      })
      if (response.success && response.data) {
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to get vehicle details:', e)
      return null
    }
  }

  return {
    isVehiclesInitialized,
    isLoading,
    error,
    vehicles,
    initializeVehicleCatalog,
    searchVehicles,
    getVehicleDetails,
  }
}
