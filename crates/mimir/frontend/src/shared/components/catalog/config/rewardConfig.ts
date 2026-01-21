import type { CatalogConfig } from './types'

export const rewardConfig: CatalogConfig = {
  name: 'rewards',
  title: 'Rewards & Supernatural Gifts',
  searchCommands: {
    search: 'search_rewards',
    details: 'get_reward_details',
    sources: 'get_reward_sources',
    itemTypes: 'get_reward_types'
  },
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'reward_type',
      label: 'Type',
      sortable: true,
      type: 'badge',
      className: 'catalog-table__cell-center',
      formatter: (reward: any) => {
        const typeClass = getTypeClass(reward.reward_type);
        return {
          text: reward.reward_type || 'Reward',
          variant: typeClass
        }
      }
    },
    {
      key: 'description',
      label: 'Description',
      type: 'text',
      className: 'catalog-table__cell-description'
    },
    {
      key: 'has_prerequisites',
      label: 'Prerequisites',
      type: 'prerequisites',
      className: 'catalog-table__cell-center',
      formatter: (reward: any) => ({
        hasPrerequisites: reward.has_prerequisites
      })
    },
    {
      key: 'source',
      label: 'Source',
      sortable: true,
      type: 'text',
      className: 'catalog-table__cell-source'
    }
  ],
  filters: [
    {
      type: 'text',
      key: 'search',
      label: 'Search',
      placeholder: 'Search rewards...'
    },
    {
      type: 'multiselect',
      key: 'reward_types',
      label: 'Type',
      options: [], // Will be populated dynamically from API
      apiSource: 'get_reward_types'
    },
    {
      type: 'checkbox',
      key: 'has_prerequisites',
      label: 'Has Prerequisites',
      tooltip: 'Show only rewards that have prerequisites'
    }
  ]
}

function getTypeClass(type: string): string {
  if (!type) return 'type-default'
  
  switch (type.toLowerCase()) {
    case 'blessing':
      return 'type-blessing'
    case 'epic boon':
    case 'boon':
      return 'type-boon'
    case 'charm':
      return 'type-charm'
    case 'feat':
      return 'type-feat'
    default:
      return 'type-default'
  }
}