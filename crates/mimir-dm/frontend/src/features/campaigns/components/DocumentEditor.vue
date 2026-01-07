<template>
  <div class="document-editor">
    <!-- Editor Header -->
    <div class="editor-header">
      <div class="header-left">
        <button class="btn-icon" @click="$emit('close')" title="Back to overview">
          ← Back
        </button>
        <h2>{{ document?.title || 'Untitled Document' }}</h2>
      </div>
      <div class="header-right">
        <span v-if="saveStatus" class="save-status" :class="saveStatus">
          {{ saveStatusText }}
        </span>
        <button
          class="btn-toolbar"
          @click="exportToPdf"
          :disabled="exporting || !document?.id || document.id < 0"
          title="Export document to PDF"
        >
          {{ exporting ? 'Exporting...' : 'Export PDF' }}
        </button>
      </div>
    </div>

    <!-- Editor Content -->
    <div class="editor-content">
      <!-- Editor -->
      <div class="editor-wrapper">
        <!-- Toolbar -->
        <div v-if="editor" class="editor-toolbar" style="display: flex; align-items: center; gap: var(--spacing-xs); padding: var(--spacing-sm) var(--spacing-lg); background-color: var(--color-surface); border-bottom: 1px solid var(--color-border); flex-wrap: wrap;">
          <button
            @click="editor?.chain().focus().toggleHeading({ level: 1 }).run()"
            :class="{ 'btn-toolbar--active': editor?.isActive('heading', { level: 1 }) }"
            class="btn-toolbar"
          >
            H1
          </button>
          <button
            @click="editor?.chain().focus().toggleHeading({ level: 2 }).run()"
            :class="{ 'btn-toolbar--active': editor?.isActive('heading', { level: 2 }) }"
            class="btn-toolbar"
          >
            H2
          </button>
          <button
            @click="editor?.chain().focus().toggleHeading({ level: 3 }).run()"
            :class="{ 'btn-toolbar--active': editor?.isActive('heading', { level: 3 }) }"
            class="btn-toolbar"
          >
            H3
          </button>
          <div class="btn-toolbar-divider"></div>
          <button
            @click="editor?.chain().focus().toggleBold().run()"
            :class="{ 'btn-toolbar--active': editor?.isActive('bold') }"
            class="btn-toolbar"
          >
            <strong>B</strong>
          </button>
          <button
            @click="editor?.chain().focus().toggleItalic().run()"
            :class="{ 'btn-toolbar--active': editor?.isActive('italic') }"
            class="btn-toolbar"
          >
            <em>I</em>
          </button>
          <button
            @click="editor?.chain().focus().toggleStrike().run()"
            :class="{ 'btn-toolbar--active': editor?.isActive('strike') }"
            class="btn-toolbar"
          >
            <strike>S</strike>
          </button>
          <div class="btn-toolbar-divider"></div>
          <button
            @click="editor?.chain().focus().toggleBulletList().run()"
            :class="{ 'btn-toolbar--active': editor?.isActive('bulletList') }"
            class="btn-toolbar"
          >
            • List
          </button>
          <button
            @click="editor?.chain().focus().toggleOrderedList().run()"
            :class="{ 'btn-toolbar--active': editor?.isActive('orderedList') }"
            class="btn-toolbar"
          >
            1. List
          </button>
          <button
            @click="editor?.chain().focus().toggleBlockquote().run()"
            :class="{ 'btn-toolbar--active': editor?.isActive('blockquote') }"
            class="btn-toolbar"
          >
            " Quote
          </button>
          <div class="btn-toolbar-divider"></div>
          <!-- Table commands -->
          <button
            @click="editor?.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()"
            class="btn-toolbar"
            title="Insert Table"
          >
            + Table
          </button>
          <button
            @click="editor?.chain().focus().deleteTable().run()"
            :disabled="!editor?.isActive('table')"
            class="btn-toolbar"
            title="Delete Table"
          >
            - Table
          </button>
          <button
            @click="editor?.chain().focus().addColumnAfter().run()"
            :disabled="!editor?.isActive('table')"
            class="btn-toolbar"
            title="Add Column"
          >
            + Col
          </button>
          <button
            @click="editor?.chain().focus().deleteColumn().run()"
            :disabled="!editor?.isActive('table')"
            class="btn-toolbar"
            title="Delete Column"
          >
            - Col
          </button>
          <button
            @click="editor?.chain().focus().addRowAfter().run()"
            :disabled="!editor?.isActive('table')"
            class="btn-toolbar"
            title="Add Row"
          >
            + Row
          </button>
          <button
            @click="editor?.chain().focus().deleteRow().run()"
            :disabled="!editor?.isActive('table')"
            class="btn-toolbar"
            title="Delete Row"
          >
            - Row
          </button>
          <div class="btn-toolbar-divider"></div>
          <button
            @click="editor?.chain().focus().setHorizontalRule().run()"
            class="btn-toolbar"
          >
            — Rule
          </button>
          <button
            @click="editor?.chain().focus().undo().run()"
            :disabled="!editor?.can().undo()"
            class="btn-toolbar"
          >
            ↶ Undo
          </button>
          <button
            @click="editor?.chain().focus().redo().run()"
            :disabled="!editor?.can().redo()"
            class="btn-toolbar"
          >
            ↷ Redo
          </button>
        </div>
        <EditorContent :editor="editor" class="editor-area" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, computed, nextTick } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import Placeholder from '@tiptap/extension-placeholder'
