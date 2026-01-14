import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import AppModal from '../AppModal.vue'

describe('AppModal', () => {
  // Store original body overflow style
  let originalOverflow: string

  beforeEach(() => {
    originalOverflow = document.body.style.overflow
    vi.clearAllMocks()
  })

  afterEach(() => {
    // Restore body overflow
    document.body.style.overflow = originalOverflow
    // Clean up any teleported content
    document.body.innerHTML = ''
  })

  describe('visibility', () => {
    it('does not render content when visible is false', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: false,
          title: 'Test Modal'
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      expect(wrapper.find('.modal-overlay').exists()).toBe(false)
    })

    it('renders content when visible is true', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test Modal'
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      expect(wrapper.find('.modal-overlay').exists()).toBe(true)
    })
  })

  describe('title and header', () => {
    it('renders title in header', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'My Modal Title'
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      expect(wrapper.find('.modal-title').text()).toBe('My Modal Title')
    })

    it('renders custom header slot content', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true
        },
        slots: {
          header: '<h1 class="custom-header">Custom Header</h1>'
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      expect(wrapper.find('.custom-header').text()).toBe('Custom Header')
    })

    it('sets aria-labelledby on dialog', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test'
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      const dialog = wrapper.find('[role="dialog"]')
      expect(dialog.attributes('aria-labelledby')).toBeTruthy()
    })
  })

  describe('close button', () => {
    it('renders close button when closable is true (default)', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test'
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      expect(wrapper.find('.modal-close').exists()).toBe(true)
    })

    it('does not render close button when closable is false', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test',
          closable: false
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      expect(wrapper.find('.modal-close').exists()).toBe(false)
    })

    it('emits close event when close button is clicked', async () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test'
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      await wrapper.find('.modal-close').trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
      expect(wrapper.emitted('update:visible')).toBeTruthy()
      expect(wrapper.emitted('update:visible')![0]).toEqual([false])
    })
  })

  describe('overlay click', () => {
    it('closes modal when overlay is clicked and closeOnOverlay is true (default)', async () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test'
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      await wrapper.find('.modal-overlay').trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('does not close modal when overlay is clicked and closeOnOverlay is false', async () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test',
          closeOnOverlay: false
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      await wrapper.find('.modal-overlay').trigger('click')

      expect(wrapper.emitted('close')).toBeFalsy()
    })

    it('does not close when content area is clicked', async () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test'
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      await wrapper.find('.modal-content').trigger('click')

      expect(wrapper.emitted('close')).toBeFalsy()
    })
  })

  describe('sizes', () => {
    const sizes = ['sm', 'md', 'lg', 'xl', 'full'] as const

    it.each(sizes)('applies %s size class correctly', (size) => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test',
          size
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      expect(wrapper.find('.modal-content').classes()).toContain(`modal-${size}`)
    })

    it('defaults to md size', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test'
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      expect(wrapper.find('.modal-content').classes()).toContain('modal-md')
    })
  })

  describe('body content', () => {
    it('renders default slot in modal body', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test'
        },
        slots: {
          default: '<p class="test-content">Modal content here</p>'
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      expect(wrapper.find('.modal-body .test-content').text()).toBe('Modal content here')
    })

    it('applies no-padding class when noPadding is true', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test',
          noPadding: true
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      expect(wrapper.find('.modal-body').classes()).toContain('no-padding')
    })
  })

  describe('footer', () => {
    it('renders footer slot when provided', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test'
        },
        slots: {
          footer: '<button class="test-btn">Save</button>'
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      expect(wrapper.find('.modal-footer').exists()).toBe(true)
      expect(wrapper.find('.test-btn').text()).toBe('Save')
    })

    it('does not render footer when slot is not provided', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test'
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      expect(wrapper.find('.modal-footer').exists()).toBe(false)
    })
  })

  describe('accessibility', () => {
    it('sets role="dialog" on modal', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test'
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      expect(wrapper.find('[role="dialog"]').exists()).toBe(true)
    })

    it('sets aria-modal="true"', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test'
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      expect(wrapper.find('[aria-modal="true"]').exists()).toBe(true)
    })

    it('close button has aria-label', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test'
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      expect(wrapper.find('.modal-close').attributes('aria-label')).toBe('Close modal')
    })
  })

  describe('stacking', () => {
    it('applies stacked class when stackIndex > 0', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test',
          stackIndex: 1
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      expect(wrapper.find('.modal-overlay').classes()).toContain('modal-stacked')
    })

    it('sets CSS variable for stack index', () => {
      const wrapper = mount(AppModal, {
        props: {
          visible: true,
          title: 'Test',
          stackIndex: 2
        },
        global: {
          stubs: {
            Teleport: true
          }
        }
      })

      const overlay = wrapper.find('.modal-overlay')
      expect(overlay.attributes('style')).toContain('--stack-index: 2')
    })
  })
})
