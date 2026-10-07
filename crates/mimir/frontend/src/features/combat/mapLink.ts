/**
 * How the combat tracker shows on the map and the player display
 * (MIMIR-T-0680). Pure functions of the tracker state.
 */
import type { InitiativeUpdatePayload } from '@/composables/map/usePlayerDisplayEvents'
import type { CombatEntry, CombatState } from './types'

/** What the DM map marks for the fight. */
export interface CombatMapState {
  /** Token of the creature whose turn it is. */
  currentTurnTokenId: string | null
  /** Tokens of creatures at 0 HP. */
  downTokenIds: string[]
}

export function combatMapState(state: CombatState | null): CombatMapState {
  if (!state) return { currentTurnTokenId: null, downTokenIds: [] }
  const current = state.entries.find((e) => e.id === state.current_entry_id)
  return {
    currentTurnTokenId: current?.token_id ?? null,
    downTokenIds: state.entries
      .filter((e) => e.token_id && e.current_hp === 0)
      .map((e) => e.token_id as string),
  }
}

/** Entries that come from the module and so have a token on the map. */
export function isMapCreature(e: CombatEntry): boolean {
  return e.source_kind === 'module_monster' || e.source_kind === 'module_npc'
}

/**
 * The turn order the players may see: names only, no HP. A monster or NPC
 * shows only when its token is visible to the players on the current map, so
 * the order never gives away a creature the players have not seen.
 */
export function playerInitiativeOrder(
  state: CombatState,
  visibleTokenIds: readonly string[],
): InitiativeUpdatePayload {
  const visible = new Set(visibleTokenIds)
  return {
    visible: true,
    round: state.session.round,
    entries: state.entries
      .filter((e) => !isMapCreature(e) || (e.token_id !== null && visible.has(e.token_id)))
      .map((e) => ({ name: e.display_name, current: e.id === state.current_entry_id })),
  }
}

/** Hides the order on the player display. */
export const HIDDEN_ORDER: InitiativeUpdatePayload = { visible: false, round: null, entries: [] }
