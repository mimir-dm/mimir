<template>
  <section class="dashboard-section npcs-section">
    <div class="section-header">
      <h3>NPCs</h3>
      <button class="btn-add" @click="$emit('add')" title="Add NPC">+</button>
    </div>
    <div v-if="npcs.length === 0" class="section-empty">
      No NPCs assigned
    </div>
    <div v-else class="npc-cards">
      <div
        v-for="npc in npcs"
        :key="npc.id"
        class="npc-card"
        @click="$emit('view', npc)"
      >
        <span class="npc-name">{{ npc.name }}</span>
        <span class="npc-role">{{ npc.role || 'NPC' }}</span>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
export interface ModuleNpc {
  id: string
  module_id: string
  name: string
  role: string | null
  description: string | null
  appearance: string | null
  personality: string | null
  motivation: string | null
  secrets: string | null
}

defineProps<{
  npcs: ModuleNpc[]
}>()

defineEmits<{
  add: []
  view: [npc: ModuleNpc]
}>()
</script>

<style scoped>
.dashboard-section {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-sm);
  padding-bottom: var(--spacing-xs);
  border-bottom: 1px solid var(--color-border);
}

.section-header h3 {
  margin: 0;
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text);
}

.btn-add {
  width: 20px;
  height: 20px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
}

.btn-add:hover {
  background: var(--color-primary-500);
  color: var(--color-background);
  border-color: var(--color-primary-500);
}

.section-empty {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  text-align: center;
  padding: var(--spacing-md);
}

.npc-cards {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-xs);
}

.npc-card {
  display: flex;
  flex-direction: column;
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-left: 3px solid var(--color-warning);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.npc-card:hover {
  border-color: var(--color-primary-500);
  border-left-color: var(--color-warning);
}

.npc-name {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--color-text);
}

.npc-role {
  font-size: 0.65rem;
  color: var(--color-text-secondary);
}
</style>
