<template>
  <div class="stage-transition-card" v-if="available">
    <h3>Ready for Next Stage!</h3>
    <p>{{ prompt }}</p>
    <button class="btn btn-primary btn-large" @click="$emit('transition')">
      Advance to {{ nextStageName }} →
    </button>
  </div>
</template>

<script setup lang="ts">
interface Props {
  available: boolean
  prompt: string
  nextStageName: string
}

defineProps<Props>()

defineEmits<{
  transition: []
}>()
</script>