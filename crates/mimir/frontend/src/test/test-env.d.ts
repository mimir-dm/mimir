// Type declarations for test environment
declare global {
  interface Window {
    __TAURI_INTERNALS__?: {
      invoke: any
    }
  }
}

export {}