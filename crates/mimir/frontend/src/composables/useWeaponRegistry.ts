/**
 * Weapon registry composable.
 *
 * Loads weapon names from the catalog at startup and provides a reactive
 * `isWeapon()` check. Falls back to the static PHB list in characterUtils
 * until the catalog loads.
 */

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { isWeapon as staticIsWeapon } from '@/utils/characterUtils'

/** Lowercase weapon names from the catalog */
const catalogWeaponNames = ref<Set<string> | null>(null)
const loaded = ref(false)
const loading = ref(false)

/**
 * Load weapon names from the catalog. Safe to call multiple times —
 * only the first call actually fetches.
 */
async function loadWeaponNames(): Promise<void> {
  if (loaded.value || loading.value) return

  loading.value = true
  try {
    const result = await invoke<{ success: boolean; data: string[] }>('list_weapon_names')
    if (result.success && result.data) {
      catalogWeaponNames.value = new Set(result.data.map((n) => n.toLowerCase()))
    }
    loaded.value = true
  } catch {
    // Catalog not available — static fallback will be used
  } finally {
    loading.value = false
  }
}

/**
 * Check if an item name is a weapon. Uses the catalog if loaded,
 * otherwise falls back to the static PHB weapon list.
 */
function isWeapon(itemName: string): boolean {
  if (catalogWeaponNames.value) {
    const name = itemName.toLowerCase()
    if (catalogWeaponNames.value.has(name)) return true
    for (const weapon of catalogWeaponNames.value) {
      if (name.startsWith(weapon) || name.endsWith(weapon)) return true
    }
    return false
  }
  return staticIsWeapon(itemName)
}

/**
 * Use the weapon registry. Call `loadWeaponNames()` once at app startup
 * or when the character sheet mounts.
 */
export function useWeaponRegistry() {
  return {
    isWeapon,
    loadWeaponNames,
    loaded,
  }
}
