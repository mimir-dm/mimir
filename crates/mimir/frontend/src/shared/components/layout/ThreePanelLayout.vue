<template>
  <div class="three-panel-layout">
    <aside class="panel-left">
      <slot name="left"></slot>
    </aside>
    <nav class="panel-center">
      <slot name="center"></slot>
    </nav>
    <main class="panel-right">
      <slot name="right"></slot>
    </main>
  </div>
</template>

<script setup lang="ts">
// Three-panel layout component for book reader
// Provides flexible slots for left (library), center (TOC), and right (content) panels
</script>

<style scoped>
.three-panel-layout {
  display: flex;
  height: 100vh;
  width: 100%;
  overflow: hidden;
}

.panel-left {
  width: 250px;
  min-width: 200px;
  background: var(--color-surface, #1a1a1a);
  border-right: 1px solid var(--color-border, #333);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.panel-center {
  width: 300px;
  min-width: 250px;
  background: var(--color-background, #0d0d0d);
  border-right: 1px solid var(--color-border, #333);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.panel-right {
  flex: 1;
  background: var(--color-background, #0d0d0d);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

/* Responsive adjustments */
@media (max-width: 1200px) {
  .panel-left {
    width: 200px;
    min-width: 150px;
  }
  
  .panel-center {
    width: 250px;
    min-width: 200px;
  }
}

@media (max-width: 768px) {
  .three-panel-layout {
    flex-direction: column;
  }
  
  .panel-left,
  .panel-center {
    width: 100%;
    max-height: 30vh;
    border-right: none;
    border-bottom: 1px solid var(--color-border, #333);
  }
}
</style>