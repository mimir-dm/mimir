<template>
  <MainLayout>
    <div class="character-sheet">
      <div v-if="loading" class="loading-state">
        Loading character...
      </div>

      <div v-else-if="error" class="error-state">
        <p>{{ error }}</p>
        <button @click="loadCharacter" class="btn btn-primary">Retry</button>
      </div>

      <template v-else-if="character">
        <!-- Header -->
        <div class="sheet-header">
          <div class="header-content">
            <h1 class="character-name">{{ character.name }}</h1>
            <div class="character-subtitle">
              <span v-if="character.race_name">{{ character.race_name }}</span>
              <span v-if="character.is_npc === 1" class="npc-badge">NPC</span>
              <span v-else-if="character.player_name" class="player-name">
                Player: {{ character.player_name }}
              </span>
            </div>
          </div>
          <div class="header-actions">
            <button @click="goBack" class="btn btn-secondary">Back</button>
            <button @click="openInventory" class="btn btn-primary">Inventory</button>
          </div>
        </div>

        <!-- Main Content -->
        <div class="sheet-content">
          <!-- Ability Scores -->
          <section class="sheet-section">
            <h2>Ability Scores</h2>
            <div class="ability-grid">
              <div class="ability-score">
                <div class="ability-name">STR</div>
                <div class="ability-value">{{ character.strength }}</div>
                <div class="ability-modifier">{{ formatModifier(character.strength) }}</div>
              </div>
              <div class="ability-score">
                <div class="ability-name">DEX</div>
                <div class="ability-value">{{ character.dexterity }}</div>
                <div class="ability-modifier">{{ formatModifier(character.dexterity) }}</div>
              </div>
              <div class="ability-score">
                <div class="ability-name">CON</div>
                <div class="ability-value">{{ character.constitution }}</div>
                <div class="ability-modifier">{{ formatModifier(character.constitution) }}</div>
              </div>
              <div class="ability-score">
                <div class="ability-name">INT</div>
                <div class="ability-value">{{ character.intelligence }}</div>
                <div class="ability-modifier">{{ formatModifier(character.intelligence) }}</div>
              </div>
              <div class="ability-score">
                <div class="ability-name">WIS</div>
                <div class="ability-value">{{ character.wisdom }}</div>
                <div class="ability-modifier">{{ formatModifier(character.wisdom) }}</div>
              </div>
              <div class="ability-score">
                <div class="ability-name">CHA</div>
                <div class="ability-value">{{ character.charisma }}</div>
                <div class="ability-modifier">{{ formatModifier(character.charisma) }}</div>
              </div>
            </div>
          </section>

          <!-- Currency -->
          <section class="sheet-section">
            <h2>Currency</h2>
            <div class="currency-grid">
              <div class="currency-item">
                <span class="currency-label">CP</span>
                <span class="currency-value">{{ character.cp }}</span>
              </div>
              <div class="currency-item">
                <span class="currency-label">SP</span>
                <span class="currency-value">{{ character.sp }}</span>
              </div>
              <div class="currency-item">
                <span class="currency-label">EP</span>
                <span class="currency-value">{{ character.ep }}</span>
              </div>
              <div class="currency-item">
                <span class="currency-label">GP</span>
                <span class="currency-value">{{ character.gp }}</span>
              </div>
              <div class="currency-item">
                <span class="currency-label">PP</span>
                <span class="currency-value">{{ character.pp }}</span>
              </div>
            </div>
          </section>

          <!-- Background Info -->
          <section v-if="character.background_name" class="sheet-section">
            <h2>Background</h2>
            <p>{{ character.background_name }}</p>
          </section>

          <!-- Personality -->
          <section v-if="hasPersonality" class="sheet-section">
            <h2>Personality</h2>
            <div v-if="character.traits" class="personality-item">
              <strong>Traits:</strong> {{ character.traits }}
            </div>
            <div v-if="character.ideals" class="personality-item">
              <strong>Ideals:</strong> {{ character.ideals }}
            </div>
            <div v-if="character.bonds" class="personality-item">
              <strong>Bonds:</strong> {{ character.bonds }}
            </div>
            <div v-if="character.flaws" class="personality-item">
              <strong>Flaws:</strong> {{ character.flaws }}
            </div>
          </section>

          <!-- NPC Info -->
          <section v-if="character.is_npc === 1 && hasNpcInfo" class="sheet-section">
            <h2>NPC Details</h2>
            <div v-if="character.role" class="npc-item">
              <strong>Role:</strong> {{ character.role }}
            </div>
            <div v-if="character.location" class="npc-item">
              <strong>Location:</strong> {{ character.location }}
            </div>
            <div v-if="character.faction" class="npc-item">
              <strong>Faction:</strong> {{ character.faction }}
            </div>
          </section>
        </div>
      </template>
    </div>

    <!-- Inventory Manager Dialog -->
    <InventoryManager
      v-if="character"
      :visible="showInventory"
      :character-id="characterId"
      :character-data="character"
      @close="showInventory = false"
      @updated="loadCharacter"
    />
  </MainLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import MainLayout from '../../../shared/components/layout/MainLayout.vue'
