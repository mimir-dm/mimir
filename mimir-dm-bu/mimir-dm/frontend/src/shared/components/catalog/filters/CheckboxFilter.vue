<template>
  <div class="catalog-table__filter-group">
    <label class="form-checkbox" :title="tooltip">
      <input 
        type="checkbox" 
        class="form-checkbox__input"
        :checked="modelValue"
        @change="$emit('update:modelValue', ($event.target as HTMLInputElement).checked)"
      />
      <span class="form-checkbox__box"></span>
      <span class="form-checkbox__label">{{ label }}</span>
    </label>
  </div>
</template>

<script setup lang="ts">
interface Props {
  modelValue: boolean
  label: string
  tooltip?: string
}

defineProps<Props>()

defineEmits<{
  'update:modelValue': [value: boolean]
}>()
</script>

<style scoped>
/* Checkbox styles are already defined globally in the forms.css */
</style>