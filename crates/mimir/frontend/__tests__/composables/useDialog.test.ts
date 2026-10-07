/**
 * App dialogs instead of native confirm()/alert() (MIMIR-T-0684).
 */
import { describe, it, expect, afterEach, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import DialogHost from '@/shared/components/ui/DialogHost.vue'
import AppModal from '@/components/shared/AppModal.vue'
import { useDialog, pendingDialogs } from '@/composables/useDialog'

function mountHost() {
  // An open modal under the dialog, to check that Esc closes only the dialog.
  const onModalClose = vi.fn()
  const wrapper = mount(
    defineComponent({
      setup: () => () => [
        h(AppModal, { visible: true, title: 'Map setup', onClose: onModalClose }, () => 'content'),
        h(DialogHost),
      ],
    }),
    { attachTo: document.body },
  )
  return { wrapper, onModalClose }
}

const byTestId = (id: string) => document.body.querySelector<HTMLElement>(`[data-testid="${id}"]`)

describe('useDialog with a DialogHost', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('confirm resolves true on the confirm button, with a danger style when asked', async () => {
    const { wrapper } = mountHost()
    const answer = useDialog().confirm({ title: 'Delete map?', message: 'Delete "Cave"?', confirmLabel: 'Delete', danger: true })
    await flushPromises()
    expect(byTestId('dialog-message')!.textContent).toBe('Delete "Cave"?')
    const confirm = byTestId('dialog-confirm')!
    expect(confirm.textContent!.trim()).toBe('Delete')
    expect(confirm.classList).toContain('btn-danger')
    confirm.click()
    await expect(answer).resolves.toBe(true)
    expect(pendingDialogs.value).toHaveLength(0)
    wrapper.unmount()
  })

  it('confirm resolves false on cancel', async () => {
    const { wrapper } = mountHost()
    const answer = useDialog().confirm({ title: 'Sure?', message: 'Really?' })
    await flushPromises()
    expect(byTestId('dialog-confirm')!.classList).toContain('btn-primary')
    byTestId('dialog-cancel')!.click()
    await expect(answer).resolves.toBe(false)
    wrapper.unmount()
  })

  it('Esc cancels the dialog and leaves the modal under it open', async () => {
    const { wrapper, onModalClose } = mountHost()
    const answer = useDialog().confirm({ title: 'Sure?', message: 'Really?' })
    await flushPromises()
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await expect(answer).resolves.toBe(false)
    expect(onModalClose).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('shows dialogs one at a time, in order; alert resolves on OK', async () => {
    const { wrapper } = mountHost()
    const first = useDialog().alert({ title: 'Import failed', message: 'a.zip: bad archive' })
    const second = useDialog().alert({ title: 'Second', message: 'two' })
    await flushPromises()
    expect(byTestId('dialog-message')!.textContent).toBe('a.zip: bad archive')
    byTestId('dialog-ok')!.click()
    await expect(first).resolves.toBeUndefined()
    await flushPromises()
    expect(byTestId('dialog-message')!.textContent).toBe('two')
    byTestId('dialog-ok')!.click()
    await expect(second).resolves.toBeUndefined()
    wrapper.unmount()
  })
})

describe('useDialog without a DialogHost', () => {
  it('falls back to the native dialogs so a caller never waits forever', async () => {
    const native = vi.spyOn(window, 'confirm').mockReturnValue(true)
    await expect(useDialog().confirm({ title: 'T', message: 'M' })).resolves.toBe(true)
    expect(native).toHaveBeenCalledWith('T\n\nM')
    native.mockRestore()
  })
})
