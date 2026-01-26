/**
 * Composable for managing map tokens.
 * Provides token CRUD operations and state management.
 */
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Token, TokenSummary, CreateTokenRequest, UpdateTokenRequest, TokenType, TokenSize } from '@/types/api'
import { TOKEN_SIZE_GRID_SQUARES, TOKEN_TYPE_COLORS } from '@/types/api'

interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

export function useTokens(mapId: string) {
  const tokens = ref<Token[]>([])
  const tokenSummaries = ref<TokenSummary[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Cache for token images (token_id -> base64 data URL)
  const tokenImages = ref<Map<string, string>>(new Map())

  // Computed
  const visibleTokens = computed(() =>
    tokens.value.filter(t => t.visible_to_players)
  )

  const tokensByType = computed(() => {
    const grouped: Record<TokenType, Token[]> = {
      monster: [],
      pc: [],
      npc: [],
      trap: [],
      marker: []
    }
    for (const token of tokens.value) {
      grouped[token.token_type as TokenType].push(token)
    }
    return grouped
  })

  // Load all tokens for the map
  async function loadTokens(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      const response = await invoke<ApiResponse<Token[]>>('list_tokens', { mapId })
      if (response.success && response.data) {
        tokens.value = response.data
      } else {
        error.value = response.error || 'Failed to load tokens'
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load tokens'
      console.error('Failed to load tokens:', e)
    } finally {
      loading.value = false
    }
  }

  // Load token summaries (with monster/character names)
  async function loadTokenSummaries(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      const response = await invoke<ApiResponse<TokenSummary[]>>('list_token_summaries', { mapId })
      if (response.success && response.data) {
        tokenSummaries.value = response.data
        // Also update the base tokens array
        tokens.value = response.data
      } else {
        error.value = response.error || 'Failed to load token summaries'
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load token summaries'
      console.error('Failed to load token summaries:', e)
    } finally {
      loading.value = false
    }
  }

  // Create a new token
  async function createToken(request: CreateTokenRequest): Promise<Token | null> {
    loading.value = true
    error.value = null
    try {
      const response = await invoke<ApiResponse<Token>>('create_token', { request })
      if (response.success && response.data) {
        tokens.value.push(response.data)
        return response.data
      } else {
        error.value = response.error || 'Failed to create token'
        return null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create token'
      console.error('Failed to create token:', e)
      return null
    } finally {
      loading.value = false
    }
  }

  // Update a token
  async function updateToken(id: string, request: UpdateTokenRequest): Promise<Token | null> {
    loading.value = true
    error.value = null
    try {
      const response = await invoke<ApiResponse<Token>>('update_token', { id, request })
      if (response.success && response.data) {
        const index = tokens.value.findIndex(t => t.id === id)
        if (index !== -1) {
          tokens.value[index] = response.data
        }
        return response.data
      } else {
        error.value = response.error || 'Failed to update token'
        return null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update token'
      console.error('Failed to update token:', e)
      return null
    } finally {
      loading.value = false
    }
  }

  // Update token position (for drag operations)
  async function updateTokenPosition(id: string, x: number, y: number): Promise<Token | null> {
    try {
      const response = await invoke<ApiResponse<Token>>('update_token_position', { id, x, y })
      if (response.success && response.data) {
        const index = tokens.value.findIndex(t => t.id === id)
        if (index !== -1) {
          tokens.value[index] = response.data
        }
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to update token position:', e)
      return null
    }
  }

  // Toggle token visibility
  async function toggleVisibility(id: string): Promise<Token | null> {
    try {
      const response = await invoke<ApiResponse<Token>>('toggle_token_visibility', { id })
      if (response.success && response.data) {
        const index = tokens.value.findIndex(t => t.id === id)
        if (index !== -1) {
          tokens.value[index] = response.data
        }
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to toggle token visibility:', e)
      return null
    }
  }

  // Delete a token
  async function deleteToken(id: string): Promise<boolean> {
    loading.value = true
    error.value = null
    try {
      const response = await invoke<ApiResponse<void>>('delete_token', { id })
      if (response.success) {
        tokens.value = tokens.value.filter(t => t.id !== id)
        return true
      } else {
        error.value = response.error || 'Failed to delete token'
        return false
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete token'
      console.error('Failed to delete token:', e)
      return false
    } finally {
      loading.value = false
    }
  }

  // Get token display size in pixels based on grid size
  function getTokenDisplaySize(token: Token, gridSizePx: number): number {
    const gridSquares = TOKEN_SIZE_GRID_SQUARES[token.size as TokenSize] || 1
    return gridSquares * gridSizePx
  }

  // Get token color (from token or fallback based on type)
  function getTokenColor(token: Token): string {
    return token.color || TOKEN_TYPE_COLORS[token.token_type as TokenType] || '#666666'
  }

  // Load a token's image
  async function loadTokenImage(tokenId: string): Promise<string | null> {
    // Check cache first
    if (tokenImages.value.has(tokenId)) {
      return tokenImages.value.get(tokenId)!
    }

    try {
      const response = await invoke<ApiResponse<string>>('serve_token_image', { tokenId })
      if (response.success && response.data) {
        tokenImages.value.set(tokenId, response.data)
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to load token image:', e)
      return null
    }
  }

  // Load images for all monster tokens (convention-based paths on backend)
  async function loadAllTokenImages(): Promise<void> {
    const tokensWithImages = tokens.value.filter(t => t.token_type === 'monster')
    await Promise.all(tokensWithImages.map(t => loadTokenImage(t.id)))
  }

  // Get cached token image
  function getTokenImage(tokenId: string): string | undefined {
    return tokenImages.value.get(tokenId)
  }

  return {
    // State
    tokens,
    tokenSummaries,
    tokenImages,
    loading,
    error,
    // Computed
    visibleTokens,
    tokensByType,
    // Methods
    loadTokens,
    loadTokenSummaries,
    createToken,
    updateToken,
    updateTokenPosition,
    toggleVisibility,
    deleteToken,
    getTokenDisplaySize,
    getTokenColor,
    loadTokenImage,
    loadAllTokenImages,
    getTokenImage
  }
}