import { Markdown } from '@tiptap/markdown'
import { Table } from '@tiptap/extension-table'
import { TableRow } from '@tiptap/extension-table-row'
import { TableCell } from '@tiptap/extension-table-cell'
import { TableHeader } from '@tiptap/extension-table-header'
import { invoke } from '@tauri-apps/api/core'
import { debounce } from '../../../shared/utils/debounce'
import { PrintService } from '../../../services/PrintService'

const props = defineProps<{
  document: any
  campaignId: number
}>()

const emit = defineEmits<{
  close: []
  updated: [document: any]
  'stage-transitioned': [campaign: any]
}>()

// State
const showPreview = ref(false)
const saveStatus = ref<'saving' | 'saved' | 'error' | null>(null)
const pendingContent = ref<string | null>(null)
const exporting = ref(false)
const isLoading = ref(false) // Prevent saves during document load

// Initialize Tiptap editor with markdown support
const editor = useEditor({
  content: '',
  extensions: [
    StarterKit.configure({
      heading: {
        levels: [1, 2, 3, 4, 5, 6]
      }
    }),
    Placeholder.configure({
      placeholder: 'Start writing your document...'
    }),
    Markdown,
    Table.configure({
      resizable: true
    }),
    TableRow,
    TableCell,
    TableHeader
  ],
  onCreate: ({ editor: e }) => {
    // Load document when editor is ready
    if (props.document) {
      // Use a small delay to ensure editor is fully ready
      setTimeout(() => loadDocument(), 50)
    } else if (pendingContent.value) {
      // If we have pending content from before editor was ready, set it now
      e.commands.setContent(pendingContent.value, { contentType: 'markdown' })
      pendingContent.value = null
    }
  },
  onUpdate: ({ editor }) => {
    debouncedSave()
  }
})

// Computed
const saveStatusText = computed(() => {
  switch (saveStatus.value) {
    case 'saving': return 'Saving...'
    case 'saved': return 'Saved'
    case 'error': return 'Error saving'
    default: return ''
  }
})

// Load document content
const loadDocument = async () => {
  if (!props.document?.file_path) {
    console.warn('No file_path for document:', props.document)
    return
  }

  isLoading.value = true // Prevent saves during load

  try {
    const response = await invoke<{ success: boolean; data?: string; error?: string }>('read_document_file', {
      filePath: props.document.file_path
    })

    if (response.success && response.data) {
      // Set markdown content - Tiptap will parse it
      if (editor.value) {
        editor.value.commands.setContent(response.data, { contentType: 'markdown' })
      } else {
        // Store content to set later
        pendingContent.value = response.data
      }
    } else {
      console.error('Failed to load document:', response.error || 'Unknown error', 'Path:', props.document.file_path)
    }
  } catch (e) {
    console.error('Error loading document:', e, 'Path:', props.document.file_path)
  } finally {
    isLoading.value = false // Re-enable saves
  }
}

// Get content as markdown
const getMarkdown = (): string => {
  if (!editor.value) return ''

  // Use the official markdown extension API
  return editor.value.getMarkdown()
}

