<template>
  <div class="stage-header">
    <h2>{{ title }}</h2>
    <p class="stage-subtitle">{{ subtitle }}</p>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  title: string
  subtitle: string
}>()
</script>

<style scoped>
.stage-header {
  margin-bottom: var(--spacing-md, 12px);
}

.stage-header h2 {
  margin: 0 0 var(--spacing-xs, 4px) 0;
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--color-text, #e0e0e0);
}

.stage-subtitle {
  margin: 0;
  font-size: 0.9rem;
  color: var(--color-text-muted, #888);
}
</style>
