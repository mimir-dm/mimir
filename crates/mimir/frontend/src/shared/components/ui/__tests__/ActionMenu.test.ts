/**
 * ActionMenu: the "More actions" menu (COLLIERY-I-0466, MIMIR-T-0690).
 */
import { describe, it, expect, vi, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ActionMenu from '../ActionMenu.vue'

function mountMenu() {
  const onDelete = vi.fn()
  const onRename = vi.fn()
  const wrapper = mount(ActionMenu, {
    props: {
      label: 'More module actions',
      items: [
        { label: 'Rename', onSelect: onRename },
        { label: 'Delete module…', danger: true, onSelect: onDelete },
      ],
    },
    attachTo: document.body,
  })
  return { wrapper, onDelete, onRename }
}

const trigger = (w: ReturnType<typeof mountMenu>['wrapper']) => w.get('[data-testid="action-menu-trigger"]')

describe('ActionMenu', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('is closed until the button is used; the button names the menu', () => {
    const { wrapper } = mountMenu()
    expect(wrapper.find('[role="menu"]').exists()).toBe(false)
    expect(trigger(wrapper).attributes('aria-label')).toBe('More module actions')
    expect(trigger(wrapper).attributes('aria-expanded')).toBe('false')
  })

  it('opens with the first item focused; arrows move; Esc closes and refocuses the button', async () => {
    const { wrapper } = mountMenu()
    await trigger(wrapper).trigger('click')
    await flushPromises()
    expect(trigger(wrapper).attributes('aria-expanded')).toBe('true')
    const items = wrapper.findAll('[role="menuitem"]')
    expect(document.activeElement).toBe(items[0].element)
    await wrapper.get('.action-menu').trigger('keydown', { key: 'ArrowDown' })
    await flushPromises()
    expect(document.activeElement).toBe(items[1].element)
    await wrapper.get('.action-menu').trigger('keydown', { key: 'ArrowDown' })
    await flushPromises()
    expect(document.activeElement).toBe(items[0].element)
    await wrapper.get('.action-menu').trigger('keydown', { key: 'Escape' })
    expect(wrapper.find('[role="menu"]').exists()).toBe(false)
    expect(document.activeElement).toBe(trigger(wrapper).element)
  })

  it('runs the chosen action and closes; danger items are marked', async () => {
    const { wrapper, onDelete } = mountMenu()
    await trigger(wrapper).trigger('click')
    const del = wrapper.findAll('[role="menuitem"]')[1]
    expect(del.classes()).toContain('danger')
    await del.trigger('click')
    expect(onDelete).toHaveBeenCalledOnce()
    expect(wrapper.find('[role="menu"]').exists()).toBe(false)
  })

  it('closes on a click outside', async () => {
    const { wrapper } = mountMenu()
    await trigger(wrapper).trigger('click')
    document.body.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    await flushPromises()
    expect(wrapper.find('[role="menu"]').exists()).toBe(false)
  })
})