// Save document content
const saveDocument = async () => {
  if (!props.document?.file_path) return
  if (isLoading.value) return // Don't save while loading

  saveStatus.value = 'saving'
  
  try {
    // Get content as markdown
    const markdown = getMarkdown()
    
    // Just save the file - no need to update database for every save
    await invoke('save_document_file', {
      filePath: props.document.file_path,
      content: markdown
    })
    
    saveStatus.value = 'saved'
    setTimeout(() => {
      saveStatus.value = null
    }, 2000)
  } catch (e) {
    saveStatus.value = 'error'
    setTimeout(() => {
      saveStatus.value = null
    }, 3000)
  }
}

// Debounced save function
const debouncedSave = debounce(saveDocument, 1000)

// Export document to PDF
const exportToPdf = async () => {
  if (!props.document?.id || props.document.id < 0) return

  exporting.value = true

  try {
    // Save any pending changes first
    await saveDocument()

    // Export the document
    const result = await PrintService.exportCampaignDocument(props.document.id)

    // Generate filename from document title
    const filename = `${props.document.title || 'document'}.pdf`
      .replace(/[^a-z0-9\s\-_.]/gi, '')
      .replace(/\s+/g, '_')

    // Save the PDF
    const savedPath = await PrintService.savePdf(result, filename)

    if (savedPath) {
      saveStatus.value = 'saved'
      setTimeout(() => {
        saveStatus.value = null
      }, 2000)
    }
  } catch (e) {
    console.error('Failed to export PDF:', e)
    saveStatus.value = 'error'
    setTimeout(() => {
      saveStatus.value = null
    }, 3000)
  } finally {
    exporting.value = false
  }
}

// Toggle preview mode
const togglePreview = () => {
  showPreview.value = !showPreview.value
  if (editor.value) {
    // Toggle editor's editable state
    editor.value.setEditable(!showPreview.value)
  }
}

// Mark document as complete
const markComplete = async () => {
  try {
    // Save any pending changes first
    await saveDocument()
    
    const response = await invoke<{ data: any }>('complete_document', {
      document_id: props.document.id
    })
    
    if (response.data) {
      emit('updated', response.data)
      
      // Check if stage is complete after marking this document
      const stageStatus = await invoke<{ success: boolean; data: any }>('check_campaign_stage_completion', {
        campaignId: props.campaignId
      })
      
      if (stageStatus.success && stageStatus.data.can_progress) {
        // Show transition prompt
        showTransitionPrompt(stageStatus.data)
      }
    }
  } catch (e) {
  }
}

// Show stage transition prompt
const showTransitionPrompt = (status: any) => {
  const metadata = status.stage_metadata
  
  // For now, just show an alert. In a real app, you'd use a modal
  const message = metadata.transition_prompt || 
    `You can always edit this document later, but make sure your party has a chance to look at this and provide feedback before progressing.`
  
  if (confirm(message + '\n\nWould you like to progress to the next stage?')) {
    transitionToNextStage(status.next_stage)
  }
}

// Transition to next stage
const transitionToNextStage = async (nextStage: string) => {
  try {
    const response = await invoke<{ success: boolean; data: any }>('transition_campaign_stage', {
      campaignId: props.campaignId,
      newStage: nextStage
    })
    
    if (response.success) {
      // Emit event to refresh the campaign view
      emit('stage-transitioned', response.data)
    }
  } catch (e) {
  }
}

// Watch for document changes
watch(() => props.document, (newDoc, oldDoc) => {
  // Load if document changed - check multiple fields since temporary docs have id = -1
  if (newDoc && (
    newDoc?.id !== oldDoc?.id ||
    newDoc?.file_path !== oldDoc?.file_path ||
    newDoc?.template_id !== oldDoc?.template_id
  )) {
    if (editor.value) {
      // Prevent saves while switching documents
      isLoading.value = true
      // Clear the editor first to avoid mixing content
      editor.value.commands.clearContent()
      loadDocument()
    } else {
      // Editor not ready yet, store for later
      pendingContent.value = null
    }
  }
}, { deep: true })

// Load content when component mounts
onMounted(() => {
  // Load document content if available
  if (props.document) {
    loadDocument()
  }
})

// Cleanup
onBeforeUnmount(() => {
  editor.value?.destroy()
})
</script>

<!-- Component styles have been moved to centralized CSS files -->
