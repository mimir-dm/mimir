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

export interface SpellPrintOptions {
  title?: string
  show_description?: boolean
  show_cut_lines?: boolean
}

export interface MonsterPrintOptions {
  title?: string
  notes?: string
  show_cut_lines?: boolean
}

export interface SessionPrintData {
  title?: string
  module?: string
  session_number?: number
  date?: string
  summary?: string
  npcs?: Array<{
    name: string
    role?: string
    notes?: string
  }>
  locations?: Array<{
    name: string
    type?: string
    notes?: string
  }>
  encounters?: Array<{
    name: string
    type?: string
    difficulty?: 'easy' | 'medium' | 'hard' | 'deadly'
    monsters?: Array<{ name: string; count: number }>
    notes?: string
  }>
  items?: Array<{
    name: string
    description?: string
  }>
  hooks?: string[]
  secrets?: string[]
  notes?: string[]
}

export interface NpcData {
  name: string
  race?: string
  occupation?: string
  role?: string
  alignment?: string
  location?: string
  appearance?: string
  personality?: string
  mannerisms?: string
  voice?: string
  goal?: string
  motivation?: string
  bond?: string
  flaw?: string
  secret?: string
  key_info?: string
}

export interface HandoutData {
  title: string
  subtitle?: string
  type?: string
  author?: string
  date?: string
  style?: 'default' | 'aged' | 'formal' | 'magical'
  body?: string | string[]
  sections?: Array<{
    title?: string
    content: string
  }>
  footer?: string
}

/** Map print mode */
export type MapPrintMode = 'preview' | 'play'

/** Options for printing a map */
export interface MapPrintOptions {
  /** Print mode: preview (fit to page) or play (1"=5ft scale) */
  mode?: MapPrintMode
  /** Show grid overlay on the map */
  show_grid?: boolean
  /** Show LOS wall segments as red lines */
  show_los_walls?: boolean
  /** Show starting positions as numbered circles (instead of tokens) */
  show_positions?: boolean
  /** Include token cutout sheets for printing */
  include_cutouts?: boolean
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
   * Generate a PDF from any template with custom data
   */
  async generatePdf(templateId: string, data: Record<string, unknown>): Promise<PrintResult> {
    const response = await invoke<ApiResponse<PrintResult>>('generate_pdf', {
      templateId,
      data
    })

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to generate PDF')
    }

    return response.data
  }

  /**
   * Generate a character sheet PDF
   * @param characterId - The ID of the character
   * @param template - Template variant to use ('sheet' or 'summary')
   * @param includeSpellCards - Whether to include spell cards in the PDF (default: true)
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
   * Generate spell PDF (card, list, or multi-up cards)
   */
  async generateSpellPdf(
    template: 'card' | 'list' | 'cards-multiup',
    spells: Record<string, unknown>[],
    options?: SpellPrintOptions
  ): Promise<PrintResult> {
    const response = await invoke<ApiResponse<PrintResult>>('generate_spell_pdf', {
      template,
      spells,
      options
    })

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to generate spell PDF')
    }

    return response.data
  }

  /**
   * Generate monster PDF (stat-block, card, encounter, or multi-up cards)
   */
  async generateMonsterPdf(
    template: 'stat-block' | 'card' | 'encounter' | 'cards-multiup',
    monsters: Record<string, unknown>[],
    options?: MonsterPrintOptions
  ): Promise<PrintResult> {
    const response = await invoke<ApiResponse<PrintResult>>('generate_monster_pdf', {
      template,
      monsters,
      options
    })

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to generate monster PDF')
    }

    return response.data
  }

  /**
   * Generate session prep sheet
   */
  async generateSessionPrep(data: SessionPrintData): Promise<PrintResult> {
    const response = await invoke<ApiResponse<PrintResult>>('generate_session_pdf', {
      template: 'prep',
      data
    })

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to generate session prep sheet')
    }

    return response.data
  }

  /**
   * Generate NPC card(s)
   */
  async generateNpcCards(
    npcs: NpcData[],
    multiUp: boolean = false
  ): Promise<PrintResult> {
    const template = multiUp ? 'npc-cards-multiup' : 'npc-card'
    const data = multiUp ? { npcs, show_cut_lines: true } : npcs[0]

    const response = await invoke<ApiResponse<PrintResult>>('generate_session_pdf', {
      template,
      data
    })

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to generate NPC cards')
    }

    return response.data
  }

  /**
   * Generate a player handout
   */
  async generateHandout(data: HandoutData): Promise<PrintResult> {
    const response = await invoke<ApiResponse<PrintResult>>('generate_session_pdf', {
      template: 'handout',
      data
    })

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to generate handout')
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
   */
  async exportCampaignDocuments(campaignId: number): Promise<PrintResult> {
    const response = await invoke<ApiResponse<PrintResult>>('export_campaign_documents', {
      campaignId
    })

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to export campaign documents')
    }

    return response.data
  }

  /**
   * Export a single module's documents and monsters as PDF
   * @param moduleId - The ID of the module
   */
  async exportModuleDocuments(moduleId: number): Promise<PrintResult> {
    const response = await invoke<ApiResponse<PrintResult>>('export_module_documents', {
      moduleId
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
