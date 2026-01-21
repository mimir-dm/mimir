<template>
  <div class="stage-header">
    <div class="header-content">
      <div class="header-text">
        <h2>{{ module?.name || 'Module' }}</h2>
        <p class="stage-subtitle">{{ stageInfo.subtitle }}</p>
      </div>
      <div class="header-actions">
        <slot name="actions" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Module } from '@/types'

interface StageInfo {
  title: string
  subtitle: string
  color?: string
  phase?: string
}

interface Props {
  module: Module | null
  stageInfo: StageInfo
}

defineProps<Props>()
</script>

<style scoped>
.stage-header {
  margin-bottom: 1.5rem;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 1rem;
}

.header-text {
  flex: 1;
}

.header-text h2 {
  margin: 0;
}

.stage-subtitle {
  margin: 0.25rem 0 0 0;
  color: var(--color-text-muted);
}

.header-actions {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}
</style>
