<template>
  <div class="welcome-message">
    <h2>{{ title }}</h2>
    <p v-for="(line, index) in lines" :key="index">{{ line }}</p>
    <slot></slot>
  </div>
</template>

<script setup lang="ts">
interface Props {
  title: string
  lines: string[]
}

defineProps<Props>()
</script>

<style scoped>
.welcome-message {
  text-align: center;
  padding: var(--spacing-xl, 24px);
}

.welcome-message h2 {
  color: var(--color-text, #e0e0e0);
  margin-bottom: var(--spacing-md, 12px);
  font-size: var(--font-size-2xl, 1.5rem);
}

.welcome-message p {
  color: var(--color-text-secondary, #999);
  font-size: var(--font-size-lg, 1.125rem);
  margin-bottom: var(--spacing-sm, 8px);
}
</style>