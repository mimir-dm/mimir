<template>
  <div class="checkbox-group">
    <span v-if="label" class="checkbox-group__label">{{ label }}</span>
    <div class="checkbox-group__options">
      <label
        v-for="option in normalizedOptions"
        :key="option.value"
        class="checkbox-group__option"
      >
        <input
          type="checkbox"
          class="checkbox-group__input"
          :checked="modelValue.includes(option.value)"
          @change="toggleOption(option.value)"
        />
        <span class="checkbox-group__box"></span>
        <span class="checkbox-group__text">{{ option.label }}</span>
      </label>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface SelectOption {
  value: string
  label: string
}

interface Props {
  modelValue: string[]
  options: (string | SelectOption)[]
  label?: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: string[]]
}>()

const normalizedOptions = computed(() => {
  return props.options.map(opt => {
    if (typeof opt === 'string') {
      return { value: opt, label: opt }
    }
    return opt
  })
})

function toggleOption(value: string) {
  const current = [...props.modelValue]
  const index = current.indexOf(value)
  if (index === -1) {
    current.push(value)
  } else {
    current.splice(index, 1)
  }
  emit('update:modelValue', current)
}
</script>

<style scoped>
.checkbox-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.checkbox-group__label {
  font-size: 0.85rem;
  color: var(--color-text-secondary);
  margin-right: 4px;
}

.checkbox-group__options {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.checkbox-group__option {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  font-size: 0.85rem;
  color: var(--color-text);
}

.checkbox-group__input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}

.checkbox-group__box {
  width: 16px;
  height: 16px;
  border: 2px solid var(--color-border);
  border-radius: 3px;
  background: var(--color-surface);
  transition: all 0.15s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.checkbox-group__box::after {
  content: '\2715';
  font-size: 12px;
  font-weight: bold;
  color: var(--color-primary);
  opacity: 0;
  transform: scale(0);
  transition: all 0.15s ease;
}

.checkbox-group__input:checked + .checkbox-group__box {
  border-color: var(--color-primary);
}

.checkbox-group__input:checked + .checkbox-group__box::after {
  opacity: 1;
  transform: scale(1);
}

.checkbox-group__option:hover .checkbox-group__box {
  border-color: var(--color-primary);
}

.checkbox-group__text {
  user-select: none;
}
</style>
