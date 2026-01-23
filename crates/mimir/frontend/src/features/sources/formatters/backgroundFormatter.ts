import { processFormattingTags, formatEntries } from '../utils/textFormatting'

export async function formatBackgroundDetails(background: any): Promise<string> {
  if (!background) {
    return `<div class="error">Background details not available</div>`
  }

  const bg = background.background || background
  let content = ''
  
  // Add styles for tables
  content += `<style>
    .characteristic-table {
      width: 100%;
      border-collapse: collapse;
      margin: 1em 0;
    }
    .characteristic-table th,
    .characteristic-table td {
      padding: 0.5em;
      border: 1px solid var(--color-border, #333);
      text-align: left;
    }
    .characteristic-table th {
      background: var(--color-background-tertiary, #262626);
      font-weight: bold;
    }
    .characteristic-table .dice-column {
      width: 60px;
      text-align: center;
    }
    .characteristic-table .description-column {
      width: auto;
    }
    .characteristic-table tbody tr:nth-child(odd) {
      background: rgba(255, 255, 255, 0.02);
    }
  </style>`
  
  // Header with name and source
  content += `<div class="background-header">`
  content += `<h1>${bg.name}</h1>`
  content += `<div class="source-info">Source: ${bg.source}${bg.page ? `, p. ${bg.page}` : ''}</div>`
  content += `</div>`
  
  // Proficiencies section
  content += `<div class="proficiencies-section">`
  content += `<h2>Proficiencies</h2>`
  
  // Skills
  if (bg.skill_proficiencies?.length || bg.skills) {
    const skills = bg.skills || formatProficiencies(bg.skill_proficiencies, 'skill')
    content += `<p><strong>Skills:</strong> ${skills}</p>`
  } else if (bg.skillProficiencies?.length) {
    const skills = formatProficiencies(bg.skillProficiencies, 'skill')
    content += `<p><strong>Skills:</strong> ${skills}</p>`
  }
  
  // Languages
  if (bg.language_proficiencies?.length || bg.languages) {
    const languages = bg.languages || formatProficiencies(bg.language_proficiencies, 'language')
    content += `<p><strong>Languages:</strong> ${languages}</p>`
  } else if (bg.languageProficiencies?.length) {
    const languages = formatProficiencies(bg.languageProficiencies, 'language')
    content += `<p><strong>Languages:</strong> ${languages}</p>`
  }
  
  // Tools
  if (bg.tool_proficiencies?.length || bg.tools) {
    const tools = bg.tools || formatProficiencies(bg.tool_proficiencies, 'tool')
    content += `<p><strong>Tools:</strong> ${tools}</p>`
  } else if (bg.toolProficiencies?.length) {
    const tools = formatProficiencies(bg.toolProficiencies, 'tool')
    content += `<p><strong>Tools:</strong> ${tools}</p>`
  }
  
  content += `</div>`
  
  // Starting Equipment
  if (bg.starting_equipment?.length || bg.startingEquipment?.length) {
    const equipment = bg.starting_equipment || bg.startingEquipment
    content += `<div class="equipment-section">`
    content += `<h2>Starting Equipment</h2>`
    content += formatEquipment(equipment)
    content += `</div>`
  }
  
  // Features and Traits
  if (bg.entries?.length) {
    content += `<div class="features-section">`
    for (const entry of bg.entries) {
      content += await formatEntry(entry)
    }
    content += `</div>`
  }
  
  // Fluff content if available
  if (background.fluff?.entries?.length) {
    content += `<div class="fluff-section">`
    content += `<h2>Description</h2>`
    for (const entry of background.fluff.entries) {
      content += await formatEntry(entry)
    }
    content += `</div>`
  }
  
  return content
}

function formatProficiencies(proficiencies: any[], type: string): string {
  if (!proficiencies?.length) return 'None'
  
  const profs: string[] = []
  for (const prof of proficiencies) {
    if (typeof prof === 'string') {
      profs.push(prof)
    } else if (typeof prof === 'object') {
      // Handle various proficiency formats
      const keys = Object.keys(prof)
      for (const key of keys) {
        if (key === 'any' || key === 'anyStandard') {
          const count = prof[key]
          if (key === 'anyStandard') {
            profs.push(`Any ${count} standard ${type}${count > 1 ? 's' : ''}`)
          } else {
            profs.push(`Any ${count}`)
          }
        } else if (key === 'choose') {
          const choice = prof[key]
          if (choice.from && choice.count) {
            profs.push(`Choose ${choice.count} from: ${choice.from.join(', ')}`)
          }
        } else if (prof[key] === true) {
          // Format the key name nicely
          profs.push(key.charAt(0).toUpperCase() + key.slice(1))
        }
      }
    }
  }
  
  return profs.join(', ') || 'None'
}

