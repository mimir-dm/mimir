---
id: v0-5-pinia-store-design
level: task
title: "v0.5 Pinia Store Design"
short_code: "MIMIR-T-0360"
created_at: 2026-01-19T22:06:59.835606+00:00
updated_at: 2026-01-21T16:38:38.813773+00:00
parent: MIMIR-I-0041
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0041
---

# v0.5 Pinia Store Design

## Parent Initiative
[[MIMIR-I-0041]] - Mimir v0.5 Architecture Rewrite

## Objective
Design Pinia stores that mirror backend state. Stores own all frontend state; components are purely presentational. Backend is source of truth; stores sync via Tauri commands.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] Store definitions for all domains
- [ ] Clear state shape for each store
- [ ] Actions map to Tauri commands
- [ ] Getters provide computed/filtered views
- [ ] Loading and error states handled consistently

## Architecture Principles

1. **Backend is truth**: Stores fetch from and sync to backend
2. **Stores own state**: Components read from stores, never manage local state for domain data
3. **Optimistic updates**: Update store immediately, rollback on error
4. **Normalized data**: Avoid deep nesting; use IDs and lookup maps

## Store Definitions

### useCampaignStore

```typescript
interface CampaignState {
  campaigns: Campaign[];
  activeCampaignId: number | null;
  loading: boolean;
  error: string | null;
}

interface Campaign {
  id: number;
  name: string;
  description: string | null;
  createdAt: string;
  archivedAt: string | null;
}

export const useCampaignStore = defineStore('campaign', {
  state: (): CampaignState => ({
    campaigns: [],
    activeCampaignId: null,
    loading: false,
    error: null,
  }),

  getters: {
    activeCampaign: (state) => 
      state.campaigns.find(c => c.id === state.activeCampaignId),
    
    activeCampaigns: (state) => 
      state.campaigns.filter(c => !c.archivedAt),
    
    archivedCampaigns: (state) => 
      state.campaigns.filter(c => c.archivedAt),
  },

  actions: {
    async fetchCampaigns(includeArchived = false) {
      this.loading = true;
      try {
        this.campaigns = await invoke('list_campaigns', { includeArchived });
      } catch (e) {
        this.error = String(e);
      } finally {
        this.loading = false;
      }
    },

    async setActive(campaignId: number) {
      await invoke('set_active_campaign', { campaignId });
      this.activeCampaignId = campaignId;
    },

    async create(input: CreateCampaign) {
      const campaign = await invoke('create_campaign', input);
      this.campaigns.push(campaign);
      return campaign;
    },

    async archive(id: number) {
      await invoke('archive_campaign', { id });
      const campaign = this.campaigns.find(c => c.id === id);
      if (campaign) campaign.archivedAt = new Date().toISOString();
    },
  },
});
```

### useModuleStore

```typescript
interface ModuleState {
  modules: Module[];
  moduleDetails: Record<number, ModuleDetails>;  // keyed by module ID
  loading: boolean;
  error: string | null;
}

interface Module {
  id: number;
  campaignId: number;
  name: string;
  moduleNumber: number;
  description: string | null;
}

interface ModuleDetails extends Module {
  monsters: ModuleMonster[];
  items: ModuleItem[];
  npcs: ModuleNpc[];
  documents: DocumentSummary[];
}

export const useModuleStore = defineStore('module', {
  state: (): ModuleState => ({
    modules: [],
    moduleDetails: {},
    loading: false,
    error: null,
  }),

  getters: {
    byId: (state) => (id: number) => state.modules.find(m => m.id === id),
    
    sortedModules: (state) => 
      [...state.modules].sort((a, b) => a.moduleNumber - b.moduleNumber),
  },

  actions: {
    async fetchModules(campaignId: number) {
      this.loading = true;
      try {
        this.modules = await invoke('list_modules', { campaignId });
      } finally {
        this.loading = false;
      }
    },

    async fetchDetails(moduleId: number) {
      const details = await invoke('get_module_details', { moduleId });
      this.moduleDetails[moduleId] = details;
      return details;
    },

    async addMonster(input: AddMonsterInput) {
      const monster = await invoke('add_monster_to_module', input);
      const details = this.moduleDetails[input.moduleId];
      if (details) details.monsters.push(monster);
      return monster;
    },

    async addItem(input: AddItemInput) {
      const item = await invoke('add_item_to_module', input);
      const details = this.moduleDetails[input.moduleId];
      if (details) details.items.push(item);
      return item;
    },
  },
});
```

