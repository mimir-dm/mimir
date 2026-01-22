import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: () => import('../../views/HomeView.vue')
  },
  {
    path: '/campaigns',
    name: 'campaigns',
    redirect: '/'
  },
  {
    path: '/campaigns/new',
    name: 'campaign-new',
    component: () => import('../../features/campaigns/views/CampaignCreateView.vue')
  },
  {
    path: '/campaigns/:id',
    name: 'campaign-detail',
    component: () => import('../../features/campaigns/views/CampaignDetailView.vue'),
    props: true
  },
  // Dashboard Routes
  {
    path: '/campaigns/:id/dashboard',
    component: () => import('../../features/campaigns/views/CampaignDashboardView.vue'),
    props: true,
    children: [
      {
        path: '',
        redirect: to => `/campaigns/${to.params.id}/dashboard/campaign`
      },
      {
        path: 'campaign',
        name: 'dashboard-campaign',
        component: () => import('../../features/campaigns/components/dashboard/WorldTab.vue')
      },
      {
        path: 'modules',
        name: 'dashboard-modules',
        component: () => import('../../features/campaigns/components/dashboard/ModulesTab.vue')
      },
      {
        path: 'npcs',
        name: 'dashboard-npcs',
        component: () => import('../../features/campaigns/components/dashboard/NPCsTab.vue')
      },
      {
        path: 'pcs',
        name: 'dashboard-pcs',
        component: () => import('../../features/campaigns/components/dashboard/PCsTab.vue')
      },
      {
        path: 'modules/:moduleId/play',
        name: 'dashboard-play',
        component: () => import('../../features/modules/views/ModulePlayView.vue'),
        props: true,
        meta: { hideTabBar: true, fullScreen: true }
      },
    ]
  },
  {
    path: '/modules/:id',
    name: 'module-detail',
    component: () => import('../../features/modules/views/ModuleDetailView.vue'),
    props: true
  },
  {
    path: '/modules/:id/play',
    name: 'module-play',
    component: () => import('../../features/modules/views/ModulePlayView.vue'),
    props: true
  },
  {
    path: '/settings',
    name: 'settings',
    component: () => import('../../views/SettingsView.vue')
  },
  {
    path: '/characters',
    name: 'characters',
    component: () => import('../../features/characters/views/CharacterListView.vue')
  },
  {
    path: '/characters/:id',
    name: 'character-sheet',
    component: () => import('../../features/characters/views/CharacterSheetView.vue'),
    props: true
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router