// Composable for managing the book library

import { ref } from 'vue'
import type { Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import type { BookInfo } from '../../../types/book'

// Catalog source info from backend
interface CatalogSourceInfo {
  id: string
  name: string
  enabled: boolean
  imported_at: string
}

export function useBookLibrary() {
  const libraryBooks: Ref<BookInfo[]> = ref([])
  const catalogSources: Ref<BookInfo[]> = ref([])
  const selectedBook: Ref<BookInfo | null> = ref(null)
  const isLoadingLibrary = ref(false)

  // Check for development mode
  const isDevelopment = import.meta.env.DEV


  // Load library books from backend (for reading mode - actual book archives)
  async function loadLibraryBooks() {
    try {
      isLoadingLibrary.value = true
      const response = await invoke<{ success: boolean; data: BookInfo[]; message?: string }>('list_library_books')

      if (response.success && response.data) {
        libraryBooks.value = response.data

        // Auto-select first book if none selected
        if (!selectedBook.value && response.data.length > 0) {
          selectedBook.value = response.data[0]
        }
      } else {
        libraryBooks.value = []
      }
    } catch (error) {
      libraryBooks.value = []
    } finally {
      isLoadingLibrary.value = false
    }
  }

  // Load catalog sources from backend (for catalog mode - imported 5etools data)
  async function loadCatalogSources() {
    try {
      isLoadingLibrary.value = true
      const response = await invoke<{ success: boolean; data?: CatalogSourceInfo[]; error?: string }>('list_catalog_sources')

      if (response.success && response.data) {
        // Convert catalog sources to BookInfo format for compatibility with Library.vue
        catalogSources.value = response.data.map(source => ({
          id: source.id,
          name: source.name,
          enabled: source.enabled,
          imported_at: source.imported_at,
        }))
      } else {
        catalogSources.value = []
      }
    } catch (error) {
      console.error('Failed to load catalog sources:', error)
      catalogSources.value = []
    } finally {
      isLoadingLibrary.value = false
    }
  }

  // Add books to the library (supports multi-select)
  async function addBook(): Promise<boolean> {
    try {
      // Open file dialog with multi-select
      const selected = await open({
        multiple: true,
        filters: [{
          name: 'Book Archive',
          extensions: ['tar.gz', 'gz']
        }],
        title: 'Select book archives to add to your library'
      })

      if (selected) {
        // Normalize to array
        const filePaths = Array.isArray(selected) ? selected : [selected]

        if (filePaths.length === 0) return false

        let successCount = 0
        const failures: string[] = []

        for (const filePath of filePaths) {
          const fileName = filePath.split('/').pop() || filePath
          try {
            const response = await invoke<{ success: boolean; data?: BookInfo; message?: string }>('import_catalog_from_zip', {
              archive_path: filePath
            })
            if (response.success && response.data) {
              successCount++
            } else {
              failures.push(`${fileName}: ${response.message}`)
            }
          } catch {
            failures.push(`${fileName}: Import failed`)
          }
        }

        // Reload books list
        await loadLibraryBooks()

        // Show summary if there were failures
        if (failures.length > 0) {
          if (successCount > 0) {
            alert(`Imported ${successCount} book(s).\n\nFailed:\n${failures.join('\n')}`)
          } else {
            alert(`Failed to import:\n${failures.join('\n')}`)
          }
        }

        return successCount > 0
      }
      return false
    } catch {
      alert('Failed to add books. Please try again.')
      return false
    }
  }

  // Remove a book from the library
  async function removeBook(book: BookInfo): Promise<boolean> {
    if (!confirm(`Are you sure you want to remove "${book.name}" from your library?`)) {
      return false
    }

    try {
      const response = await invoke<{ success: boolean; message?: string }>('delete_catalog_source', {
        source_code: book.id
      })
      
      if (response.success) {
        // Reload the library
        await loadLibraryBooks()
        
        // Clear selection if this was the selected book
        if (selectedBook.value?.id === book.id) {
          selectedBook.value = libraryBooks.value[0] || null
        }
        
        return true
      } else {
        alert(`Failed to remove book: ${response.message}`)
        return false
      }
    } catch (error) {
      alert('Failed to remove book. Please try again.')
      return false
    }
  }

  // Select a book
  function selectBook(book: BookInfo | null) {
    selectedBook.value = book
  }

  return {
    libraryBooks,
    catalogSources,
    selectedBook,
    isLoadingLibrary,
    isDevelopment,
    loadLibraryBooks,
    loadCatalogSources,
    addBook,
    removeBook,
    selectBook
  }
}