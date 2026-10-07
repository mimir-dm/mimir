/**
 * App dialogs instead of the browser's native confirm() and alert()
 * (MIMIR-T-0684). Call `useDialog().confirm(...)` or `.alert(...)` from any
 * component or composable; the `DialogHost` of the window shows the dialog
 * with AppModal and settles the promise.
 *
 * Each window root that can show a dialog mounts one `DialogHost` (main app,
 * DM map, sources). Without a host (a window that has none), the service
 * falls back to the native dialogs so that a caller never waits forever.
 */
import { readonly, ref } from 'vue'

export interface ConfirmOptions {
  title: string
  message: string
  /** Label of the confirm button (default "OK"). */
  confirmLabel?: string
  /** Label of the cancel button (default "Cancel"). */
  cancelLabel?: string
  /** A destructive action: the confirm button is red. */
  danger?: boolean
}

export interface AlertOptions {
  title: string
  message: string
  /** Label of the button (default "OK"). */
  okLabel?: string
}

export type PendingDialog =
  | { id: number; kind: 'confirm'; options: ConfirmOptions; resolve: (ok: boolean) => void }
  | { id: number; kind: 'alert'; options: AlertOptions; resolve: (ok: boolean) => void }

const queue = ref<PendingDialog[]>([])
let nextId = 1
let hosts = 0

/** Dialogs waiting to be shown, first one on screen. */
export const pendingDialogs = readonly(queue)

/** Settle a dialog and take it off the queue (DialogHost calls this). */
export function settleDialog(id: number, ok: boolean): void {
  const dialog = queue.value.find((d) => d.id === id)
  if (!dialog) return
  queue.value = queue.value.filter((d) => d.id !== id)
  dialog.resolve(ok)
}

/** A DialogHost mounted (returns the function that unregisters it). */
export function registerDialogHost(): () => void {
  hosts++
  return () => {
    hosts--
  }
}

/** Ask the user to confirm; resolves true on confirm, false on cancel. */
export function confirmDialog(options: ConfirmOptions): Promise<boolean> {
  if (hosts === 0) return Promise.resolve(window.confirm(`${options.title}\n\n${options.message}`))
  return new Promise((resolve) => {
    queue.value = [...queue.value, { id: nextId++, kind: 'confirm', options, resolve }]
  })
}

/** Tell the user something; resolves when the dialog is closed. */
export function alertDialog(options: AlertOptions): Promise<void> {
  if (hosts === 0) {
    window.alert(`${options.title}\n\n${options.message}`)
    return Promise.resolve()
  }
  return new Promise((resolve) => {
    queue.value = [...queue.value, { id: nextId++, kind: 'alert', options, resolve: () => resolve() }]
  })
}

export function useDialog() {
  return { confirm: confirmDialog, alert: alertDialog }
}
