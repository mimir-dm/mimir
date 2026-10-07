<template>
  <div class="empty-state-container">
    <!-- Icon based on variant -->
    <div class="empty-state-icon">
      <component :is="VARIANT_ICONS[variant ?? 'generic']" :size="80" :stroke-width="1.5" aria-hidden="true" />
    </div>

    <h3 class="empty-state-title">{{ title }}</h3>
    <p v-if="description" class="empty-state-description">{{ description }}</p>

    <div v-if="$slots.action" class="empty-state-action">
      <slot name="action"></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import {
  BookText,
  FileText,
  FlaskConical,
  Folder,
  FolderPlus,
  SearchX,
  Skull,
  Sparkles,
  SquarePlus,
  User,
  Users,
} from '@lucide/vue'

export type EmptyStateVariant =
  | 'users'
  | 'characters'
  | 'campaigns'
  | 'books'
  | 'search'
  | 'generic'
  | 'documents'
  | 'modules'
  | 'homebrew'
  | 'monsters'
  | 'spells'

/** The icon of each variant (@lucide/vue, MIMIR-T-0683). */
const VARIANT_ICONS: Record<EmptyStateVariant, Component> = {
  users: Users,
  characters: User,
  campaigns: FolderPlus,
  books: BookText,
  search: SearchX,
  generic: SquarePlus,
  documents: FileText,
  modules: Folder,
  homebrew: FlaskConical,
  monsters: Skull,
  spells: Sparkles,
}

defineProps<{
  variant?: EmptyStateVariant
  title: string
  description?: string
}>()
</script>

<style scoped>
/* Uses global .empty-state-* classes from animations.css */
</style>
