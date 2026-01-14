import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { PrintService, type PrintResult, type ApiResponse } from '../PrintService'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  save: vi.fn()
}))

const mockInvoke = vi.mocked(invoke)
const mockSave = vi.mocked(save)

// Mock window.open
const mockWindowOpen = vi.fn()
const originalWindowOpen = window.open

// Mock atob for base64 decoding
const mockAtob = vi.fn((str: string) => {
  // Return a simple mock binary string
  return 'mock-binary-content'
})
const originalAtob = window.atob

// Mock URL.createObjectURL and revokeObjectURL
const mockCreateObjectURL = vi.fn(() => 'blob:mock-url')
const mockRevokeObjectURL = vi.fn()
const originalCreateObjectURL = URL.createObjectURL
const originalRevokeObjectURL = URL.revokeObjectURL

// Helper to create a mock PrintResult
function createMockPrintResult(overrides: Partial<PrintResult> = {}): PrintResult {
  return {
    pdf_base64: 'bW9jay1wZGYtY29udGVudA==', // "mock-pdf-content" in base64
    size_bytes: 1024,
    ...overrides
  }
}

describe('PrintService', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    window.open = mockWindowOpen
    window.atob = mockAtob
    URL.createObjectURL = mockCreateObjectURL
    URL.revokeObjectURL = mockRevokeObjectURL
  })

  afterEach(() => {
    vi.restoreAllMocks()
    window.open = originalWindowOpen
    window.atob = originalAtob
    URL.createObjectURL = originalCreateObjectURL
    URL.revokeObjectURL = originalRevokeObjectURL
  })

  describe('listTemplates', () => {
    it('returns list of available templates', async () => {
      const mockTemplates = [
        { id: 'character/sheet', name: 'Character Sheet', category: 'character' },
        { id: 'character/summary', name: 'Character Summary', category: 'character' },
        { id: 'spell/card', name: 'Spell Card', category: 'spell' }
      ]
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockTemplates })

      const result = await PrintService.listTemplates()

      expect(mockInvoke).toHaveBeenCalledWith('list_print_templates')
      expect(result).toEqual(mockTemplates)
      expect(result).toHaveLength(3)
    })

    it('throws error when API returns failure', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false, error: 'Template list not available' })

      await expect(PrintService.listTemplates())
        .rejects.toThrow('Template list not available')
    })

    it('throws error when data is missing', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true })

      await expect(PrintService.listTemplates())
        .rejects.toThrow('Failed to list templates')
    })
  })

  describe('generateCharacterSheet', () => {
    it('generates character sheet with default options', async () => {
      const mockResult = createMockPrintResult()
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockResult })

      const result = await PrintService.generateCharacterSheet(1)

      expect(mockInvoke).toHaveBeenCalledWith('generate_character_sheet', {
        characterId: 1,
        template: undefined,
        includeSpellCards: true
      })
      expect(result).toEqual(mockResult)
    })

    it('generates character sheet with specific template', async () => {
      const mockResult = createMockPrintResult()
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockResult })

      const result = await PrintService.generateCharacterSheet(1, 'summary')

      expect(mockInvoke).toHaveBeenCalledWith('generate_character_sheet', {
        characterId: 1,
        template: undefined,
        includeSpellCards: true
      })
      expect(result).toEqual(mockResult)
    })

    it('generates character sheet without spell cards', async () => {
      const mockResult = createMockPrintResult()
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockResult })

      await PrintService.generateCharacterSheet(1, 'sheet', false)

      expect(mockInvoke).toHaveBeenCalledWith('generate_character_sheet', {
        characterId: 1,
        template: 'character/sheet',
        includeSpellCards: false
      })
    })

    it('throws error on failure', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false, error: 'Character not found' })

      await expect(PrintService.generateCharacterSheet(999))
        .rejects.toThrow('Character not found')
    })
  })

  describe('generateCharacterExport', () => {
    it('exports character with default options', async () => {
      const mockResult = createMockPrintResult()
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockResult })

      const result = await PrintService.generateCharacterExport(1)

      expect(mockInvoke).toHaveBeenCalledWith('export_character', {
        characterId: 1,
        options: undefined
      })
      expect(result).toEqual(mockResult)
    })

    it('exports character with specific options', async () => {
      const mockResult = createMockPrintResult()
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockResult })

      const options = {
        include_compact_sheet: true,
        include_spell_cards: true,
        include_equipment_cards: false
      }
      await PrintService.generateCharacterExport(1, options)

      expect(mockInvoke).toHaveBeenCalledWith('export_character', {
        characterId: 1,
        options
      })
    })

    it('throws error on failure', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false, error: 'Export failed' })

      await expect(PrintService.generateCharacterExport(1))
        .rejects.toThrow('Export failed')
    })
  })

  describe('exportCampaignDocument', () => {
    it('exports single campaign document', async () => {
      const mockResult = createMockPrintResult()
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockResult })

      const result = await PrintService.exportCampaignDocument(42)

      expect(mockInvoke).toHaveBeenCalledWith('export_campaign_document', {
        documentId: 42
      })
      expect(result).toEqual(mockResult)
    })

    it('throws error on failure', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false, error: 'Document not found' })

      await expect(PrintService.exportCampaignDocument(999))
        .rejects.toThrow('Document not found')
    })
  })

  describe('exportCampaignDocuments', () => {
    it('exports all campaign documents with default options', async () => {
      const mockResult = createMockPrintResult()
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockResult })

      const result = await PrintService.exportCampaignDocuments(1)

      expect(mockInvoke).toHaveBeenCalledWith('export_campaign_documents', {
        campaignId: 1,
        options: undefined
      })
      expect(result).toEqual(mockResult)
    })

    it('exports campaign documents with specific options', async () => {
      const mockResult = createMockPrintResult()
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockResult })

      const options = {
        include_campaign_docs: true,
        include_module_content: true,
        include_npcs: false
      }
      await PrintService.exportCampaignDocuments(1, options)

      expect(mockInvoke).toHaveBeenCalledWith('export_campaign_documents', {
        campaignId: 1,
        options
      })
    })

    it('throws error on failure', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false, error: 'Campaign export failed' })

      await expect(PrintService.exportCampaignDocuments(1))
        .rejects.toThrow('Campaign export failed')
    })
  })

  describe('exportModuleDocuments', () => {
    it('exports module documents with default options', async () => {
      const mockResult = createMockPrintResult()
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockResult })

      const result = await PrintService.exportModuleDocuments(5)

      expect(mockInvoke).toHaveBeenCalledWith('export_module_documents', {
        moduleId: 5,
        options: undefined
      })
      expect(result).toEqual(mockResult)
    })

    it('exports module documents with specific options', async () => {
      const mockResult = createMockPrintResult()
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockResult })

      const options = {
        include_documents: true,
        include_monsters: true,
        include_traps: false,
        include_preview: true
      }
      await PrintService.exportModuleDocuments(5, options)

      expect(mockInvoke).toHaveBeenCalledWith('export_module_documents', {
        moduleId: 5,
        options
      })
    })

    it('throws error on failure', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false, error: 'Module export failed' })

      await expect(PrintService.exportModuleDocuments(5))
        .rejects.toThrow('Module export failed')
    })
  })

  describe('printMap', () => {
    it('prints map with default options', async () => {
      const mockResult = createMockPrintResult()
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockResult })

      const result = await PrintService.printMap(10)

      expect(mockInvoke).toHaveBeenCalledWith('print_map', {
        mapId: 10,
        options: undefined
      })
      expect(result).toEqual(mockResult)
    })

    it('prints map with specific options', async () => {
      const mockResult = createMockPrintResult()
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockResult })

      const options = {
        include_preview: true,
        preview_grid: true,
        include_play: false,
        play_cutouts: false
      }
      await PrintService.printMap(10, options)

      expect(mockInvoke).toHaveBeenCalledWith('print_map', {
        mapId: 10,
        options
      })
    })

    it('throws error on failure', async () => {
      mockInvoke.mockResolvedValueOnce({ success: false, error: 'Map print failed' })

      await expect(PrintService.printMap(10))
        .rejects.toThrow('Map print failed')
    })
  })

  describe('pdfToBlob', () => {
    it('converts base64 PDF to Blob', () => {
      const printResult = createMockPrintResult()

      const blob = PrintService.pdfToBlob(printResult)

      expect(mockAtob).toHaveBeenCalledWith(printResult.pdf_base64)
      expect(blob).toBeInstanceOf(Blob)
      expect(blob.type).toBe('application/pdf')
    })
  })

  describe('createPdfUrl', () => {
    it('creates object URL from PrintResult', () => {
      const printResult = createMockPrintResult()

      const url = PrintService.createPdfUrl(printResult)

      expect(url).toBe('blob:mock-url')
      expect(mockCreateObjectURL).toHaveBeenCalled()
    })
  })

  describe('savePdf', () => {
    it('saves PDF to file system', async () => {
      const printResult = createMockPrintResult()
      mockSave.mockResolvedValueOnce('/path/to/saved.pdf')
      mockInvoke.mockResolvedValueOnce(undefined)

      const result = await PrintService.savePdf(printResult, 'character.pdf')

      expect(mockSave).toHaveBeenCalledWith({
        defaultPath: 'character.pdf',
        filters: [{ name: 'PDF', extensions: ['pdf'] }]
      })
      expect(mockInvoke).toHaveBeenCalledWith('save_pdf', {
        pdfBase64: printResult.pdf_base64,
        path: '/path/to/saved.pdf'
      })
      expect(result).toBe('/path/to/saved.pdf')
    })

    it('uses default filename when not specified', async () => {
      const printResult = createMockPrintResult()
      mockSave.mockResolvedValueOnce('/path/to/document.pdf')
      mockInvoke.mockResolvedValueOnce(undefined)

      await PrintService.savePdf(printResult)

      expect(mockSave).toHaveBeenCalledWith({
        defaultPath: 'document.pdf',
        filters: [{ name: 'PDF', extensions: ['pdf'] }]
      })
    })

    it('returns null when user cancels save dialog', async () => {
      const printResult = createMockPrintResult()
      mockSave.mockResolvedValueOnce(null)

      const result = await PrintService.savePdf(printResult)

      expect(result).toBeNull()
      expect(mockInvoke).not.toHaveBeenCalledWith('save_pdf', expect.anything())
    })
  })

  describe('openPdf', () => {
    it('opens PDF in new browser window', async () => {
      const printResult = createMockPrintResult()

      await PrintService.openPdf(printResult)

      expect(mockWindowOpen).toHaveBeenCalledWith('blob:mock-url', '_blank')
    })
  })

  describe('printPdf', () => {
    it('creates iframe and triggers print dialog', () => {
      vi.useFakeTimers()

      const printResult = createMockPrintResult()
      const mockPrint = vi.fn()
      const mockIframe = {
        style: { display: '' },
        src: '',
        onload: null as (() => void) | null,
        contentWindow: { print: mockPrint }
      }

      const mockAppendChild = vi.fn()
      const mockRemoveChild = vi.fn()
      const mockCreateElement = vi.spyOn(document, 'createElement').mockReturnValue(mockIframe as any)
      document.body.appendChild = mockAppendChild
      document.body.removeChild = mockRemoveChild

      PrintService.printPdf(printResult)

      expect(mockCreateElement).toHaveBeenCalledWith('iframe')
      expect(mockIframe.style.display).toBe('none')
      expect(mockIframe.src).toBe('blob:mock-url')
      expect(mockAppendChild).toHaveBeenCalledWith(mockIframe)

      // Simulate iframe load
      mockIframe.onload?.()

      expect(mockPrint).toHaveBeenCalled()

      // Fast-forward to cleanup timeout
      vi.advanceTimersByTime(1000)

      expect(mockRemoveChild).toHaveBeenCalledWith(mockIframe)
      expect(mockRevokeObjectURL).toHaveBeenCalledWith('blob:mock-url')

      vi.useRealTimers()
    })
  })

  describe('error handling', () => {
    it('handles network errors gracefully', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network error'))

      await expect(PrintService.listTemplates())
        .rejects.toThrow('Network error')
    })

    it('handles missing data in response', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true, data: null })

      await expect(PrintService.generateCharacterSheet(1))
        .rejects.toThrow('Failed to generate character sheet')
    })
  })
})
