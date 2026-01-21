<template>
  <div class="search-filters">
    <template v-if="category === 'Spells'">
      <select v-model="localFilters.spells.class" @change="emitUpdate">
        <option value="">All Classes</option>
        <option value="Artificer">Artificer</option>
        <option value="Bard">Bard</option>
        <option value="Cleric">Cleric</option>
        <option value="Druid">Druid</option>
        <option value="Paladin">Paladin</option>
        <option value="Ranger">Ranger</option>
        <option value="Sorcerer">Sorcerer</option>
        <option value="Warlock">Warlock</option>
        <option value="Wizard">Wizard</option>
      </select>

      <select v-model="localFilters.spells.level" @change="emitUpdate">
        <option value="">All Levels</option>
        <option value="0">Cantrip</option>
        <option v-for="level in 9" :key="level" :value="String(level)">
          Level {{ level }}
        </option>
      </select>

      <select v-model="localFilters.spells.school" @change="emitUpdate">
        <option value="">All Schools</option>
        <option value="abjuration">Abjuration</option>
        <option value="conjuration">Conjuration</option>
        <option value="divination">Divination</option>
        <option value="enchantment">Enchantment</option>
        <option value="evocation">Evocation</option>
        <option value="illusion">Illusion</option>
        <option value="necromancy">Necromancy</option>
        <option value="transmutation">Transmutation</option>
      </select>

      <label class="checkbox-label">
        <input
          type="checkbox"
          v-model="localFilters.spells.ritual"
          @change="emitUpdate"
        >
        Ritual
      </label>

      <label class="checkbox-label">
        <input
          type="checkbox"
          v-model="localFilters.spells.concentration"
          @change="emitUpdate"
        >
        Concentration
      </label>
    </template>
    
    <template v-else-if="category === 'Equipment'">
      <select v-model="localFilters.equipment.type" @change="emitUpdate">
        <option value="">All Types</option>
        <option value="weapon">Weapon</option>
        <option value="armor">Armor</option>
        <option value="gear">Adventuring Gear</option>
        <option value="tool">Tools</option>
      </select>
    </template>
    
    <template v-else-if="category === 'Magic Items'">
      <select v-model="localFilters.magicItems.rarity" @change="emitUpdate">
        <option value="">All Rarities</option>
        <option value="common">Common</option>
        <option value="uncommon">Uncommon</option>
        <option value="rare">Rare</option>
        <option value="very rare">Very Rare</option>
        <option value="legendary">Legendary</option>
        <option value="artifact">Artifact</option>
      </select>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import type { SearchFilters } from '../../services/SearchService'

interface Props {
  category: string
  filters: SearchFilters
}

const props = defineProps<Props>()

const emit = defineEmits<{
  update: [filters: SearchFilters]
}>()

const localFilters = ref<SearchFilters>({ ...props.filters })

watch(() => props.filters, (newFilters) => {
  localFilters.value = { ...newFilters }
}, { deep: true })

function emitUpdate() {
  emit('update', localFilters.value)
}
</script>

<style scoped>
.search-filters {
  display: flex;
  gap: var(--spacing-sm, 8px);
  align-items: center;
  flex-wrap: wrap;
}

.search-filters select {
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
  background: var(--color-background, #0d0d0d);
  border: 1px solid var(--color-border, #333);
  border-radius: 4px;
  color: var(--color-text, #e0e0e0);
  font-size: 0.85rem;
  cursor: pointer;
}

.search-filters select:focus {
  outline: none;
  border-color: var(--color-primary, #4a9eff);
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--color-text-secondary, #999);
  font-size: 0.85rem;
  cursor: pointer;
}

.checkbox-label input[type="checkbox"] {
  cursor: pointer;
}
</style>