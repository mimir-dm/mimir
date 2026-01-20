<template>
  <div class="two-panel-layout">
    <div class="panel-left">
      <slot name="left"></slot>
    </div>
    <div class="panel-right">
      <slot name="right"></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
// Two-panel layout component for catalog mode
</script>

<style scoped>
.two-panel-layout {
  display: flex;
  height: calc(100vh - 40px); /* Account for mode switcher bar */
  width: 100%;
  overflow: hidden;
  background-color: var(--color-background);
}

.panel-left {
  width: 250px;
  min-width: 200px;
  max-width: 350px;
  height: 100%;
  border-right: 1px solid var(--color-border);
  overflow-y: auto;
  flex-shrink: 0;
}

.panel-right {
  flex: 1;
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>