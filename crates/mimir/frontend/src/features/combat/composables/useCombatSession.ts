import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { CombatEntry, CombatState } from '../types'

interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

/**
 * A module's combat, driven through the combat commands (COLLIERY-I-0468).
 * All rules (order, rounds, HP, conditions) live in the backend; this keeps
 * the latest state and reports errors.
 */
export function useCombatSession(moduleId: string) {
  const state = ref<CombatState | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const round = computed(() => state.value?.session.round ?? null)
  const currentEntry = computed<CombatEntry | null>(() => {
    const s = state.value
    if (!s?.current_entry_id) return null
    return s.entries.find((e) => e.id === s.current_entry_id) ?? null
  })

  /** Run a command; on failure keep the state and set `error`. */
  async function call<T>(command: string, args: Record<string, unknown>): Promise<{ ok: boolean; data?: T }> {
    loading.value = true
    try {
      const response = await invoke<ApiResponse<T>>(command, args)
      if (!response.success) {
        error.value = response.error ?? `${command} failed`
        return { ok: false }
      }
      error.value = null
      return { ok: true, data: response.data }
    } catch (e) {
      error.value = String(e)
      return { ok: false }
    } finally {
      loading.value = false
    }
  }

  const sessionId = () => state.value?.session.id

  async function refresh() {
    const id = sessionId()
    if (!id) return
    const r = await call<CombatState>('get_combat', { sessionId: id })
    if (r.ok && r.data) state.value = r.data
  }

  /** Load the module's active combat (or none). */
  async function load() {
    const r = await call<CombatState | null>('get_active_combat', { moduleId })
    if (r.ok) state.value = r.data ?? null
  }

  /** Resume the active combat or start one. */
  async function start() {
    const r = await call<CombatState>('start_combat', { moduleId })
    if (r.ok && r.data) state.value = r.data
  }

  async function end() {
    const id = sessionId()
    if (!id) return
    const r = await call<null>('end_combat', { sessionId: id })
    if (r.ok) state.value = null
  }

  async function turn(command: 'combat_next_turn' | 'combat_previous_turn') {
    const id = sessionId()
    if (!id) return
    const r = await call<CombatState>(command, { sessionId: id })
    if (r.ok && r.data) state.value = r.data
  }

  const nextTurn = () => turn('combat_next_turn')
  const previousTurn = () => turn('combat_previous_turn')

  /** Add a module monster group (default: its quantity). */
  async function addMonsterGroup(moduleMonsterId: string, count: number | null = null) {
    const id = sessionId()
    if (!id) return
    const r = await call('add_combat_module_monster', { sessionId: id, moduleMonsterId, count })
    if (r.ok) await refresh()
  }

  async function addCharacter(characterId: string) {
    const id = sessionId()
    if (!id) return
    const r = await call('add_combat_character', { sessionId: id, characterId })
    if (r.ok) await refresh()
  }

  async function addCustom(name: string, maxHp: number | null = null, initiative: number | null = null) {
    const id = sessionId()
    if (!id) return
    const r = await call('add_combat_custom', { sessionId: id, name, maxHp, initiative })
    if (r.ok) await refresh()
  }

  /** Set (or clear, with null) an entry's initiative; the order is refreshed. */
  async function setInitiative(entryId: string, initiative: number | null) {
    const r = await call('set_combat_initiative', { entryId, initiative })
    if (r.ok) await refresh()
  }

  async function removeEntry(entryId: string) {
    const r = await call<CombatState>('remove_combat_entry', { entryId })
    if (r.ok && r.data) state.value = r.data
  }

  return {
    state,
    loading,
    error,
    round,
    currentEntry,
    load,
    start,
    end,
    refresh,
    nextTurn,
    previousTurn,
    addMonsterGroup,
    addCharacter,
    addCustom,
    setInitiative,
    removeEntry,
  }
}
