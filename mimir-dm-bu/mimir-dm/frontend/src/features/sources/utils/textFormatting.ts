/**
 * Process D&D 5e tools formatting tags and convert them to HTML
 */
export function processFormattingTags(text: string | any): string {
  if (!text) return ''
  
  // Convert to string if not already
  let processed = typeof text === 'string' ? text : String(text)
  
  // Basic formatting
  processed = processed
    // Bold
    .replace(/{@b ([^}]+)}/g, '<strong>$1</strong>')
    // Italic
    .replace(/{@i ([^}]+)}/g, '<em>$1</em>')
    // Bold-Italic
    .replace(/{@bi ([^}]+)}/g, '<strong><em>$1</em></strong>')
    
  // Dice rolls - format them nicely
  processed = processed
    .replace(/{@dice ([^}]+)}/g, (match, diceContent) => {
      // Handle both simple {@dice 1d6} and complex {@dice 1d6|display text} formats
      const parts = diceContent.split('|')
      const dice = parts[0].trim()
      const display = parts[1]?.trim() || dice
      return `<span class="dice-roll">${display}</span>`
    })
    .replace(/{@damage ([^|}]+)(?:\|([^}]*))?}/g, (match, damage, display) => {
      const text = display || damage
      return `<span class="damage-roll">${text}</span>`
    })
    .replace(/{@d20 ([^|}]+)(?:\|([^}]*))?}/g, (match, modifier, display) => {
      const text = display || `d20${modifier}`
      return `<span class="d20-roll">${text}</span>`
    })
    .replace(/{@hit ([^}]+)}/g, '<span class="hit-bonus">$1</span>')
    
  // Conditions and status - make conditions clickable
  processed = processed
    .replace(/{@condition ([^|}]+)(?:\|([^}]*))?}/gi, (match, name, source) => {
      const displayName = name
      const actualSource = source || 'PHB'
      return `<a href="#" class="cross-ref-link condition-ref" data-ref-type="condition" data-ref-name="${name}" data-ref-source="${actualSource}">${displayName}</a>`
    })
    .replace(/{@status ([^|}]+)(?:\|[^}]*)?}/gi, '<span class="status">$1</span>')
    
  // Spells - make them clickable
  processed = processed
    .replace(/{@spell ([^|}]+)(?:\|([^}]*))?}/gi, (match, name, source) => {
      const displayName = name
      const actualSource = source || 'PHB'
      return `<a href="#" class="cross-ref-link spell-ref" data-ref-type="spell" data-ref-name="${name}" data-ref-source="${actualSource}">${displayName}</a>`
    })
    
  // Items - make them clickable
  processed = processed
    .replace(/{@item ([^|}]+)(?:\|([^|}]+))?(?:\|([^}]*))?}/gi, (match, name, source, displayText) => {
      const displayName = displayText || name
      const actualSource = source || 'PHB'
      return `<a href="#" class="cross-ref-link item-ref" data-ref-type="item" data-ref-name="${name}" data-ref-source="${actualSource}">${displayName}</a>`
    })
    
  // Item entry references - these reference the description of another item
  // Special handling for known item groups
  processed = processed
    .replace(/{#itemEntry Armor of Resistance(?:\|[^}]*)?}/gi, 
      '<span class="item-description">You have resistance to one type of damage while you wear this armor.</span>')
    .replace(/{#itemEntry Potion of Resistance(?:\|[^}]*)?}/gi,
      '<span class="item-description">When you drink this potion, you gain resistance to one type of damage for 1 hour.</span>')
    .replace(/{#itemEntry Grenade(?:\|[^}]*)?}/gi,
      '<span class="item-description">As an action, a character can throw a grenade at a point up to 60 feet away. Each creature within 20 feet of an exploding fragmentation grenade must make a DC 15 Dexterity saving throw, taking 5d6 piercing damage on a failed save, or half as much damage on a successful one.</span>')
    // Generic fallback for other item entry references
    .replace(/{#itemEntry ([^|}]+)(?:\|[^}]*)?}/gi, '<span class="item-entry-ref">[See base item: $1]</span>')
    
  // Creatures - make them clickable
  processed = processed
    .replace(/{@creature ([^|}]+)(?:\|([^|}]+))?(?:\|([^}]*))?}/gi, (match, name, source, displayText) => {
      const displayName = displayText || name
      const actualSource = source || 'MM'
      return `<a href="#" class="cross-ref-link creature-ref" data-ref-type="creature" data-ref-name="${name}" data-ref-source="${actualSource}">${displayName}</a>`
    })
    
  // Races - make them clickable
  processed = processed
    .replace(/{@race ([^|}]+)(?:\|([^|}]+))?(?:\|([^}]*))?}/gi, (match, name, source, displayText) => {
      const displayName = displayText || name
      const actualSource = source || 'PHB'
      return `<a href="#" class="cross-ref-link race-ref" data-ref-type="race" data-ref-name="${name}" data-ref-source="${actualSource}">${displayName}</a>`
    })
    
  // Classes and features - make them clickable
  processed = processed
    .replace(/{@class ([^|}]+)(?:\|([^|}]+))?(?:\|([^}]*))?}/gi, (match, name, source, displayText) => {
      const displayName = displayText || name
      const actualSource = source || 'PHB'
      return `<a href="#" class="cross-ref-link class-ref" data-ref-type="class" data-ref-name="${name}" data-ref-source="${actualSource}">${displayName}</a>`
    })
    .replace(/{@classFeature ([^|}]+)(?:\|([^}]*))?}/gi, (match, name, source) => {
      const displayName = name
      const actualSource = source || 'PHB'
      return `<a href="#" class="cross-ref-link feature-ref" data-ref-type="feature" data-ref-name="${name}" data-ref-source="${actualSource}">${displayName}</a>`
    })

  // Feats - make them clickable
  processed = processed
    .replace(/{@feat ([^|}]+)(?:\|([^|}]+))?(?:\|([^}]*))?}/gi, (match, name, source, displayText) => {
      const displayName = displayText || name
      const actualSource = source || 'PHB'
      return `<a href="#" class="cross-ref-link feat-ref" data-ref-type="feat" data-ref-name="${name}" data-ref-source="${actualSource}">${displayName}</a>`
    })

  // Backgrounds - make them clickable
  processed = processed
    .replace(/{@background ([^|}]+)(?:\|([^|}]+))?(?:\|([^}]*))?}/gi, (match, name, source, displayText) => {
      const displayName = displayText || name
      const actualSource = source || 'PHB'
      return `<a href="#" class="cross-ref-link background-ref" data-ref-type="background" data-ref-name="${name}" data-ref-source="${actualSource}">${displayName}</a>`
    })

  // Skills and abilities
  processed = processed
    .replace(/{@skill ([^|}]+)(?:\|[^}]*)?}/gi, '<span class="skill">$1</span>')
    .replace(/{@sense ([^|}]+)(?:\|[^}]*)?}/gi, '<span class="sense">$1</span>')
    
  // Actions - make them clickable
  processed = processed
    .replace(/{@action ([^|}]+)(?:\|([^}]*))?}/gi, (match, name, source) => {
      const displayName = name
      const actualSource = source || 'PHB'
      return `<a href="#" class="cross-ref-link action-ref" data-ref-type="action" data-ref-name="${name}" data-ref-source="${actualSource}">${displayName}</a>`
    })
    
  // Attack types
  processed = processed
    .replace(/{@atk mw}/gi, '<em>Melee Weapon Attack:</em>')
    .replace(/{@atk rw}/gi, '<em>Ranged Weapon Attack:</em>')
    .replace(/{@atk mw,rw}/gi, '<em>Melee or Ranged Weapon Attack:</em>')
    .replace(/{@atk ms}/gi, '<em>Melee Spell Attack:</em>')
    .replace(/{@atk rs}/gi, '<em>Ranged Spell Attack:</em>')
    
  // Hit bonus (the {@h} tag)
  processed = processed
    .replace(/{@h}/gi, '<em>Hit:</em>')
    
  // DC checks
  processed = processed
    .replace(/{@dc (\d+)(?:\|([^}]+))?}/gi, (match, dc, ability) => {
      return `<span class="dc-check">DC ${dc}${ability ? ' ' + ability : ''}</span>`
    })
    
  // Filters (complex references)
  processed = processed
    .replace(/{@filter ([^|}]+)(?:\|[^}]*)?}/gi, '<span class="filter-ref">$1</span>')
    
  // Books and sources
  processed = processed
    .replace(/{@book ([^|}]+)(?:\|[^}]*)?}/gi, '<span class="book-ref">$1</span>')
    
  // Chance
  processed = processed
    .replace(/{@chance (\d+)(?:\|([^}]+))?}/gi, (match, num, text) => {
      return `<span class="chance">${text || num + '% chance'}</span>`
    })
    
  // Recharge
  processed = processed
    .replace(/{@recharge\s*(\d+)?}/gi, (match, num) => {
      return `<span class="recharge">(Recharge${num ? ' ' + num + '–6' : ''})</span>`
    })
    
  // Note blocks
  processed = processed
    .replace(/{@note ([^}]+)}/gi, '<span class="note">Note: $1</span>')
    
  // Catch-all for any remaining tags we haven't handled
  processed = processed
    .replace(/{@\w+ ([^|}]+)(?:\|[^}]*)?}/gi, '<span class="tagged">$1</span>')
  
  return processed
}