### useCharacterStore

```typescript
interface CharacterState {
  characters: CharacterSummary[];
  characterDetails: Record<number, Character>;
  loading: boolean;
  error: string | null;
}

interface CharacterSummary {
  id: number;
  name: string;
  isNpc: boolean;
  race: string | null;
  class: string | null;
  level: number;
}

interface Character extends CharacterSummary {
  abilities: AbilityScores;
  maxHp: number | null;
  currentHp: number | null;
  armorClass: number | null;
  currency: Currency;
  inventory: InventoryItem[];
  npcRole: string | null;
  npcLocation: string | null;
  backstory: string | null;
}

export const useCharacterStore = defineStore('character', {
  state: (): CharacterState => ({
    characters: [],
    characterDetails: {},
    loading: false,
    error: null,
  }),

  getters: {
    pcs: (state) => state.characters.filter(c => !c.isNpc),
    npcs: (state) => state.characters.filter(c => c.isNpc),
    byId: (state) => (id: number) => state.characterDetails[id],
  },

  actions: {
    async fetchCharacters(campaignId: number, type: 'pc' | 'npc' | 'all' = 'all') {
      this.loading = true;
      try {
        this.characters = await invoke('list_characters', { characterType: type });
      } finally {
        this.loading = false;
      }
    },

    async fetchDetails(characterId: number) {
      const character = await invoke('get_character', { characterId });
      this.characterDetails[characterId] = character;
      return character;
    },

    async create(input: CreateCharacter) {
      const character = await invoke('create_character', input);
      this.characters.push(character);
      return character;
    },

    async update(id: number, input: UpdateCharacter) {
      const character = await invoke('edit_character', { characterId: id, ...input });
      this.characterDetails[id] = character;
      // Update summary in list
      const idx = this.characters.findIndex(c => c.id === id);
      if (idx >= 0) this.characters[idx] = character;
      return character;
    },
  },
});
```

### useDocumentStore

```typescript
interface DocumentState {
  documents: DocumentSummary[];
  documentContent: Record<number, Document>;
  loading: boolean;
  error: string | null;
}

interface DocumentSummary {
  id: number;
  title: string;
  documentType: string | null;
  moduleId: number | null;
}

interface Document extends DocumentSummary {
  content: string;
}

export const useDocumentStore = defineStore('document', {
  state: (): DocumentState => ({
    documents: [],
    documentContent: {},
    loading: false,
    error: null,
  }),

  getters: {
    byModule: (state) => (moduleId: number) => 
      state.documents.filter(d => d.moduleId === moduleId),
    
    campaignLevel: (state) => 
      state.documents.filter(d => !d.moduleId),
  },

  actions: {
    async fetchDocuments(moduleId?: number) {
      this.loading = true;
      try {
        this.documents = await invoke('list_documents', { moduleId });
      } finally {
        this.loading = false;
      }
    },

    async fetchContent(documentId: number) {
      const doc = await invoke('read_document', { documentId });
      this.documentContent[documentId] = doc;
      return doc;
    },

    async create(input: CreateDocument) {
      const doc = await invoke('create_user_document', input);
      this.documents.push(doc);
      return doc;
    },

    async searchReplace(id: number, search: string, replace: string, replaceAll = false) {
      const result = await invoke('edit_document', { documentId: id, search, replace, replaceAll });
      // Refetch content to get updated version
      await this.fetchContent(id);
      return result;
    },
  },
});
```

### useMapStore

```typescript
interface MapState {
  maps: MapSummary[];
  mapDetails: Record<number, MapDetails>;
  loading: boolean;
}

interface MapDetails {
  map: Map;
  tokens: Token[];
  fogAreas: FogArea[];
  lights: LightSource[];
}

export const useMapStore = defineStore('map', {
  // Similar pattern to other stores
});
```

