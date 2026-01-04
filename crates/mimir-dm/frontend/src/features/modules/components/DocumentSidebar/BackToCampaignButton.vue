<template>
  <div class="back-button-container" v-if="campaignId">
    <router-link :to="`/campaigns/${campaignId}/dashboard`" class="back-to-campaign">
      ← Back to Campaign
    </router-link>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  campaignId?: number
}>()
</script>

<style scoped>
.back-button-container {
  padding: var(--spacing-md);
  border-bottom: 1px solid var(--color-border);
}

.back-to-campaign {
  display: inline-flex;
  align-items: center;
  padding: var(--spacing-sm) var(--spacing-md);
  background-color: var(--color-surface-variant);
  color: var(--color-text);
  text-decoration: none;
  border-radius: var(--radius-sm);
  font-size: 0.875rem;
  font-weight: 500;
  transition: all var(--transition-base);
  width: 100%;
  justify-content: center;
}

.back-to-campaign:hover {
  background-color: var(--color-primary-100);
  color: var(--color-primary-600);
  transform: translateX(-2px);
}
</style>
