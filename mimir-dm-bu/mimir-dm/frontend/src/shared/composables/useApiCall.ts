import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ApiResponse } from '../../types'

export interface ApiCallOptions {
  retryCount?: number
  retryDelay?: number
  onSuccess?: (data: any) => void
  onError?: (error: string) => void
}

export function useApiCall<T = any>() {
  const loading = ref(false)
  const error = ref<string | null>(null)
  const data = ref<T | null>(null)

  const execute = async (
    command: string,
    params?: any,
    options: ApiCallOptions = {}
  ): Promise<T | null> => {
    const { retryCount = 0, retryDelay = 1000, onSuccess, onError } = options

    loading.value = true
    error.value = null

    let attempts = 0
    while (attempts <= retryCount) {
      try {
        const response = await invoke<ApiResponse<T>>(command, params)
        
        if (response.success && response.data) {
          data.value = response.data
          if (onSuccess) {
            onSuccess(response.data)
          }
          return response.data
        } else {
          const errorMsg = response.error || `Failed to execute ${command}`
          error.value = errorMsg
          if (onError) {
            onError(errorMsg)
          }
          
          // Don't retry if the error is not network-related
          if (!errorMsg.includes('network') && !errorMsg.includes('timeout')) {
            break
          }
        }
      } catch (e) {
        const errorMsg = e instanceof Error ? e.message : 'Unknown error occurred'
        error.value = errorMsg
        
        if (attempts < retryCount) {
          await new Promise(resolve => setTimeout(resolve, retryDelay))
          attempts++
          continue
        }
        
        if (onError) {
          onError(errorMsg)
        }
        break
      }
    }
    
    loading.value = false
    return null
  }

  const reset = () => {
    loading.value = false
    error.value = null
    data.value = null
  }

  return {
    loading,
    error,
    data,
    execute,
    reset
  }
}