### useCatalogStore

```typescript
interface CatalogState {
  monsterResults: MonsterSummary[];
  itemResults: ItemSummary[];
  trapResults: TrapSummary[];
  loading: boolean;
}

export const useCatalogStore = defineStore('catalog', {
  state: (): CatalogState => ({
    monsterResults: [],
    itemResults: [],
    trapResults: [],
    loading: false,
  }),

  actions: {
    async searchMonsters(query: MonsterQuery) {
      this.loading = true;
      try {
        this.monsterResults = await invoke('search_monsters', query);
      } finally {
        this.loading = false;
      }
    },

    async searchItems(query: ItemQuery) {
      this.loading = true;
      try {
        this.itemResults = await invoke('search_items', query);
      } finally {
        this.loading = false;
      }
    },
  },
});
```

## Component Pattern

Components should be thin wrappers around store state:

```vue
<script setup lang="ts">
const campaignStore = useCampaignStore();
const moduleStore = useModuleStore();

// Reactive access to store state
const campaign = computed(() => campaignStore.activeCampaign);
const modules = computed(() => moduleStore.sortedModules);

// Actions trigger store mutations
async function createModule(name: string) {
  await moduleStore.create({ 
    campaignId: campaign.value!.id, 
    name 
  });
}
</script>

<template>
  <div v-if="campaign">
    <h1>{{ campaign.name }}</h1>
    <ModuleList :modules="modules" @create="createModule" />
  </div>
</template>
```

## Dependencies
- Depends on: [[MIMIR-T-0358]] Service Layer API Design

## Investigation Findings (2026-01-21)

### Existing Store Analysis

**Store Inventory (mimir-dm-bu/mimir-dm/frontend/src/):**
- 7 main stores + 6 chat sub-stores
- Uses composition API `defineStore()` pattern (matches v0.5)
- Consistent `loading` and `error` state handling

**Campaign Store** (332 LOC) - 90% reusable:
- Already implements campaigns, activeCampaignId, loading, error pattern
- Has getters for activeCampaign, activeCampaigns, archivedCampaigns
- Actions map to Tauri commands correctly

**Character Store** (599 LOC) - 70% reusable:
- Needs state normalization (currently uses nested objects)
- PC/NPC separation exists but needs cleanup

**Module Store** - Well aligned with v0.5 design

**App Settings/Theme Stores** - 95% reusable as-is

### Key Differences from Design

| Aspect | v0.5 Design | Current Implementation |
|--------|-------------|----------------------|
| Return types | Direct types | `ApiResponse<T>` wrapper |
| Chat store | Monolithic | Sub-store coordinator pattern |
| State coordination | Store watches | Event bus (`dataEvents`) |
| Composables | Thin wrappers | Independent state (problematic) |

### What Can Be Reused
- ✅ All Pinia store patterns and structure
- ✅ Tauri invocation infrastructure
- ✅ `dataEvents` bus utility for cache invalidation
- ✅ Store action patterns

### What Needs Refactoring
- ❌ ApiResponse wrapper handling (add adapter or change backend)
- ❌ Move composable state into stores (30+ composables duplicating state)
- ❌ Flatten chat store sub-stores into single store
- ❌ Normalize character store state shape

### Migration Approach

**Option A - Adapter Layer:**
Add `unwrapResponse()` utility to handle ApiResponse in stores.
Minimal backend changes, frontend absorbs complexity.

**Option B - Backend Alignment:**
Update Tauri commands to return direct types.
Cleaner frontend, requires backend coordination.

**Recommendation:** Option A for initial migration, Option B as follow-up.

### Acceptance Criteria Status
- [x] Store definitions for all domains (exist, need refinement)
- [x] Clear state shape for each store (documented above)
- [x] Actions map to Tauri commands (implemented)
- [x] Getters provide computed/filtered views (implemented)
- [ ] Loading and error states handled consistently (needs ApiResponse fix)

## Progress

- 2026-01-21: Investigation complete. Existing stores ~70% aligned with design.