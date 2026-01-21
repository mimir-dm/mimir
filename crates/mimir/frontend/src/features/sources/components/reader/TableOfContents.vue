<template>
  <Panel title="Contents" variant="default">
    <div class="toc-list">
      <div v-if="sections && sections.length > 0">
        <div v-for="(section, index) in sections" :key="index">
          <div 
            @click="$emit('select', index)"
            :class="['toc-item', { active: selectedSection === index, 'has-children': hasSubEntries(section) }]"
          >
            <!-- Chevron for collapsible sections -->
            <span 
              v-if="hasSubEntries(section)"
              class="toc-chevron"
              :class="{ expanded: expandedSections.has(index) }"
              @click.stop="toggleSection(index)"
            >
              ▶
            </span>
            <span 
              v-else
              class="toc-chevron-spacer"
            ></span>
            <span class="toc-name">{{ getSectionName(section) }}</span>
          </div>
          
          <!-- Show sub-entries if expanded -->
          <div v-if="hasSubEntries(section) && expandedSections.has(index)" class="toc-sub-entries">
            <template v-for="(entry, subIndex) in getSubEntries(section)" :key="`${index}-${subIndex}`">
              <div 
                class="toc-sub-item"
                :class="{ 'has-children': entry.children && entry.children.length > 0 }"
                :style="{ paddingLeft: `${30 + (entry.level * 15)}px` }"
                @click.stop="$emit('jump', index, entry.id)"
              >
                <!-- Chevron for collapsible sub-sections -->
                <span 
                  v-if="entry.children && entry.children.length > 0"
                  class="toc-chevron"
                  :class="{ expanded: expandedSections.has(`${index}-${subIndex}`) }"
                  @click.stop="toggleSection(`${index}-${subIndex}`)"
                >
                  ▶
                </span>
                <span 
                  v-else
                  class="toc-chevron-spacer"
                ></span>
                <span class="toc-sub-name">{{ entry.name }}</span>
              </div>
              
              <!-- Nested sub-entries -->
              <div v-if="entry.children && entry.children.length > 0 && expandedSections.has(`${index}-${subIndex}`)" class="toc-nested-entries">
                <div 
                  v-for="(child, childIndex) in entry.children"
                  :key="`${index}-${subIndex}-${childIndex}`"
                  class="toc-nested-item"
                  :style="{ paddingLeft: `${45 + ((entry.level + 1) * 15)}px` }"
                  @click.stop="$emit('jump', index, child.id)"
                >
                  {{ child.name }}
                </div>
              </div>
            </template>
          </div>
        </div>
      </div>
      <div v-else class="empty-toc">
        No content available
      </div>
    </div>
  </Panel>
</template>

<script setup lang="ts">
import Panel from '../../../../shared/components/layout/Panel.vue'
import { ref } from 'vue'
import { useBookNavigation } from '../../composables/useBookNavigation'
import { useBookContent } from '../../composables/useBookContent'
import type { BookSection } from '../../../../types/book'

interface Props {
  sections: BookSection[]
  selectedSection: number
}

interface Emits {
  (e: 'select', index: number): void
  (e: 'jump', sectionIndex: number, entryId: string): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const { getSubEntries } = useBookNavigation()
const { getSectionName } = useBookContent()

// Track which sections are expanded (using string keys to support nested sections)
const expandedSections = ref<Set<string | number>>(new Set())

// Toggle section expansion
function toggleSection(index: string | number) {
  if (expandedSections.value.has(index)) {
    expandedSections.value.delete(index)
  } else {
    expandedSections.value.add(index)
  }
  // Trigger reactivity
  expandedSections.value = new Set(expandedSections.value)
}

// Check if section has sub-entries
function hasSubEntries(section: BookSection): boolean {
  return !!(section.entries && Array.isArray(section.entries) && getSubEntries(section).length > 0)
}
</script>

<!-- Component styles have been moved to centralized CSS files -->