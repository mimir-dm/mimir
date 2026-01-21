<template>
  <div class="form-select-custom">
    <button 
      @click="toggleDropdown" 
      class="btn-filter"
      :class="{ 'btn-filter--active': isOpen || hasActiveFilters }"
    >
      {{ label }}
      <span v-if="activeCount > 0" class="btn-filter__count">{{ activeCount }}</span>
      <svg class="form-select-custom__chevron" :class="{ rotated: isOpen }" width="12" height="12" viewBox="0 0 12 12">
        <path d="M3 4.5L6 7.5L9 4.5" stroke="currentColor" stroke-width="1.5" fill="none"/>
      </svg>
    </button>
    
    <div v-if="isOpen" class="form-select-custom__dropdown">
      <div class="form-select-custom__search" v-if="options.length > 10">
        <input 
          type="text" 
          v-model="searchTerm" 
          :placeholder="`Search ${label.toLowerCase()}...`"
        >
      </div>
      
      <div class="form-select-custom__options">
        <label 
          v-for="option in filteredOptions" 
          :key="option"
          class="form-checkbox"
        >
          <input 
            type="checkbox" 
            class="form-checkbox__input"
            :checked="modelValue.includes(option)"
            @change="toggleOption(option)"
          >
          <span class="form-checkbox__box"></span>
          <span class="form-checkbox__label">{{ option }}</span>
        </label>
      </div>
      
      <div class="form-select-custom__actions">
        <button @click="clearAll" class="form-select-custom__action-btn">Clear All</button>
        <button @click="selectAll" class="form-select-custom__action-btn">Select All</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

interface Props {
  label: string
  options: string[]
  modelValue: string[]
}

const props = defineProps<Props>()
const emit = defineEmits<{
  'update:modelValue': [value: string[]]
}>()

const isOpen = ref(false)
const searchTerm = ref('')

const filteredOptions = computed(() => {
  if (!searchTerm.value) return props.options
  const term = searchTerm.value.toLowerCase()
  return props.options.filter(opt => opt.toLowerCase().includes(term))
})

const hasActiveFilters = computed(() => props.modelValue.length > 0)
const activeCount = computed(() => props.modelValue.length)

function toggleDropdown() {
  isOpen.value = !isOpen.value
}

function toggleOption(option: string) {
  const current = [...props.modelValue]
  const index = current.indexOf(option)
  
  if (index > -1) {
    current.splice(index, 1)
  } else {
    current.push(option)
  }
  
  emit('update:modelValue', current)
}

function clearAll() {
  emit('update:modelValue', [])
}

function selectAll() {
  emit('update:modelValue', [...filteredOptions.value])
}

// Close dropdown when clicking outside
function handleClickOutside(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (!target.closest('.form-select-custom')) {
    isOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
/* Only component-specific styling that can't be handled by global classes */
.form-select-custom__chevron {
  transition: transform var(--transition-base);
}

.form-select-custom__chevron.rotated {
  transform: rotate(180deg);
}
</style>