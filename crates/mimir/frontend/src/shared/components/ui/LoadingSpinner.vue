<template>
  <div class="loading-container">
    <div class="loading-spinner"></div>
    <p v-if="message" class="loading-message">{{ message }}</p>
  </div>
</template>

<script setup lang="ts">
interface Props {
  message?: string
}

defineProps<Props>()
</script>

<style scoped>
.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-xl, 24px);
  min-height: 200px;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 4px solid var(--color-surface, #1a1a1a);
  border-top-color: var(--color-primary, #4a9eff);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.loading-message {
  margin-top: var(--spacing-md, 12px);
  color: var(--color-text-secondary, #999);
  font-size: var(--font-size-base, 1rem);
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style>