import InventoryManager from '../components/InventoryManager.vue'
import { useCharacterStore } from '../../../stores/characters'
import type { Character } from '../../../types/character'
import { abilityModifier } from '../../../types/character'

const route = useRoute()
const router = useRouter()
const characterStore = useCharacterStore()

const characterId = computed(() => route.params.id as string)
const character = ref<Character | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const showInventory = ref(false)

const hasPersonality = computed(() => {
  if (!character.value) return false
  return character.value.traits || character.value.ideals || character.value.bonds || character.value.flaws
})

const hasNpcInfo = computed(() => {
  if (!character.value) return false
  return character.value.role || character.value.location || character.value.faction
})

const formatModifier = (score: number) => {
  const mod = abilityModifier(score)
  return mod >= 0 ? `+${mod}` : `${mod}`
}

const loadCharacter = async () => {
  loading.value = true
  error.value = null

  try {
    character.value = await characterStore.getCharacter(characterId.value)
    if (!character.value) {
      error.value = 'Character not found'
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load character'
  } finally {
    loading.value = false
  }
}

const goBack = () => {
  router.back()
}

const openInventory = () => {
  showInventory.value = true
}

onMounted(() => {
  loadCharacter()
})
</script>

<style scoped>
.character-sheet {
  max-width: 900px;
  margin: 0 auto;
  padding: var(--spacing-lg);
}

.loading-state,
.error-state {
  text-align: center;
  padding: var(--spacing-xl);
  color: var(--color-text-secondary);
}

.sheet-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--spacing-xl);
  padding-bottom: var(--spacing-lg);
  border-bottom: 2px solid var(--color-border);
}

.character-name {
  font-size: 2rem;
  font-weight: bold;
  color: var(--color-text);
  margin: 0;
}

.character-subtitle {
  display: flex;
  gap: var(--spacing-sm);
  align-items: center;
  color: var(--color-text-secondary);
  margin-top: var(--spacing-xs);
}

.npc-badge {
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-warning-bg, #fef3c7);
  color: var(--color-warning, #d97706);
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
}

.player-name {
  font-style: italic;
}

.header-actions {
  display: flex;
  gap: var(--spacing-sm);
}

.sheet-content {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xl);
}

.sheet-section {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--spacing-lg);
}

.sheet-section h2 {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 var(--spacing-md) 0;
  padding-bottom: var(--spacing-sm);
  border-bottom: 1px solid var(--color-border);
}

.ability-grid {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: var(--spacing-md);
}

.ability-score {
  text-align: center;
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.ability-name {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.ability-value {
  font-size: 1.5rem;
  font-weight: bold;
  color: var(--color-text);
  margin: var(--spacing-xs) 0;
}

.ability-modifier {
  font-size: 1rem;
  color: var(--color-primary-600);
  font-weight: 500;
}

.currency-grid {
  display: flex;
  gap: var(--spacing-lg);
}

.currency-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-xs);
}

.currency-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
}

.currency-value {
  font-size: 1.25rem;
  font-weight: bold;
  color: var(--color-text);
}

.personality-item,
.npc-item {
  margin-bottom: var(--spacing-sm);
}

.personality-item:last-child,
.npc-item:last-child {
  margin-bottom: 0;
}

@media (max-width: 768px) {
  .ability-grid {
    grid-template-columns: repeat(3, 1fr);
  }

  .currency-grid {
    flex-wrap: wrap;
  }

  .sheet-header {
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .header-actions {
    width: 100%;
  }

  .header-actions .btn {
    flex: 1;
  }
}
</style>
