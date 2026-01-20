<template>
  <div class="empty-state-container">
    <!-- Icon based on variant -->
    <div class="empty-state-icon">
      <!-- Users/Players -->
      <svg v-if="variant === 'users'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
        <circle cx="9" cy="7" r="4"/>
        <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
        <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
      </svg>

      <!-- Characters -->
      <svg v-else-if="variant === 'characters'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
        <circle cx="12" cy="7" r="4"/>
        <path d="M12 14l-2 7h4l-2-7z" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>

      <!-- Campaigns/Maps -->
      <svg v-else-if="variant === 'campaigns'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"/>
        <path d="M12 11v6"/>
        <path d="M9 14h6"/>
      </svg>

      <!-- Books/Library -->
      <svg v-else-if="variant === 'books'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
        <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
        <path d="M8 7h8"/>
        <path d="M8 11h6"/>
      </svg>

      <!-- Search/No Results -->
      <svg v-else-if="variant === 'search'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="11" cy="11" r="8"/>
        <path d="M21 21l-4.35-4.35"/>
        <path d="M8 8l6 6"/>
        <path d="M14 8l-6 6"/>
      </svg>

      <!-- Generic/Default -->
      <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
        <path d="M12 8v8"/>
        <path d="M8 12h8"/>
      </svg>
    </div>

    <h3 class="empty-state-title">{{ title }}</h3>
    <p v-if="description" class="empty-state-description">{{ description }}</p>

    <div v-if="$slots.action" class="empty-state-action">
      <slot name="action"></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  variant?: 'users' | 'characters' | 'campaigns' | 'books' | 'search' | 'generic'
  title: string
  description?: string
}>()
</script>

<style scoped>
/* Uses global .empty-state-* classes from animations.css */
</style>
