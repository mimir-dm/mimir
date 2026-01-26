import { ref, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Document } from '@/types/api'

interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

/**
 * Composable for managing play notes with auto-save
 * Uses the documents table to store play notes per module
 */
export function usePlayNotes() {
  // State
  const notesCollapsed = ref(true)
  const notesContent = ref('')
  const notesDocumentId = ref<string | null>(null)
  const notesSaving = ref(false)
  const notesLastSaved = ref(false)
  let saveTimeout: ReturnType<typeof setTimeout> | null = null

  // Toggle notes panel visibility
  function toggleNotes() {
    notesCollapsed.value = !notesCollapsed.value
  }

  // Load play notes document for a module
  async function loadNotesForModule(moduleId: string): Promise<void> {
    try {
      // Get all documents for the module
      const response = await invoke<ApiResponse<Document[]>>('list_module_documents', {
        moduleId
      })

      if (response.success && response.data) {
        // Find the play_notes document
        const playNotesDoc = response.data.find(doc => doc.doc_type === 'play_notes')
        if (playNotesDoc) {
          notesDocumentId.value = playNotesDoc.id
          notesContent.value = playNotesDoc.content || ''
        } else {
          console.log('No play_notes document found for module')
          notesDocumentId.value = null
          notesContent.value = ''
        }
      }
    } catch (error) {
      console.error('Failed to load play notes:', error)
      notesDocumentId.value = null
      notesContent.value = ''
    }
  }

  // Save notes to database
  async function saveNotes(): Promise<void> {
    if (!notesDocumentId.value) {
      console.warn('No play notes document ID set, cannot save')
      return
    }

    notesSaving.value = true
    notesLastSaved.value = false

    try {
      await invoke<ApiResponse<Document>>('update_document', {
        id: notesDocumentId.value,
        request: {
          content: notesContent.value
        }
      })
      notesLastSaved.value = true
      // Clear the "Saved" indicator after 2 seconds
      setTimeout(() => {
        notesLastSaved.value = false
      }, 2000)
    } catch (error) {
      console.error('Failed to save notes:', error)
    } finally {
      notesSaving.value = false
    }
  }

  // Handle notes input with debounced auto-save
  function handleNotesInput() {
    // Clear any pending save
    if (saveTimeout) {
      clearTimeout(saveTimeout)
    }

    // Schedule save after 1 second of inactivity
    saveTimeout = setTimeout(() => {
      saveNotes()
    }, 1000)
  }

  // Cleanup function - save any pending notes
  function cleanup() {
    if (saveTimeout) {
      clearTimeout(saveTimeout)
      saveNotes()
    }
  }

  // Auto-cleanup when component unmounts
  onBeforeUnmount(() => {
    cleanup()
  })

  return {
    // State
    notesCollapsed,
    notesContent,
    notesDocumentId,
    notesSaving,
    notesLastSaved,
    // Actions
    toggleNotes,
    loadNotesForModule,
    saveNotes,
    handleNotesInput,
    cleanup
  }
}
