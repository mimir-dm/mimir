/**
 * Dev-only Tauri IPC shim for the UI harness (MIMIR-I-0074).
 *
 * When the app runs in a plain browser (Playwright, Chrome) instead of the
 * Tauri webview, `window.__TAURI_INTERNALS__` does not exist and every
 * `invoke()` from @tauri-apps/api would throw. This module — imported first
 * in main.ts — installs a minimal replacement that forwards commands over
 * HTTP to the ui-bridge binary (`cargo run -p mimir --features ui-harness
 * --bin ui-bridge`), which dispatches them through the real backend against
 * a scratch copy of the database.
 *
 * Guarded so it is inert everywhere that matters:
 * - production builds: `import.meta.env.DEV` is false → tree-shaken no-op
 * - the real Tauri app: `__TAURI_INTERNALS__` already exists → untouched
 *
 * Known gaps (documented, harness-unsupported):
 * - plugin commands (dialog, shell, event) are stubbed: event listeners
 *   register but never fire; dialogs reject with a console warning
 * - convertFileSrc returns the raw path — asset-protocol images don't render
 */

const BRIDGE_URL =
  (import.meta.env.VITE_BRIDGE_URL as string | undefined) ?? 'http://127.0.0.1:4175'

function installBridgeShim(): void {
  let nextCallbackId = 1
  const callbacks = new Map<number, (payload: unknown) => void>()

  async function bridgeInvoke(cmd: string, args: Record<string, unknown> = {}): Promise<unknown> {
    if (cmd.startsWith('plugin:')) {
      // Core/plugin IPC has no HTTP equivalent; stub the common ones.
      if (cmd === 'plugin:event|listen' || cmd === 'plugin:event|unlisten') {
        return null
      }
      // Native save dialog: pretend the user accepted the suggested name;
      // the actual write is turned into a browser download (see save_pdf).
      if (cmd === 'plugin:dialog|save') {
        const options = (args as { options?: { defaultPath?: string } })?.options ?? {}
        return options.defaultPath ?? 'download.pdf'
      }
      console.warn(`[bridge-shim] unsupported plugin command in browser harness: ${cmd}`)
      return Promise.reject(new Error(`${cmd} is not available in the browser harness`))
    }

    // In the desktop app save_pdf writes to the dialog-chosen path; in a
    // browser the equivalent UX is a download.
    if (cmd === 'save_pdf') {
      const { pdfBase64, path } = args as { pdfBase64: string; path: string }
      const bytes = Uint8Array.from(atob(pdfBase64), (c) => c.charCodeAt(0))
      const url = URL.createObjectURL(new Blob([bytes], { type: 'application/pdf' }))
      const anchor = document.createElement('a')
      anchor.href = url
      anchor.download = String(path).split('/').pop() || 'document.pdf'
      anchor.click()
      setTimeout(() => URL.revokeObjectURL(url), 10_000)
      return { success: true, data: path }
    }
    const response = await fetch(`${BRIDGE_URL}/invoke/${cmd}`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(args ?? {}),
    })
    const text = await response.text()
    let data: unknown = null
    try {
      data = text ? JSON.parse(text) : null
    } catch {
      data = text
    }
    if (!response.ok) {
      return Promise.reject(data)
    }
    return data
  }

  ;(window as any).__TAURI_INTERNALS__ = {
    invoke: bridgeInvoke,
    transformCallback(callback?: (payload: unknown) => void): number {
      const id = nextCallbackId++
      if (callback) callbacks.set(id, callback)
      return id
    },
    convertFileSrc(filePath: string, _protocol = 'asset'): string {
      return filePath
    },
    metadata: {
      currentWindow: { label: 'main' },
      currentWebview: { label: 'main', windowLabel: 'main' },
    },
    plugins: {},
  }

  console.info(`[bridge-shim] Tauri IPC forwarded to ${BRIDGE_URL} (UI harness mode)`)
}

if (import.meta.env.DEV && !('__TAURI_INTERNALS__' in window)) {
  installBridgeShim()
}

export {}
