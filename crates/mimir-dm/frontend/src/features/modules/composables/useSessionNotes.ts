import { ref, type Ref, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'

/**
 * Composable for managing session notes with auto-save
 * Handles loading, saving, and debounced auto-save of notes
 */
export function useSessionNotes() {
  // State
  const notesCollapsed = ref(true)
  const notesContent = ref('')
  const notesFilePath = ref('')
  const notesSaving = ref(false)
  const notesLastSaved = ref(false)
  let saveTimeout: ReturnType<typeof setTimeout> | null = null

  // Toggle notes panel visibility
  function toggleNotes() {
    notesCollapsed.value = !notesCollapsed.value
  }

  // Set the file path for notes (call after determining the path)
  function setNotesFilePath(path: string) {
    notesFilePath.value = path
  }

  // Load notes from file
  async function loadNotes() {
    if (!notesFilePath.value) return

    try {
      const response = await invoke<{ data: string }>('read_document_file', {
        filePath: notesFilePath.value
      })
      if (response.data) {
        notesContent.value = response.data
      }
    } catch (error) {
      // File might not exist yet, that's OK
      console.log('Notes file not found, will create on first save')
      notesContent.value = ''
    }
  }

  // Save notes to file
  async function saveNotes() {
    if (!notesFilePath.value) return

    notesSaving.value = true
    notesLastSaved.value = false

    try {
      await invoke('save_document_file', {
        filePath: notesFilePath.value,
        content: notesContent.value
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
    notesFilePath,
    notesSaving,
    notesLastSaved,
    // Actions
    toggleNotes,
    setNotesFilePath,
    loadNotes,
    saveNotes,
    handleNotesInput,
    cleanup
  }
}

/**
 * Build the notes file path for a module
 */
export function buildNotesFilePath(
  campaignDirectoryPath: string,
  moduleNumber: number
): string {
  const paddedNumber = String(moduleNumber).padStart(2, '0')
  return `${campaignDirectoryPath}/modules/module_${paddedNumber}/play-notes.md`
}
