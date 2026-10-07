import { useCampaignStore } from '@/stores/campaigns'

/**
 * Campaign scope for catalog searches (MIMIR-T-0675).
 *
 * The backend decides which sources a search covers (CatalogSearch::
 * effective_sources): sources the user chose explicitly, else the campaign's
 * configured sources, else all. The frontend only sends the explicit choice
 * and the current campaign id.
 */

/** The sources the user chose, or null when none were chosen. */
export function explicitSources(filterSources: unknown): string[] | null {
  return Array.isArray(filterSources) && filterSources.length > 0
    ? (filterSources as string[])
    : null
}

/** The selected campaign's id, or null. Store access is lazy (Pinia may not be ready). */
export function currentCampaignId(): string | null {
  try {
    return useCampaignStore().currentCampaign?.id ?? null
  } catch {
    return null
  }
}
