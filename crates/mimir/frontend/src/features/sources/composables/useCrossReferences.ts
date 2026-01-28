// Composable for managing cross-references and tooltips

import { ref } from 'vue'
import type { Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ReferenceData, ReferenceType } from '@/types/reference'
import type { TooltipPosition, ModalContent } from '@/types/content'
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

  // Map reference types to their backend command names (exact lookup)
  const refTypeToCommand: Record<string, string> = {
    spell: 'get_spell_by_name',
    item: 'get_item_by_name',
    creature: 'get_monster_by_name',
    race: 'get_race_by_name',
    class: 'get_class_by_name',
    classFeature: 'get_class_feature',
    subclass: 'get_subclass_by_name',
    subclassFeature: 'get_subclass_feature',
    background: 'get_background_by_name',
    feat: 'get_feat_by_name',
    condition: 'get_condition_by_name',
    action: 'get_action_by_name',
    language: 'get_language_by_name',
    trap: 'get_trap_by_name',
    hazard: 'get_hazard_by_name',
  }

  // Map reference types to their search command names (fallback)
  const refTypeToSearchCommand: Record<string, string> = {
    spell: 'search_spells',
    item: 'search_items',
    creature: 'search_monsters',
    race: 'search_races',
    class: 'search_classes',
    background: 'search_backgrounds',
    feat: 'search_feats',
    condition: 'search_conditions',
    action: 'search_actions',
    language: 'search_languages',
    trap: 'search_traps',
    hazard: 'search_hazards',
  }

  // Lookup reference data from backend
  async function lookupReference(refType: string, refName: string, refSource?: string, className?: string): Promise<any> {
    const cacheKey = `${refType}:${refName}:${refSource || ''}:${className || ''}`

    // Check cache first
    if (referenceCache.has(cacheKey)) {
      return referenceCache.get(cacheKey)
    }

    // Get the command for this reference type
    const command = refTypeToCommand[refType]
    if (!command) {
      console.warn(`No backend command for reference type: ${refType}`)
      return null
    }

    // Special handling for classFeature - uses className instead of source
    if (refType === 'classFeature' && className) {
      try {
        const response = await invoke<{ success: boolean; data?: any; error?: string }>(command, {
          name: refName,
          className: className
        })

        if (response.success && response.data) {
          const result = {
            name: response.data.name || refName,
            data: response.data,
            preview: null
          }
          referenceCache.set(cacheKey, result)
          return result
        }
      } catch (error) {
        console.error(`Failed to lookup class feature "${refName}":`, error)
      }
      return null
    }

    // Special handling for subclass - uses className + source
    if (refType === 'subclass' && className && refSource) {
      try {
        const response = await invoke<{ success: boolean; data?: any; error?: string }>(command, {
          name: refName,
          className: className,
          source: refSource
        })

        if (response.success && response.data) {
          const result = {
            name: response.data.name || refName,
            data: response.data,
            preview: null
          }
          referenceCache.set(cacheKey, result)
          return result
        }
      } catch (error) {
        console.error(`Failed to lookup subclass "${refName}":`, error)
      }
      return null
    }

    // Special handling for subclassFeature - uses subclassName + subclassSource
    if (refType === 'subclassFeature' && className && refSource) {
      try {
        // className here is actually subclassName for subclass features
        const response = await invoke<{ success: boolean; data?: any; error?: string }>(command, {
          name: refName,
          subclassName: className,
          subclassSource: refSource
        })

        if (response.success && response.data) {
          const result = {
            name: response.data.name || refName,
            data: response.data,
            preview: null
          }
          referenceCache.set(cacheKey, result)
          return result
        }
      } catch (error) {
        console.error(`Failed to lookup subclass feature "${refName}":`, error)
      }
      return null
    }

    // Try exact lookup first (name + source)
    if (refSource) {
      try {
        const response = await invoke<{ success: boolean; data?: any; error?: string }>(command, {
          name: refName,
          source: refSource
        })

        if (response.success && response.data) {
          const result = {
            name: response.data.name || refName,
            data: response.data,
            preview: null
          }
          referenceCache.set(cacheKey, result)
          return result
        }
      } catch (error) {
        // Exact lookup failed, will try search fallback
      }
    }

    // Fallback: search by name only
    const searchCommand = refTypeToSearchCommand[refType]
    if (searchCommand) {
      try {
        const searchResponse = await invoke<{ success: boolean; data?: any[]; error?: string }>(searchCommand, {
          filter: { name_contains: refName },
          limit: 5,
          offset: 0
        })

        if (searchResponse.success && searchResponse.data && searchResponse.data.length > 0) {
          // Try to find exact name match first, otherwise use first result
          const exactMatch = searchResponse.data.find(
            (item: any) => item.name?.toLowerCase() === refName.toLowerCase()
          )
          const found = exactMatch || searchResponse.data[0]
          const result = {
            name: found.name || refName,
            data: found,
            preview: null
          }
          referenceCache.set(cacheKey, result)
          return result
        }
      } catch (error) {
        console.error(`Failed to search ${refType} "${refName}":`, error)
      }
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
    const className = target.dataset.className

    if (!refType || !refName) return

    // Show loading tooltip
    tooltipContent.value = 'Loading...'
    tooltipVisible.value = true
    tooltipPosition.value = { x: event.clientX, y: event.clientY }

    // Lookup reference data
    const refData = await lookupReference(refType, refName, refSource, className)
    
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
    const className = target.dataset.className

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
    const refData = await lookupReference(refType, refName, refSource, className)
    
    if (refData) {
      // The backend returns { name, data, preview }
      // We need to pass the inner data object to formatModalContent
      const contentData = refData.data || refData
      modalContent.value = {
        title: refData.name || refName,
        content: await formatModalContent(refType as ReferenceType, contentData),
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
      case 'classFeature': {
        const className = data.className || data.class_name || ''
        const level = data.level
        if (className && level) {
          return `${data.name} (${className} Level ${level})`
        }
        return data.name
      }
      case 'subclass': {
        const className = data.className || data.class_name || ''
        if (className) {
          return `${data.name} (${className})`
        }
        return data.name
      }
      case 'subclassFeature': {
        const subclassName = data.subclassShortName || data.subclass_short_name || ''
        const level = data.level
        if (subclassName && level) {
          return `${data.name} (${subclassName} Level ${level})`
        }
        return data.name
      }
      case 'background':
        return data.name
      case 'feat':
        return data.name
      default:
        return data.name || 'Unknown'
    }
  }

  // Format modal content based on reference type
  async function formatModalContent(type: ReferenceType, data: any): Promise<string> {
    // Add the ref_type to the data for the renderer
    const contentWithType = {
      ...data,
      ref_type: type
    }
    return await renderModalContent(contentWithType)
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