// Composable for managing cross-references and tooltips

import { ref } from 'vue'
import type { Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ReferenceData, ReferenceType } from '../../../types/reference'
import type { TooltipPosition, ModalContent } from '../../../types/content'
import { renderModalContent } from '../formatters/modalFormatters'

export function useCrossReferences() {
  // Tooltip state
  const tooltipContent = ref('')
  const tooltipVisible = ref(false)
  const tooltipPosition: Ref<TooltipPosition> = ref({ x: 0, y: 0 })
  
  // Modal state
  const modalContent: Ref<ModalContent> = ref({
    title: '',
    content: '',
    visible: false
  })
  
  // Cache for reference lookups
  const referenceCache = new Map<string, any>()

  // Lookup reference data from backend
  async function lookupReference(refType: string, refName: string, refSource?: string): Promise<any> {
    const cacheKey = `${refType}:${refName}:${refSource || ''}`
    
    // Check cache first
    if (referenceCache.has(cacheKey)) {
      return referenceCache.get(cacheKey)
    }
    
    try {
      const response = await invoke<ReferenceData>('lookup_reference', {
        refType,
        refName,
        refSource
      })
      
      if (response.success && response.data) {
        // Cache the result
        referenceCache.set(cacheKey, response.data)
        return response.data
      }
    } catch (error) {
    }
    
    return null
  }

  // Handle cross-reference hover
  async function handleCrossRefHover(event: MouseEvent) {
    const target = event.target as HTMLElement
    if (!target.classList.contains('cross-ref-link')) return
    
    const refType = target.dataset.refType
    const refName = target.dataset.refName
    const refSource = target.dataset.refSource
    
    if (!refType || !refName) return
    
    // Show loading tooltip
    tooltipContent.value = 'Loading...'
    tooltipVisible.value = true
    tooltipPosition.value = { x: event.clientX, y: event.clientY }
    
    // Lookup reference data
    const refData = await lookupReference(refType, refName, refSource)
    
    if (refData) {
      // Check if there's a preview field first
      if (refData.preview) {
        tooltipContent.value = refData.preview
      } else {
        // Format tooltip content based on type
        const contentData = refData.data || refData
        tooltipContent.value = formatTooltipContent(refType as ReferenceType, contentData)
      }
    } else {
      tooltipContent.value = `${refName} (no data available)`
    }
  }

  // Handle cross-reference click
  async function handleCrossRefClick(event: MouseEvent) {
    const target = event.target as HTMLElement
    if (!target.classList.contains('cross-ref-link')) return
    
    event.preventDefault()
    event.stopPropagation()
    
    const refType = target.dataset.refType
    const refName = target.dataset.refName
    const refSource = target.dataset.refSource
    
    if (!refType || !refName) return
    
    // Hide tooltip when opening modal
    hideTooltip()
    
    // Show loading modal
    modalContent.value = {
      title: refName,
      content: '<p>Loading...</p>',
      visible: true
    }
    
    // Lookup reference data
    const refData = await lookupReference(refType, refName, refSource)
    
    if (refData) {
      // The backend returns { name, data, preview }
      // We need to pass the inner data object to formatModalContent
      const contentData = refData.data || refData
      modalContent.value = {
        title: refData.name || refName,
        content: formatModalContent(refType as ReferenceType, contentData),
        visible: true
      }
    } else {
      modalContent.value = {
        title: refName,
        content: '<p>No data available for this reference.</p>',
        visible: true
      }
    }
  }

  // Hide tooltip
  function hideTooltip() {
    tooltipVisible.value = false
    tooltipContent.value = ''
  }

  // Close modal
  function closeModal() {
    modalContent.value.visible = false
  }

  // Format tooltip content based on reference type
  function formatTooltipContent(type: ReferenceType, data: any): string {
    switch (type) {
      case 'spell':
        return `${data.name} - Level ${data.level || 0} ${data.school || 'Unknown'}`
      case 'item':
        return `${data.name}${data.rarity ? ` (${data.rarity})` : ''}`
      case 'creature':
        return `${data.name} - CR ${data.cr || '0'}`
      case 'condition':
        return data.name
      case 'race':
        return data.name
      case 'class':
        return data.name
      case 'background':
        return data.name
      case 'feat':
        return data.name
      default:
        return data.name || 'Unknown'
    }
  }

  // Format modal content based on reference type
  function formatModalContent(type: ReferenceType, data: any): string {
    // Add the ref_type to the data for the renderer
    const contentWithType = {
      ...data,
      ref_type: type
    }
    return renderModalContent(contentWithType)
  }

  // Clear cache
  function clearCache() {
    referenceCache.clear()
  }

  return {
    // Tooltip
    tooltipContent,
    tooltipVisible,
    tooltipPosition,
    
    // Modal
    modalContent,
    
    // Methods
    lookupReference,
    handleCrossRefHover,
    handleCrossRefClick,
    hideTooltip,
    closeModal,
    clearCache
  }
}