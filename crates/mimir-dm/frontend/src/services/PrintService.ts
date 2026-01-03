/**
 * Print Service - Frontend API for PDF generation
 *
 * Provides methods to generate PDFs from Typst templates via Tauri commands.
 */
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'

// Types
export interface PrintTemplateInfo {
  /** Template identifier (e.g., "character/sheet") */
  id: string
  /** Display name (e.g., "Character Sheet") */
  name: string
  /** Category (e.g., "character", "spell", "monster") */
  category: string
}

export interface PrintResult {
  /** Base64-encoded PDF data */
  pdf_base64: string
  /** Size of the PDF in bytes */
  size_bytes: number
}

export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

/** Options for printing a map */
export interface MapPrintOptions {
  // Preview section
  /** Include preview page (fit to single page) */
  include_preview?: boolean
  /** Show grid overlay on preview */
  preview_grid?: boolean
  /** Show LOS walls on preview */
  preview_los_walls?: boolean
  /** Show starting positions on preview */
  preview_positions?: boolean
  // Play section
  /** Include play tiles (1"=5ft scale) */
  include_play?: boolean
  /** Show grid overlay on tiles */
  play_grid?: boolean
  /** Show LOS walls on tiles */
  play_los_walls?: boolean
  /** Include token cutout sheets */
  play_cutouts?: boolean
}

/** Options for exporting a module to PDF */
export interface ModuleExportOptions {
  // Content section
  /** Include module documents and notes */
  include_documents?: boolean
  /** Include monster stat blocks for tagged monsters */
  include_monsters?: boolean
  /** Include campaign NPC sheets */
  include_npcs?: boolean
  // Map Preview section
  /** Include map previews (fit to single page) */
  include_preview?: boolean
  /** Show grid overlay on preview */
  preview_grid?: boolean
  /** Show LOS walls on preview */
  preview_los_walls?: boolean
  /** Show starting positions on preview */
  preview_positions?: boolean
  // Map Play section
  /** Include play tiles (1"=5ft scale) */
  include_play?: boolean
  /** Show grid overlay on tiles */
  play_grid?: boolean
  /** Show LOS walls on tiles */
  play_los_walls?: boolean
  /** Include token cutout sheets */
  play_cutouts?: boolean
}

/** Options for exporting a character to PDF */
export interface CharacterExportOptions {
  /** Include compact 2-page character sheet */
  include_compact_sheet?: boolean
  /** Include long form character details (personality, background, RP notes) */
  include_long_form?: boolean
  /** Include spell cards (silently no-op if no spells) */
  include_spell_cards?: boolean
  /** Include equipment cards (weapons, magic items, special ammo) */
  include_equipment_cards?: boolean
  /** Include detailed equipment list with descriptions */
  include_equipment_detail?: boolean
}

/** Options for exporting a campaign to PDF */
export interface CampaignExportOptions {
  // Reference Document options
  /** Include campaign-level documents */
  include_campaign_docs?: boolean
  /** Include module content (documents and monsters) */
  include_module_content?: boolean
  /** Include campaign NPC sheets */
  include_npcs?: boolean
  // Module Maps options (sub-options for module content)
  /** Include module map previews scaled to fit one page */
  include_module_map_previews?: boolean
  /** Include module maps at 1"=5ft scale for tabletop play */
  include_module_tiled_maps?: boolean
  /** Include printable paper standees for module tokens */
  include_token_cutouts?: boolean
  // Campaign Maps options
  /** Include campaign map previews scaled to fit one page */
  include_campaign_map_previews?: boolean
  /** Include campaign maps at 1"=5ft scale for tabletop play */
  include_campaign_tiled_maps?: boolean
}

class PrintServiceClass {
  /**
   * List all available print templates
   */
  async listTemplates(): Promise<PrintTemplateInfo[]> {
    const response = await invoke<ApiResponse<PrintTemplateInfo[]>>('list_print_templates')

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to list templates')
    }

