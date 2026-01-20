<template>
  <div class="stage-transition-card">
    <h3>Ready for Next Stage!</h3>
    <p>{{ transitionPrompt }}</p>
    <button class="btn btn-primary btn-large" @click="$emit('transition')">
      Advance to {{ nextStageName }} →
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  nextStageName: string
  transitionPrompt: string
}>()

defineEmits<{
  transition: []
}>()
</script>
