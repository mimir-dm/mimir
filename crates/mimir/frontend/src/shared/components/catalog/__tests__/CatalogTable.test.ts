import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import CatalogTable from '../CatalogTable.vue'
import type { CatalogConfig } from '../config/types'

// Mock child components
vi.mock('../../ui/MultiSelectFilter.vue', () => ({
  default: {
    name: 'MultiSelectFilter',
    template: '<div class="mock-multiselect-filter"></div>',
    props: ['label', 'options', 'modelValue']
  }
}))

vi.mock('../filters/SelectFilter.vue', () => ({
  default: {
    name: 'SelectFilter',
    template: '<div class="mock-select-filter"></div>',
    props: ['label', 'placeholder', 'options', 'grouped', 'groupedOptions', 'modelValue']
  }
}))

vi.mock('../filters/CheckboxFilter.vue', () => ({
  default: {
    name: 'CheckboxFilter',
    template: '<div class="mock-checkbox-filter"></div>',
    props: ['label', 'tooltip', 'modelValue']
  }
}))

vi.mock('../filters/RangeFilter.vue', () => ({
  default: {
    name: 'RangeFilter',
    template: '<div class="mock-range-filter"></div>',
    props: ['label', 'min', 'max', 'step', 'modelValue']
  }
}))

describe('CatalogTable', () => {
  const createConfig = (overrides: Partial<CatalogConfig> = {}): CatalogConfig => ({
    title: 'Test Catalog',
    name: 'items',
    columns: [
      { key: 'name', label: 'Name', sortable: true },
      { key: 'type', label: 'Type', sortable: true },
      { key: 'level', label: 'Level', sortable: true }
    ],
    filters: [],
    emptyMessage: {
      title: 'No items',
      subtitle: 'Search to find items',
      noResults: 'No items match your criteria'
    },
    ...overrides
  })

  const createData = () => [
    { name: 'Fireball', type: 'Evocation', level: 3 },
    { name: 'Magic Missile', type: 'Evocation', level: 1 },
    { name: 'Shield', type: 'Abjuration', level: 1 },
    { name: 'Counterspell', type: 'Abjuration', level: 3 }
  ]

  describe('rendering', () => {
    it('renders title from config', () => {
      const wrapper = mount(CatalogTable, {
        props: {
          config: createConfig({ title: 'Spells Catalog' }),
          data: [],
          searchPerformed: false,
          sortColumn: '',
          sortDirection: 'asc'
        }
      })

      expect(wrapper.find('.catalog-table__title').text()).toBe('Spells Catalog')
    })

    it('renders column headers', () => {
      const wrapper = mount(CatalogTable, {
        props: {
          config: createConfig(),
          data: [],
          searchPerformed: false,
          sortColumn: '',
          sortDirection: 'asc'
        }
      })

      const headers = wrapper.findAll('th')
      expect(headers).toHaveLength(3)
      expect(headers[0].text()).toContain('Name')
      expect(headers[1].text()).toContain('Type')
      expect(headers[2].text()).toContain('Level')
    })

    it('renders data rows', () => {
      const wrapper = mount(CatalogTable, {
        props: {
          config: createConfig(),
          data: createData(),
          searchPerformed: true,
          sortColumn: '',
          sortDirection: 'asc'
        }
      })

      const rows = wrapper.findAll('tbody tr.catalog-table__row')
      expect(rows).toHaveLength(4)
    })

    it('displays result count', () => {
      const wrapper = mount(CatalogTable, {
        props: {
          config: createConfig(),
          data: createData(),
          searchPerformed: true,
          sortColumn: '',
          sortDirection: 'asc'
        }
      })

      expect(wrapper.find('.catalog-table__result-count').text()).toBe('4 items')
    })
  })

  describe('empty state', () => {
    it('shows empty message when no data and search not performed', () => {
      const wrapper = mount(CatalogTable, {
        props: {
          config: createConfig(),
          data: [],
          searchPerformed: false,
          sortColumn: '',
          sortDirection: 'asc'
        }
      })

      const emptyRow = wrapper.find('.catalog-table__empty')
      expect(emptyRow.exists()).toBe(true)
      expect(emptyRow.find('h3').text()).toBe('Search to find items')
    })

    it('shows no results message when search performed but no data', () => {
      const wrapper = mount(CatalogTable, {
        props: {
          config: createConfig(),
          data: [],
          searchPerformed: true,
          sortColumn: '',
          sortDirection: 'asc'
        }
      })

      const emptyRow = wrapper.find('.catalog-table__empty')
      expect(emptyRow.exists()).toBe(true)
      expect(emptyRow.find('h3').text()).toBe('No items')
    })
  })

  describe('sorting', () => {
    it('emits sort event when sortable column header is clicked', async () => {
      const wrapper = mount(CatalogTable, {
        props: {
          config: createConfig(),
          data: createData(),
          searchPerformed: true,
          sortColumn: '',
          sortDirection: 'asc'
        }
      })

      const sortableHeader = wrapper.find('.catalog-table__sort-header')
      await sortableHeader.trigger('click')

      expect(wrapper.emitted('sort')).toBeTruthy()
      expect(wrapper.emitted('sort')![0]).toEqual(['name'])
    })

    it('displays sort indicator for active column', () => {
      const wrapper = mount(CatalogTable, {
        props: {
          config: createConfig(),
          data: createData(),
          searchPerformed: true,
          sortColumn: 'name',
          sortDirection: 'asc'
        }
      })

      const sortIcon = wrapper.find('.catalog-table__sort-icon')
      expect(sortIcon.text()).toBe('▲')
    })

    it('displays descending sort indicator', () => {
      const wrapper = mount(CatalogTable, {
        props: {
          config: createConfig(),
          data: createData(),
          searchPerformed: true,
          sortColumn: 'name',
          sortDirection: 'desc'
        }
      })

      const sortIcon = wrapper.find('.catalog-table__sort-icon')
      expect(sortIcon.text()).toBe('▼')
    })

    it('sorts data by string column ascending', () => {
      const wrapper = mount(CatalogTable, {
        props: {
          config: createConfig(),
          data: createData(),
          searchPerformed: true,
          sortColumn: 'name',
          sortDirection: 'asc'
        }
      })

      const rows = wrapper.findAll('tbody tr.catalog-table__row')
      // Sorted: Counterspell, Fireball, Magic Missile, Shield
      expect(rows[0].text()).toContain('Counterspell')
      expect(rows[3].text()).toContain('Shield')
    })

    it('sorts data by number column', () => {
      const wrapper = mount(CatalogTable, {
        props: {
          config: createConfig(),
          data: createData(),
          searchPerformed: true,
          sortColumn: 'level',
          sortDirection: 'asc'
        }
      })

      const rows = wrapper.findAll('tbody tr.catalog-table__row')
      // Level 1 items should come first
      expect(rows[0].text()).toContain('1')
    })
  })

  describe('selection', () => {
    it('emits select event when row is clicked', async () => {
      const data = createData()
      const wrapper = mount(CatalogTable, {
        props: {
          config: createConfig(),
          data,
          searchPerformed: true,
          sortColumn: '',
          sortDirection: 'asc'
        }
      })

      const firstRow = wrapper.find('tbody tr.catalog-table__row')
      await firstRow.trigger('click')

      expect(wrapper.emitted('select')).toBeTruthy()
      expect(wrapper.emitted('select')![0][0]).toEqual(data[0])
    })
  })

  describe('filters', () => {
    it('renders multiselect filters', () => {
      const config = createConfig({
        filters: [
          { key: 'type', label: 'Type', type: 'multiselect', options: ['Evocation', 'Abjuration'] }
        ]
      })

      const wrapper = mount(CatalogTable, {
        props: {
          config,
          data: createData(),
          searchPerformed: true,
          sortColumn: '',
          sortDirection: 'asc'
        }
      })

      expect(wrapper.find('.mock-multiselect-filter').exists()).toBe(true)
    })

    it('renders checkbox filters', () => {
      const config = createConfig({
        filters: [
          { key: 'ritual', label: 'Ritual Only', type: 'checkbox' }
        ]
      })

      const wrapper = mount(CatalogTable, {
        props: {
          config,
          data: createData(),
          searchPerformed: true,
          sortColumn: '',
          sortDirection: 'asc'
        }
      })

      expect(wrapper.find('.mock-checkbox-filter').exists()).toBe(true)
    })

    it('renders range filters', () => {
      const config = createConfig({
        filters: [
          { key: 'level', label: 'Spell Level', type: 'range', min: 0, max: 9 }
        ]
      })

      const wrapper = mount(CatalogTable, {
        props: {
          config,
          data: createData(),
          searchPerformed: true,
          sortColumn: '',
          sortDirection: 'asc'
        }
      })

      expect(wrapper.find('.mock-range-filter').exists()).toBe(true)
    })
  })

  describe('cell rendering', () => {
    it('renders cell values from data', () => {
      const wrapper = mount(CatalogTable, {
        props: {
          config: createConfig(),
          data: [{ name: 'Test Spell', type: 'Test Type', level: 5 }],
          searchPerformed: true,
          sortColumn: '',
          sortDirection: 'asc'
        }
      })

      const cells = wrapper.findAll('tbody td')
      expect(cells[0].text()).toBe('Test Spell')
      expect(cells[1].text()).toBe('Test Type')
      expect(cells[2].text()).toBe('5')
    })

    it('shows dash for missing values', () => {
      const wrapper = mount(CatalogTable, {
        props: {
          config: createConfig(),
          data: [{ name: 'Incomplete', type: null, level: undefined }],
          searchPerformed: true,
          sortColumn: '',
          sortDirection: 'asc'
        }
      })

      const cells = wrapper.findAll('tbody td')
      expect(cells[1].text()).toBe('—')
      expect(cells[2].text()).toBe('—')
    })

    it('uses formatter when provided', () => {
      const config = createConfig({
        columns: [
          { key: 'name', label: 'Name', sortable: true },
          { key: 'level', label: 'Level', sortable: true, formatter: (val: number) => `Level ${val}` }
        ]
      })

      const wrapper = mount(CatalogTable, {
        props: {
          config,
          data: [{ name: 'Test', level: 3 }],
          searchPerformed: true,
          sortColumn: '',
          sortDirection: 'asc'
        }
      })

      const levelCell = wrapper.findAll('tbody td')[1]
      expect(levelCell.text()).toBe('Level 3')
    })
  })
})
