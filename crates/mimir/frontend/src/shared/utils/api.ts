/**
 * API Utilities for Tauri Commands
 *
 * Provides helper functions for working with ApiResponse wrappers from the backend.
 */

import type { ApiResponse } from '@/types/api'

/**
 * Error thrown when an API call returns an error response.
 */
export class ApiError extends Error {
  constructor(message: string) {
    super(message)
    this.name = 'ApiError'
  }
}

/**
 * Unwrap an ApiResponse, returning the data or throwing an error.
 *
 * Use this to convert the ApiResponse wrapper from Tauri commands into
 * a simple value that can be used directly.
 *
 * @param response The ApiResponse wrapper from a Tauri command
 * @returns The data from the response
 * @throws ApiError if the response indicates failure
 *
 * @example
 * ```typescript
 * const response = await invoke<ApiResponse<Campaign[]>>('list_campaigns')
 * const campaigns = unwrapResponse(response)
 * // campaigns is now Campaign[], not ApiResponse<Campaign[]>
 * ```
 */
export function unwrapResponse<T>(response: ApiResponse<T>): T {
  if (response.success && response.data !== undefined) {
    return response.data
  }
  throw new ApiError(response.error || 'Unknown API error')
}

/**
 * Safely unwrap an ApiResponse, returning undefined instead of throwing.
 *
 * Use this when you want to handle errors gracefully without try/catch.
 *
 * @param response The ApiResponse wrapper from a Tauri command
 * @returns The data from the response, or undefined if the response indicates failure
 *
 * @example
 * ```typescript
 * const response = await invoke<ApiResponse<Campaign>>('get_campaign', { id })
 * const campaign = unwrapResponseSafe(response)
 * if (!campaign) {
 *   // Handle missing campaign
 * }
 * ```
 */
export function unwrapResponseSafe<T>(response: ApiResponse<T>): T | undefined {
  if (response.success && response.data !== undefined) {
    return response.data
  }
  return undefined
}

/**
 * Check if an ApiResponse indicates success.
 *
 * @param response The ApiResponse wrapper from a Tauri command
 * @returns true if the response indicates success
 */
export function isSuccess<T>(response: ApiResponse<T>): boolean {
  return response.success && response.data !== undefined
}

/**
 * Get the error message from an ApiResponse.
 *
 * @param response The ApiResponse wrapper from a Tauri command
 * @returns The error message, or undefined if the response indicates success
 */
export function getError<T>(response: ApiResponse<T>): string | undefined {
  if (!response.success) {
    return response.error || 'Unknown error'
  }
  return undefined
}
