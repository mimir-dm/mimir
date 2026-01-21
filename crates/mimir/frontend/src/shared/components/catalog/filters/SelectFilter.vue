<template>
  <div class="catalog-table__filter-group">
    <label v-if="label" class="filter-label">{{ label }}</label>
    <select 
      :value="modelValue"
      @change="$emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
      class="catalog-table__filter-select"
    >
      <option value="">{{ placeholder || `All ${label || 'Options'}` }}</option>
      <template v-if="grouped">
        <optgroup v-for="group in groupedOptions" :key="group.label" :label="group.label">
          <option 
            v-for="option in group.options"
            :key="getOptionValue(option)" 
            :value="getOptionValue(option)"
          >
            {{ getOptionLabel(option) }}
          </option>
        </optgroup>
      </template>
      <template v-else>
        <option 
          v-for="option in normalizedOptions"
          :key="getOptionValue(option)" 
          :value="getOptionValue(option)"
        >
          {{ getOptionLabel(option) }}
        </option>
      </template>
    </select>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Option {
  value: string
  label: string
}

interface GroupedOption {
  label: string
  options: Option[]
}

interface Props {
  modelValue: string
  label?: string
  placeholder?: string
  options: (string | Option)[]
  grouped?: boolean
  groupedOptions?: GroupedOption[]
}

const props = defineProps<Props>()

defineEmits<{
  'update:modelValue': [value: string]
}>()

const normalizedOptions = computed(() => {
  return props.options.map(option => 
    typeof option === 'string' 
      ? { value: option, label: option }
      : option
  )
})

function getOptionValue(option: string | Option): string {
  return typeof option === 'string' ? option : option.value
}

function getOptionLabel(option: string | Option): string {
  return typeof option === 'string' ? option : option.label
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

.catalog-table__filter-select {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background-color: var(--color-surface);
  color: var(--color-text);
  font-size: 0.875rem;
}

.catalog-table__filter-select:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px var(--color-primary-100);
}
</style>