/**
 * Format 5etools entries (nested content structures)
 */
export function formatEntries(entries: any[]): string {
  if (!entries || !Array.isArray(entries)) return ''
  
  let html = ''
  for (const entry of entries) {
    if (typeof entry === 'string') {
      html += `<p>${processFormattingTags(entry)}</p>`
    } else if (entry && typeof entry === 'object') {
      if (entry.type === 'entries') {
        if (entry.name) {
          html += `<h5>${entry.name}</h5>`
        }
        if (entry.entries) {
          html += formatEntries(entry.entries)
        }
      } else if (entry.type === 'list') {
        html += '<ul>'
        if (entry.items) {
          for (const item of entry.items) {
            if (typeof item === 'string') {
              html += `<li>${processFormattingTags(item)}</li>`
            } else {
              html += `<li>${formatEntries([item])}</li>`
            }
          }
        }
        html += '</ul>'
      } else if (entry.type === 'table') {
        html += formatTable(entry)
      } else if (entry.type === 'inset' || entry.type === 'insetReadaloud') {
        // In modals, treat all insets as read-aloud for consistent styling
        html += `<div class="inset-readaloud">`
        if (entry.name) {
          html += `<h5>${entry.name}</h5>`
        }
        if (entry.entries) {
          html += formatEntries(entry.entries)
        }
        html += '</div>'
      } else if (entry.entries) {
        html += formatEntries(entry.entries)
      } else if (entry.text) {
        html += `<p>${processFormattingTags(entry.text)}</p>`
      }
    }
  }
  return html
}

