// Debug flag - set to true to enable IPC logging
const DEBUG_IPC = import.meta.env.DEV && typeof window !== 'undefined' && window.location.search.includes('debug=true')

// Log all document-related issues
export function debugDocument(action: string, data: any) {
  if (DEBUG_IPC) {
    console.group(`📄 Document Debug: ${action}`)
    console.log('Data:', data)
    console.trace('Call stack')
    console.groupEnd()
  }
}