// Format an equipment item reference as a cross-reference link
function formatEquipmentItem(eq: any): string {
  if (typeof eq === 'string') {
    // Check if it's an item reference in "name|source" format (common in equipment lists)
    // These need to be wrapped in {@item} tags before processing
    if (eq.includes('|') && !eq.includes('{@')) {
      // Looks like an item reference - wrap it
      return processFormattingTags(`{@item ${eq}}`)
    }
    // Plain string or already has tags - process as-is
    return processFormattingTags(eq)
  } else if (eq.item) {
    // Item reference - wrap in {@item} tag for cross-referencing
    // eq.item can be "itemName" or "itemName|source"
    const itemRef = eq.item
    const displayName = eq.displayName
    const quantity = eq.quantity

    // Build the {@item} tag
    let tag = `{@item ${itemRef}}`
    if (displayName && displayName !== itemRef.split('|')[0]) {
      // Use displayName if different from item name
      tag = `{@item ${itemRef}|${displayName}}`
    }

    // Process the tag to convert to link
    const processed = processFormattingTags(tag)

    // Add quantity prefix if present
    if (quantity && quantity > 1) {
      return `${quantity}x ${processed}`
    }
    return processed
  } else if (eq.special) {
    return processFormattingTags(eq.special)
  }
  return 'Unknown item'
}

function formatEquipment(equipment: any[]): string {
  if (!equipment?.length) return '<p>No starting equipment</p>'

  let content = '<ul>'
  for (const item of equipment) {
    if (typeof item === 'string') {
      // Check if it's an item reference in "name|source" format
      if (item.includes('|') && !item.includes('{@')) {
        content += `<li>${processFormattingTags(`{@item ${item}}`)}</li>`
      } else {
        content += `<li>${processFormattingTags(item)}</li>`
      }
    } else if (item._) {
      // Main equipment list
      content += '<li>'
      const items: string[] = []
      for (const eq of item._) {
        items.push(formatEquipmentItem(eq))
      }
      content += items.join(', ')
      content += '</li>'
    } else if (item.a || item.b) {
      // Choice between options
      content += '<li>Choose one:<ul>'
      if (item.a) {
        content += '<li>Option A: '
        const aItems = item.a.map((i: any) => formatEquipmentItem(i))
        content += aItems.join(', ')
        content += '</li>'
      }
      if (item.b) {
        content += '<li>Option B: '
        const bItems = item.b.map((i: any) => formatEquipmentItem(i))
        content += bItems.join(', ')
        content += '</li>'
      }
      content += '</ul></li>'
    }
  }
  content += '</ul>'

  return content
}

async function formatEntry(entry: any): Promise<string> {
  if (!entry) return ''
  
  if (typeof entry === 'string') {
    return `<p>${processFormattingTags(entry)}</p>`
  }
  
  let content = ''
  
  // Handle different entry types
  switch (entry.type) {
    case 'entries':
      if (entry.name) {
        // Check if it's a feature or suggested characteristics
        if (entry.name.startsWith('Feature:')) {
          content += `<h3 class="feature-header">${entry.name}</h3>`
        } else if (entry.name === 'Suggested Characteristics') {
          content += `<h2>${entry.name}</h2>`
        } else {
          content += `<h3>${entry.name}</h3>`
        }
      }
      if (entry.entries) {
        // Wrap characteristics tables in a div for better styling
        if (entry.name === 'Suggested Characteristics') {
          content += '<div class="characteristics-section">'
        }
        for (const subEntry of entry.entries) {
          content += await formatEntry(subEntry)
        }
        if (entry.name === 'Suggested Characteristics') {
          content += '</div>'
        }
      }
      break
      
    case 'list':
      content += '<ul>'
      if (entry.items) {
        for (const item of entry.items) {
          if (typeof item === 'string') {
            content += `<li>${processFormattingTags(item)}</li>`
          } else if (item.type === 'item') {
            content += '<li>'
            if (item.name) content += `<strong>${item.name}</strong> `
            if (item.entry) content += processFormattingTags(item.entry)
            content += '</li>'
          } else {
            content += `<li>${await formatEntry(item)}</li>`
          }
        }
      }
      content += '</ul>'
      break
      
    case 'table':
      content += '<table class="characteristic-table">'
      if (entry.colLabels) {
        content += '<thead><tr>'
        for (let i = 0; i < entry.colLabels.length; i++) {
          const label = entry.colLabels[i]
          // First column (usually dice) should be narrow
          const className = i === 0 ? 'dice-column' : 'description-column'
          content += `<th class="${className}">${label}</th>`
        }
        content += '</tr></thead>'
      }
      if (entry.rows) {
        content += '<tbody>'
        for (const row of entry.rows) {
          content += '<tr>'
          for (let i = 0; i < row.length; i++) {
            const cell = row[i]
            // First column gets special styling
            const className = i === 0 ? 'dice-column' : 'description-column'
            content += `<td class="${className}">${processFormattingTags(cell)}</td>`
          }
          content += '</tr>'
        }
        content += '</tbody>'
      }
      content += '</table>'
      break
      
    default:
      if (entry.entries) {
        for (const subEntry of entry.entries) {
          content += await formatEntry(subEntry)
        }
      }
  }
  
  return content
}