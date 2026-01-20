import type { CatalogConfig } from './types'

// Format vehicle size codes to full names
function formatSize(size: string | null): string {
  if (!size) return '—'
  
  // D&D 5e size abbreviations to full names
  const sizeMap: Record<string, string> = {
    'T': 'Tiny',
    'S': 'Small', 
    'M': 'Medium',
    'L': 'Large',
    'H': 'Huge',
    'G': 'Gargantuan'
  }
  
  return sizeMap[size] || size
}


// Format terrain array with capitalization
function formatTerrain(terrain: string[] | null): string {
  if (!terrain || terrain.length === 0) return '—'
  
  return terrain.map(t => 
    t.charAt(0).toUpperCase() + t.slice(1).toLowerCase()
  ).join(', ')
}

// Format vehicle type with custom badge colors
function formatVehicleType(vehicleType: string | null): { text: string; variant: string } {
  if (!vehicleType) return { text: '—', variant: 'default' }
  
  // Custom colors for different vehicle types
  const typeColors: Record<string, string> = {
    'ship': 'blue',
    'spelljammer': 'purple', 
    'infernal': 'red',
    'siege': 'orange',
    'creature': 'green',
    'mount': 'green',
    'vehicle': 'gray'
  }
  
  const variant = typeColors[vehicleType.toLowerCase()] || 'default'
  return { text: vehicleType, variant }
}

export const vehicleConfig: CatalogConfig = {
  name: 'vehicles',
  title: 'Vehicles',
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'vehicle_type',
      label: 'Type',
      type: 'badge',
      sortable: true,
      formatter: (value: any) => {
        return formatVehicleType(typeof value === 'string' ? value : null)
      }
    },
    {
      key: 'size',
      label: 'Size',
      type: 'text',
      sortable: true,
      className: 'text-center',
      formatter: (value: any) => {
        return formatSize(typeof value === 'string' ? value : null)
      }
    },
    {
      key: 'capacity',
      label: 'Crew/Passengers',
      type: 'text',
      className: 'text-center'
    },
    {
      key: 'speed',
      label: 'Speed/Pace',
      type: 'text',
      formatter: (value: any) => {
        return value || '—'
      }
    },
    {
      key: 'terrain',
      label: 'Terrain',
      type: 'array',
      formatter: (value: any) => {
        return formatTerrain(Array.isArray(value) ? value : null)
      }
    },
    {
      key: 'source',
      label: 'Source',
      type: 'text',
      sortable: true,
      formatter: (value: any) => {
        if (!value || typeof value !== 'string') return '—'
        return value
      }
    }
  ],
  filters: [
    {
      type: 'multiselect',
      key: 'vehicle_types',
      label: 'Vehicle Type',
      apiSource: 'get_vehicle_types'
    },
    {
      type: 'multiselect',
      key: 'sizes',
      label: 'Size',
      apiSource: 'get_vehicle_sizes'
    },
    {
      type: 'multiselect',
      key: 'terrains',
      label: 'Terrain',
      apiSource: 'get_vehicle_terrains'
    }
  ],
  searchCommands: {
    search: 'search_vehicles',
    details: 'get_vehicle_details',
    sources: 'get_vehicle_statistics'
  },
  emptyMessage: {
    title: 'No vehicles found',
    subtitle: 'Search for ships, mounts, siege engines, and other vehicles',
    noResults: 'No vehicles found matching your criteria'
  },
  searchPlaceholder: 'Search vehicles...'
}