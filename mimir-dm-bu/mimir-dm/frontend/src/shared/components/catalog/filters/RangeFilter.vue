<template>
  <div class="catalog-table__filter-group">
    <label v-if="label" class="filter-label">{{ label }}</label>
    <div class="range-filter">
      <input
        type="number"
        :value="modelValue.min"
        :min="min"
        :max="max"
        :step="step"
        :placeholder="`Min ${label || ''}`"
        class="range-filter__input"
        @input="updateMin(($event.target as HTMLInputElement).value)"
      />
      <span class="range-filter__separator">—</span>
      <input
        type="number"
        :value="modelValue.max"
        :min="min"
        :max="max"
        :step="step"
        :placeholder="`Max ${label || ''}`"
        class="range-filter__input"
        @input="updateMax(($event.target as HTMLInputElement).value)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
interface RangeValue {
  min?: number
  max?: number
}

interface Props {
  modelValue: RangeValue
  label?: string
  min?: number
  max?: number
  step?: number
}

const props = withDefaults(defineProps<Props>(), {
  step: 1
})

const emit = defineEmits<{
  'update:modelValue': [value: RangeValue]
}>()

function updateMin(value: string) {
  const numValue = value === '' ? undefined : Number(value)
  emit('update:modelValue', { ...props.modelValue, min: numValue })
}

function updateMax(value: string) {
  const numValue = value === '' ? undefined : Number(value)
  emit('update:modelValue', { ...props.modelValue, max: numValue })
}
</script>

<style scoped>
.filter-label {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  margin-bottom: 0.25rem;
}

.range-filter {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.range-filter__input {
  flex: 1;
  padding: 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background-color: var(--color-surface);
  color: var(--color-text);
  font-size: 0.875rem;
}

.range-filter__input:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px var(--color-primary-100);
}

.range-filter__separator {
  color: var(--color-text-secondary);
  font-weight: 500;
}
</style>