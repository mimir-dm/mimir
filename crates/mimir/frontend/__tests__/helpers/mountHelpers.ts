/**
 * Vue component mount helpers for Vitest.
 *
 * Provides pre-configured mount/shallowMount wrappers with common
 * global plugins (Pinia, router stubs, Teleport stubs) so each test
 * doesn't have to repeat the same boilerplate.
 *
 * Usage:
 *   import { mountWithPlugins, shallowMountWithPlugins } from '@tests/helpers/mountHelpers'
 *
 *   const wrapper = mountWithPlugins(MyComponent, {
 *     props: { characterId: 1 },
 *   })
 */

import { mount, shallowMount, type MountingOptions, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import type { Component, DefineComponent } from 'vue'

// ─── Types ───────────────────────────────────────────────────────────────────

export interface MountOptions<Props = Record<string, unknown>> {
  props?: Props
  slots?: Record<string, string | Component>
  /** Additional global config merged into defaults */
  global?: MountingOptions<unknown>['global']
  /** Pre-configured Pinia instance (creates a fresh one if omitted) */
  pinia?: Pinia
  /** Additional stubs beyond the defaults */
  stubs?: Record<string, boolean | Component>
  /** Attach to document.body (needed for portal/teleport tests) */
  attachTo?: HTMLElement | string
}

// ─── Default global config ───────────────────────────────────────────────────

function buildGlobalConfig(options: MountOptions = {}): MountingOptions<unknown>['global'] {
  const pinia = options.pinia ?? createPinia()
  setActivePinia(pinia)

  const defaultStubs: Record<string, boolean | Component> = {
    RouterLink: true,
    RouterView: true,
    Teleport: true,
    // Common app-level components that shouldn't be rendered in unit tests
    AppModal: true,
  }

  const stubs = { ...defaultStubs, ...options.stubs }

  return {
    plugins: [pinia],
    stubs,
    ...options.global,
    // Merge stubs from options.global too
    ...(options.global?.stubs
      ? { stubs: { ...stubs, ...(options.global.stubs as Record<string, boolean>) } }
      : {}),
  }
}

// ─── Mount wrappers ──────────────────────────────────────────────────────────

/**
 * Full mount with Pinia, router stubs, and Teleport stubs pre-configured.
 */
export function mountWithPlugins<T extends Component>(
  component: T,
  options: MountOptions = {},
): VueWrapper {
  return mount(component as DefineComponent, {
    props: options.props,
    slots: options.slots as Record<string, unknown>,
    global: buildGlobalConfig(options),
    attachTo: options.attachTo,
  }) as VueWrapper
}

/**
 * Shallow mount with Pinia, router stubs, and Teleport stubs pre-configured.
 * Child components are stubbed automatically.
 */
export function shallowMountWithPlugins<T extends Component>(
  component: T,
  options: MountOptions = {},
): VueWrapper {
  return shallowMount(component as DefineComponent, {
    props: options.props,
    slots: options.slots as Record<string, unknown>,
    global: buildGlobalConfig(options),
    attachTo: options.attachTo,
  }) as VueWrapper
}

/**
 * Create a Pinia instance with pre-set initial state.
 * Useful for tests that need specific store values before mount.
 *
 * Usage:
 *   const pinia = createTestPinia({
 *     campaign: { currentCampaign: { id: '1', name: 'Test' } }
 *   })
 *   mountWithPlugins(MyComponent, { pinia })
 */
export function createTestPinia(initialState?: Record<string, unknown>): Pinia {
  const pinia = createPinia()
  setActivePinia(pinia)

  if (initialState) {
    // Pinia stores are initialized on first use, so we set state after creation
    pinia.state.value = initialState
  }

  return pinia
}
