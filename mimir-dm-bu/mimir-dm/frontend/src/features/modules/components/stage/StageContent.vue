<template>
  <div class="stage-content-section">
    <div :class="`stage-${stage}`">
      <div class="activity-section">
        <h3>{{ content.title }}</h3>
        <p>{{ content.description }}</p>
        
        <template v-for="(section, idx) in content.sections" :key="idx">
          <h4 v-if="section.heading">{{ section.heading }}</h4>
          <p v-if="section.text">{{ section.text }}</p>
          <ol v-if="section.list">
            <li v-for="(item, itemIdx) in section.list" :key="itemIdx">
              <strong v-if="item.title">{{ item.title }}:</strong>
              <span v-if="item.text">{{ item.text }}</span>
              <br v-if="item.example">
              <em v-if="item.example">{{ item.example }}</em>
            </li>
          </ol>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface ContentSection {
  heading?: string
  text?: string
  list?: Array<{
    title?: string
    text?: string
    example?: string
  }>
}

interface StageContentData {
  title: string
  description: string
  sections: ContentSection[]
}

interface Props {
  stage: string
  content: StageContentData
}

defineProps<Props>()
</script>