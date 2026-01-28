<template>
  <div
    class="character-card"
    :class="{ 'is-npc': character.is_npc === 1 }"
    @click="$emit('click', character)"
  >
    <div class="card-header">
      <h3 class="character-name">{{ character.name }}</h3>
      <span v-if="character.is_npc === 1" class="npc-badge">NPC</span>
    </div>

    <div class="character-details">
      {{ formattedDetails }}
    </div>

    <div v-if="showPlayer && character.player_name" class="character-player">
      {{ character.player_name }}
    </div>

    <div class="card-actions" @click.stop>
      <slot name="actions">
        <button @click="$emit('view', character)" class="btn btn-sm btn-ghost">
          View
        </button>
        <button @click="$emit('print', character)" class="btn btn-sm btn-ghost">
          PDF
        </button>
      </slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Character } from '@/types/character'
import { classString } from '@/types/character'

const props = withDefaults(defineProps<{
  character: Character
  showPlayer?: boolean
}>(), {
  showPlayer: true
})

defineEmits<{
  click: [character: Character]
  view: [character: Character]
  print: [character: Character]
}>()

const formattedDetails = computed(() => {
  const parts: string[] = []

  const classInfo = classString(props.character)
  if (classInfo !== 'No Class') {
    parts.push(classInfo)
  }

  if (props.character.race_name) {
    parts.push(props.character.race_name)
  }

  return parts.join(' • ') || 'No details'
})
</script>

<style scoped>
.character-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  cursor: pointer;
  transition: all var(--transition-base);
}

.character-card:hover {
  border-color: var(--color-primary-500);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.character-card.is-npc {
  border-left: 3px solid var(--color-warning);
}

.character-card.is-npc:hover {
  border-left-color: var(--color-warning);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-xs);
}

.character-name {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
}

.npc-badge {
  display: inline-block;
  padding: 2px 8px;
  font-size: 0.625rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  background-color: var(--color-warning);
  color: white;
  border-radius: var(--radius-sm);
  flex-shrink: 0;
}

.character-details {
  font-size: 0.875rem;
  color: var(--color-primary-500);
  margin-bottom: var(--spacing-xs);
}

.character-player {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-sm);
}

.card-actions {
  display: flex;
  gap: var(--spacing-xs);
  padding-top: var(--spacing-sm);
  border-top: 1px solid var(--color-border);
}

.card-actions .btn {
  flex: 1;
}
</style>
