import type { CatalogConfig } from './types'

export const objectConfig: CatalogConfig = {
  name: 'objects',
  title: 'Objects',
  searchCommands: {
    search: 'search_objects',
    details: 'get_object_details',
    sources: 'get_object_sources'
  },
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'object_type',
      label: 'Type',
      type: 'badge',
      className: 'catalog-table__cell-type',
      formatter: (item: any) => {
        // Return badge configuration for CatalogTable to handle
        const value = item.object_type
        let variant = 'default'
        switch (value) {
          case 'Siege Weapon':
            variant = 'siege'
            break
          case 'Generic':
            variant = 'generic'
            break
        }
        return { text: value, variant }
      }
    },
    {
      key: 'size',
      label: 'Size',
      type: 'text',
      className: 'catalog-table__cell-center'
    },
    {
      key: 'ac',
      label: 'AC',
      type: 'text',
      className: 'catalog-table__cell-center',
      formatter: (value: string) => value
    },
    {
      key: 'hp',
      label: 'HP',
      type: 'text',
      className: 'catalog-table__cell-center',
      formatter: (value: string) => value
    },
    {
      key: 'source',
      label: 'Source',
      sortable: true,
      type: 'text',
      className: 'catalog-table__cell-source',
      formatter: (value: string) => value
    }
  ],
  filters: [],
  emptyMessage: {
    title: 'No objects found',
    subtitle: 'Search for objects to see results',
    noResults: 'No objects found matching your criteria'
  }
}