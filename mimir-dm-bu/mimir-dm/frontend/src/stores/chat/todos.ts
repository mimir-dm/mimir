import { ref, computed, type Ref, type ComputedRef } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { TodoItem } from './types'

// Re-export TodoItem type
export type { TodoItem } from './types'

interface TodosState {
  todos: Ref<TodoItem[]>
  todosVisible: Ref<boolean>
}

interface TodosComputed {
  todoProgress: ComputedRef<{ completed: number; total: number; percentage: number }>
  currentTodo: ComputedRef<TodoItem | undefined>
  hasTodos: ComputedRef<boolean>
}

interface TodosActions {
  updateTodos: (newTodos: TodoItem[]) => void
  toggleTodosVisibility: () => void
  clearTodos: () => void
  loadTodosForSession: (sessionId: string) => Promise<void>
  extractTodosFromMessage: (content: string, currentSessionId: string | null) => Promise<boolean>
  configureTodoStorage: (storagePath: string) => Promise<void>
}

export function createTodosStore(): TodosState & TodosComputed & TodosActions {
  // State
  const todos = ref<TodoItem[]>([])
  const todosVisible = ref(false)

  // Computed
  const todoProgress = computed(() => {
    const total = todos.value.length
    if (total === 0) return { completed: 0, total: 0, percentage: 0 }

    const completed = todos.value.filter(t => t.status === 'completed').length
    const percentage = Math.round((completed / total) * 100)
    return { completed, total, percentage }
  })

  const currentTodo = computed(() => {
    return todos.value.find(t => t.status === 'in_progress')
  })

  const hasTodos = computed(() => {
    return todos.value.length > 0
  })

  // Actions
  const updateTodos = (newTodos: TodoItem[]) => {
    todos.value = newTodos
  }

  const toggleTodosVisibility = () => {
    todosVisible.value = !todosVisible.value
  }

  const clearTodos = () => {
    todos.value = []
  }

  const loadTodosForSession = async (sessionId: string) => {
    try {
      console.log(`Loading todos for session: ${sessionId}`)
      const response = await invoke<{success: boolean, data?: TodoItem[], error?: string}>('get_session_todos', { sessionId })
      console.log('Todo API response:', response)
      if (response.success && response.data) {
        todos.value = response.data
        console.log(`Successfully loaded ${response.data.length} todos for session ${sessionId}:`, response.data)
      } else {
        console.error('Todo API returned error:', response.error || 'Unknown error')
      }
    } catch (err) {
      console.error('Failed to load todos:', err)
    }
  }

  const configureTodoStorage = async (storagePath: string) => {
    try {
      console.log(`Configuring todo storage to: ${storagePath}`)
      const response = await invoke<{success: boolean, data?: null, error?: string}>('configure_todo_storage', {
        storagePath
      })
      if (response.success) {
        console.log('Todo storage configured successfully')
      } else {
        console.error('Failed to configure todo storage:', response.error)
      }
    } catch (err) {
      console.error('Failed to configure todo storage:', err)
    }
  }

  const extractTodosFromMessage = async (content: string, currentSessionId: string | null) => {
    // Look for assistant messages that indicate todo updates
    if (content.includes("Todos have been modified successfully")) {
      // This indicates todos were updated via the tool, refresh them
      if (currentSessionId) {
        console.log('Detected todo update in message, refreshing todos...')
        await loadTodosForSession(currentSessionId)
        return true
      }
    }
    return false
  }

  return {
    // State
    todos,
    todosVisible,

    // Computed
    todoProgress,
    currentTodo,
    hasTodos,

    // Actions
    updateTodos,
    toggleTodosVisibility,
    clearTodos,
    loadTodosForSession,
    extractTodosFromMessage,
    configureTodoStorage
  }
}
