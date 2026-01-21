// Composable for managing book navigation and table of contents

import type { BookSection, SubEntry } from '../../../types/book'

export function useBookNavigation() {
  
  // Get sub-entries from a section for TOC display (with nested structure)
  function getSubEntries(section: BookSection): SubEntry[] {
    const entries: SubEntry[] = []
    
    function processEntries(items: any[], level: number = 0): SubEntry[] {
      const result: SubEntry[] = []
      if (!items) return result
      
      items.forEach((entry, index) => {
        if (typeof entry === 'object' && entry !== null) {
          // Add named entries to the sub-entries list
          if (entry.name && (entry.type === 'section' || entry.type === 'entries')) {
            const subEntry: SubEntry = {
              id: entry.id || `entry-${index}`,
              name: entry.name,
              level
            }
            
            // Recursively process nested entries as children
            if (entry.entries && Array.isArray(entry.entries)) {
              const children = processEntries(entry.entries, level + 1)
              if (children.length > 0) {
                subEntry.children = children
              }
            }
            
            result.push(subEntry)
          }
        }
      })
      
      return result
    }
    
    if (section.entries && Array.isArray(section.entries)) {
      return processEntries(section.entries)
    }
    
    return entries
  }
  
  // Scroll to top of content area
  function scrollToTop() {
    const contentArea = document.querySelector('.content-panel')
    if (contentArea) {
      contentArea.scrollTop = 0
    }
  }
  
  // Scroll to a specific element by ID
  function scrollToElement(elementId: string, smooth: boolean = true) {
    const element = document.getElementById(elementId)
    if (element) {
      element.scrollIntoView({ 
        behavior: smooth ? 'smooth' : 'auto', 
        block: 'start' 
      })
    }
  }
  
  return {
    getSubEntries,
    scrollToTop,
    scrollToElement
  }
}