    return response.data
  }

  /**
   * Generate a character sheet PDF
   * @param characterId - The ID of the character
   * @param template - Template variant to use ('sheet' or 'summary')
   * @param includeSpellCards - Whether to include spell cards in the PDF (default: true)
   * @deprecated Use generateCharacterExport instead for composable sections
   */
  async generateCharacterSheet(
    characterId: number,
    template: 'sheet' | 'summary' = 'sheet',
    includeSpellCards: boolean = true
  ): Promise<PrintResult> {
    const response = await invoke<ApiResponse<PrintResult>>('generate_character_sheet', {
      characterId,
      template: includeSpellCards ? undefined : `character/${template}`,
      includeSpellCards
    })

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to generate character sheet')
    }

    return response.data
  }

  /**
   * Export character to PDF with composable sections
   * @param characterId - The ID of the character
   * @param options - Export options for section selection
   */
  async generateCharacterExport(characterId: number, options?: CharacterExportOptions): Promise<PrintResult> {
    const response = await invoke<ApiResponse<PrintResult>>('export_character', {
      characterId,
      options
    })

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to export character')
    }

    return response.data
  }

  /**
   * Export a single campaign document to PDF
   * @param documentId - The ID of the campaign document
   */
  async exportCampaignDocument(documentId: number): Promise<PrintResult> {
    const response = await invoke<ApiResponse<PrintResult>>('export_campaign_document', {
      documentId
    })

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to export campaign document')
    }

    return response.data
  }

  /**
   * Export all campaign documents as a combined PDF
   * @param campaignId - The ID of the campaign
   * @param options - Export options for content selection
   */
  async exportCampaignDocuments(campaignId: number, options?: CampaignExportOptions): Promise<PrintResult> {
    const response = await invoke<ApiResponse<PrintResult>>('export_campaign_documents', {
      campaignId,
      options
    })

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to export campaign documents')
    }

    return response.data
  }

  /**
   * Export a single module's documents and monsters as PDF
   * @param moduleId - The ID of the module
   * @param options - Export options for content selection
   */
  async exportModuleDocuments(moduleId: number, options?: ModuleExportOptions): Promise<PrintResult> {
    const response = await invoke<ApiResponse<PrintResult>>('export_module_documents', {
      moduleId,
      options
    })

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to export module documents')
    }

    return response.data
  }

  /**
   * Print a map to PDF with configurable options
   * @param mapId - The ID of the map
   * @param options - Print options (mode, overlays, etc.)
   */
  async printMap(mapId: number, options?: MapPrintOptions): Promise<PrintResult> {
    const response = await invoke<ApiResponse<PrintResult>>('print_map', {
      mapId,
      options
    })

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to print map')
    }

    return response.data
  }

  /**
   * Convert base64 PDF to Blob for display/download
   */
  pdfToBlob(result: PrintResult): Blob {
    const binaryString = atob(result.pdf_base64)
    const bytes = new Uint8Array(binaryString.length)
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i)
    }
    return new Blob([bytes], { type: 'application/pdf' })
  }

  /**
   * Create a URL for displaying PDF in browser
   */
  createPdfUrl(result: PrintResult): string {
    const blob = this.pdfToBlob(result)
    return URL.createObjectURL(blob)
  }

  /**
   * Save PDF to file system using Tauri dialog
   */
  async savePdf(result: PrintResult, defaultFileName: string = 'document.pdf'): Promise<string | null> {
    const filePath = await save({
      defaultPath: defaultFileName,
      filters: [{
        name: 'PDF',
        extensions: ['pdf']
      }]
    })

    if (!filePath) {
      return null
    }

    await invoke('save_pdf', {
      pdfBase64: result.pdf_base64,
      path: filePath
    })

    return filePath
  }

  /**
   * Open PDF in system default viewer
   */
  async openPdf(result: PrintResult): Promise<void> {
    // Create a temporary file and open it
    const url = this.createPdfUrl(result)
    window.open(url, '_blank')
  }

  /**
   * Print PDF using browser print dialog
   */
  printPdf(result: PrintResult): void {
    const url = this.createPdfUrl(result)
    const iframe = document.createElement('iframe')
    iframe.style.display = 'none'
    iframe.src = url
    document.body.appendChild(iframe)

    iframe.onload = () => {
      iframe.contentWindow?.print()
      // Clean up after a delay to allow print dialog
      setTimeout(() => {
        document.body.removeChild(iframe)
        URL.revokeObjectURL(url)
      }, 1000)
    }
  }
}

export const PrintService = new PrintServiceClass()