function formatTable(table: any): string {
  let html = '<table class="entry-table">'
  
  // Table caption
  if (table.caption) {
    html += `<caption>${table.caption}</caption>`
  }
  
  // Table headers
  if (table.colLabels) {
    html += '<thead><tr>'
    for (const label of table.colLabels) {
      html += `<th>${processFormattingTags(label)}</th>`
    }
    html += '</tr></thead>'
  }
  
  // Table rows
  if (table.rows) {
    html += '<tbody>'
    // Check if first column is a dice roll column
    const isDiceColumn = table.colLabels && table.colLabels[0]?.includes('{@dice')
    
    for (let i = 0; i < table.rows.length; i++) {
      const row = table.rows[i]
      html += '<tr>'
      if (Array.isArray(row)) {
        for (let j = 0; j < row.length; j++) {
          const cell = row[j]
          let cellContent = ''
          
          // If this is the first column and it's a dice column, and the cell is empty/undefined,
          // fill it with the row number
          if (j === 0 && isDiceColumn && (!cell || cell === '')) {
            cellContent = String(i + 1)
          } else if (typeof cell === 'string') {
            cellContent = processFormattingTags(cell)
          } else if (cell && typeof cell === 'object') {
            // Handle cell objects with roll property {"roll": {"exact": 1}, "type": "cell"}
            if (cell.type === 'cell' && cell.roll) {
              if (cell.roll.exact !== undefined) {
                cellContent = String(cell.roll.exact)
              } else if (cell.roll.min !== undefined && cell.roll.max !== undefined) {
                cellContent = `${cell.roll.min}-${cell.roll.max}`
              } else {
                cellContent = JSON.stringify(cell.roll)
              }
            } else {
              cellContent = formatEntries([cell])
            }
          }
          
          html += `<td>${cellContent}</td>`
        }
      } else {
        html += `<td>${processFormattingTags(row)}</td>`
      }
      html += '</tr>'
    }
    html += '</tbody>'
  }
  
  html += '</table>'
  return html
}