import { formatEntries } from '../utils/textFormatting'

export function formatPsionicDetails(psionic: any): string {
  let html = '<div class="psionic-details">'
  
  // Header info
  html += '<div class="psionic-header">'
  
  // Type and Order
  const typeDisplay = psionic.psionic_type === 'D' ? 'Discipline' : 
                      psionic.psionic_type === 'T' ? 'Talent' : psionic.psionic_type
  html += `<div class="psionic-type">${typeDisplay}`
  if (psionic.order) {
    html += ` <span class="order">(${psionic.order})</span>`
  }
  html += '</div>'
  
  // Source
  if (psionic.source) {
    html += `<div class="source">Source: ${psionic.source}`
    if (psionic.page) {
      html += `, p. ${psionic.page}`
    }
    html += '</div>'
  }
  
  html += '</div>'
  
  // Main description
  if (psionic.entries && psionic.entries.length > 0) {
    html += '<div class="psionic-description">'
    html += formatEntries(psionic.entries)
    html += '</div>'
  }
  
  // Focus (for disciplines)
  if (psionic.focus) {
    html += '<div class="psionic-focus">'
    html += '<strong>Psychic Focus:</strong> '
    html += formatEntries([psionic.focus])
    html += '</div>'
  }
  
  // Modes (for disciplines)
  if (psionic.modes && psionic.modes.length > 0) {
    html += '<div class="psionic-modes">'
    html += '<h3>Psionic Modes</h3>'
    
    for (const mode of psionic.modes) {
      html += '<div class="mode-item">'
      
      // Mode name and cost
      html += '<div class="mode-header">'
      html += `<strong>${mode.name}</strong>`
      
      // Psi cost
      if (mode.cost) {
        let costStr = `${mode.cost.min} psi`
        if (mode.cost.max && mode.cost.max !== mode.cost.min) {
          costStr = `${mode.cost.min}-${mode.cost.max} psi`
        }
        html += ` <span class="psi-cost">(${costStr})`
        
        // Concentration
        if (mode.concentration) {
          const duration = formatConcentrationDuration(mode.concentration)
          html += `; conc., ${duration}`
        }
        
        html += ')</span>'
      }
      html += '</div>'
      
      // Mode entries
      if (mode.entries && mode.entries.length > 0) {
        html += '<div class="mode-description">'
        html += formatEntries(mode.entries)
        html += '</div>'
      }
      
      html += '</div>'
    }
    
    html += '</div>'
  }
  
  html += '</div>'
  
  return html
}

function formatConcentrationDuration(conc: { duration: number, unit: string }): string {
  const value = conc.duration
  const unit = conc.unit
  
  if (value === 1) {
    switch (unit) {
      case 'min': return '1 minute'
      case 'hr': return '1 hour'
      case 'round': return '1 round'
      default: return `${value} ${unit}`
    }
  } else {
    switch (unit) {
      case 'min': return `${value} minutes`
      case 'hr': return `${value} hours`
      case 'round': return `${value} rounds`
      default: return `${value} ${unit}`
    }
  }
}