import type { Table } from '../composables/catalog'
import { processFormattingTags } from '../utils/textFormatting'

export async function formatTableDetails(table: Table): Promise<string> {
  if (!table) return '<div>No table data available</div>'
  
  let html = '<div class="table-details">'
  
  // Header with name
  html += `
    <div class="detail-header">
      <h2>${table.name}</h2>
      ${table.caption && table.caption !== table.name ? `<div class="table-caption">${table.caption}</div>` : ''}
      <div class="header-tags">
        ${getCategoryBadge(table.name)}
        ${table.srd ? '<span class="tag tag-srd">SRD</span>' : ''}
        ${table.basic_rules ? '<span class="tag tag-basic">Basic Rules</span>' : ''}
      </div>
    </div>
  `
  
  // Intro text if present
  if (table.intro && table.intro.length > 0) {
    html += '<div class="content-section intro">'
    for (const entry of table.intro) {
      html += formatEntry(entry)
    }
    html += '</div>'
  }
  
  // The main table
  html += '<div class="content-section">'
  html += formatTableContent(table)
  html += '</div>'
  
  // Outro text if present
  if (table.outro && table.outro.length > 0) {
    html += '<div class="content-section outro">'
    for (const entry of table.outro) {
      html += formatEntry(entry)
    }
    html += '</div>'
  }
  
  // Footnotes if present
  if (table.footnotes && table.footnotes.length > 0) {
    html += '<div class="content-section footnotes">'
    html += '<h4>Notes</h4>'
    for (const note of table.footnotes) {
      html += formatEntry(note)
    }
    html += '</div>'
  }
  
  // Source info
  html += `
    <div class="source-info">
      <strong>Source:</strong> ${table.source}
      ${table.page ? `, p. ${table.page}` : ''}
    </div>
  `
  
  html += '</div>'
  
  // Add styles
  html += `
    <style>
      .table-details {
        padding: var(--spacing-md, 12px);
        color: var(--color-text, #e0e0e0);
      }
      
      .detail-header {
        margin-bottom: var(--spacing-lg, 16px);
      }
      
      .detail-header h2 {
        margin: 0 0 var(--spacing-sm, 8px) 0;
        color: var(--color-primary, #4a9eff);
      }
      
      .table-caption {
        font-style: italic;
        color: var(--color-text-secondary, #999);
        margin-bottom: var(--spacing-sm, 8px);
      }
      
      .header-tags {
        display: flex;
        gap: var(--spacing-sm, 8px);
        flex-wrap: wrap;
      }
      
      .tag {
        display: inline-block;
        padding: 2px 8px;
        border-radius: 12px;
        font-size: 0.85rem;
        font-weight: 500;
      }
      
      .tag-srd {
        background: rgba(76, 175, 80, 0.1);
        color: #4caf50;
        border: 1px solid rgba(76, 175, 80, 0.3);
      }
      
      .tag-basic {
        background: rgba(33, 150, 243, 0.1);
        color: #2196f3;
        border: 1px solid rgba(33, 150, 243, 0.3);
      }
      
      .content-section {
        margin: var(--spacing-lg, 16px) 0;
      }
      
      .intro, .outro {
        padding: var(--spacing-sm, 8px);
        background: rgba(255, 255, 255, 0.02);
        border-radius: 4px;
      }
      
      .footnotes {
        padding: var(--spacing-sm, 8px);
        background: rgba(255, 193, 7, 0.05);
        border-left: 3px solid #ffc107;
      }
      
      .footnotes h4 {
        margin-top: 0;
        color: #ffc107;
      }
      
      .main-table {
        width: 100%;
        border-collapse: collapse;
        margin: var(--spacing-md, 12px) 0;
      }
      
      .main-table th {
        background: var(--color-surface, #1a1a1a);
        color: var(--color-primary, #4a9eff);
        padding: var(--spacing-sm, 8px);
        text-align: left;
        border: 1px solid var(--color-border, #333);
        font-weight: 600;
      }
      
      .main-table td {
        padding: var(--spacing-sm, 8px);
        border: 1px solid var(--color-border-light, #262626);
      }
      
      .main-table tbody tr:nth-child(even) {
        background: rgba(255, 255, 255, 0.02);
      }
      
      .main-table tbody tr:hover {
        background: var(--color-surface-hover, #262626);
      }
      
      /* Column-specific styles */
      .col-2 { width: 16.66%; }
      .col-3 { width: 25%; }
      .col-4 { width: 33.33%; }
      .col-5 { width: 41.66%; }
      .col-6 { width: 50%; }
      .col-8 { width: 66.66%; }
      .col-10 { width: 83.33%; }
      
      .text-center { text-align: center; }
      .text-right { text-align: right; }
      .bold { font-weight: bold; }
      
      .dice-column {
        text-align: center;
        font-weight: bold;
        color: var(--color-accent, #ff6b6b);
      }
      
      .source-info {
        margin-top: var(--spacing-lg, 16px);
        padding-top: var(--spacing-md, 12px);
        border-top: 1px solid var(--color-border, #333);
        color: var(--color-text-secondary, #999);
        font-size: 0.9rem;
      }
      
      .subsection {
        margin-left: var(--spacing-md, 12px);
        padding-left: var(--spacing-sm, 8px);
        border-left: 2px solid var(--color-border-light, #262626);
      }
      
      ul, ol {
        margin: var(--spacing-sm, 8px) 0;
        padding-left: var(--spacing-lg, 16px);
      }
      
      li {
        margin: var(--spacing-xs, 4px) 0;
      }
    </style>
  `
  
  return html
}

function getCategoryBadge(name: string): string {
  const category = categorizeTable(name)
  const categoryClass = getCategoryClass(category)
  return `<span class="tag ${categoryClass}">${category}</span>`
}

function categorizeTable(name: string): string {
  const name_lower = name.toLowerCase()
  
  if (name_lower.includes('madness') || name_lower.includes('insanity')) {
    return 'Madness'
  } else if (name_lower.includes('treasure') || name_lower.includes('loot') || name_lower.includes('hoard')) {
    return 'Treasure'
  } else if (name_lower.includes('encounter') || name_lower.includes('random')) {
    return 'Encounters'
  } else if (name_lower.includes('trinket')) {
    return 'Trinkets'
  } else if (name_lower.includes('wild magic') || name_lower.includes('surge')) {
    return 'Wild Magic'
  } else if (name_lower.includes('damage') || name_lower.includes('critical')) {
    return 'Combat'
  } else if (name_lower.includes('npc') || name_lower.includes('name') || name_lower.includes('personality')) {
    return 'NPCs'
  } else if (name_lower.includes('quest') || name_lower.includes('adventure') || name_lower.includes('plot')) {
    return 'Adventures'
  } else if (name_lower.includes('magic item') || name_lower.includes('artifact')) {
    return 'Magic Items'
  } else {
    return 'Miscellaneous'
  }
}

function getCategoryClass(category: string | undefined): string {
  if (!category) return 'category-general'
  switch (category.toLowerCase()) {
    case 'madness':
      return 'category-madness'
    case 'treasure':
      return 'category-treasure'
    case 'encounters':
      return 'category-encounters'
    case 'trinkets':
      return 'category-trinkets'
    case 'wild magic':
      return 'category-wild-magic'
    case 'combat':
      return 'category-combat'
    case 'npcs':
      return 'category-npcs'
    case 'adventures':
      return 'category-adventures'
    case 'magic items':
      return 'category-magic-items'
    default:
      return 'category-misc'
  }
}

function formatTableContent(table: Table): string {
  let html = '<table class="main-table">'
  
  // Headers
  if (table.col_labels && table.col_labels.length > 0) {
    html += '<thead><tr>'
    table.col_labels.forEach((label, idx) => {
      const style = table.col_styles?.[idx] || ''
      const classes = style.split(' ').filter(c => c).join(' ')
      html += `<th class="${classes}">${processFormattingTags(label)}</th>`
    })
    html += '</tr></thead>'
  }
  
  // Body
  html += '<tbody>'
  for (const row of table.rows) {
    html += '<tr>'
    if (Array.isArray(row)) {
      row.forEach((cell, idx) => {
        const style = table.col_styles?.[idx] || ''
        const classes = style.split(' ').filter(c => c).join(' ')
        
        // Check if this is a dice column (first column often contains dice rolls)
        const isDiceColumn = idx === 0 && table.col_labels?.[0]?.toLowerCase().includes('d')
        const cellClass = isDiceColumn ? `${classes} dice-column` : classes
        
        if (typeof cell === 'string') {
          html += `<td class="${cellClass}">${processFormattingTags(cell)}</td>`
        } else if (cell && typeof cell === 'object') {
          html += `<td class="${cellClass}">${formatEntry(cell)}</td>`
        } else {
          html += `<td class="${cellClass}">${cell}</td>`
        }
      })
    } else {
      html += `<td>${formatEntry(row)}</td>`
    }
    html += '</tr>'
  }
  html += '</tbody>'
  
  html += '</table>'
  return html
}

function formatEntry(entry: any): string {
  if (!entry) return ''
  
  // String entry
  if (typeof entry === 'string') {
    return processFormattingTags(entry)
  }
  
  // Object entry with type
  if (entry.type) {
    switch (entry.type) {
      case 'entries':
        let html = ''
        if (entry.name) {
          html += `<h4>${entry.name}</h4>`
        }
        if (entry.entries) {
          html += '<div class="subsection">'
          for (const subEntry of entry.entries) {
            html += `<p>${formatEntry(subEntry)}</p>`
          }
          html += '</div>'
        }
        return html
        
      case 'list':
        let listHtml = '<ul>'
        if (entry.items) {
          for (const item of entry.items) {
            if (typeof item === 'string') {
              listHtml += `<li>${processFormattingTags(item)}</li>`
            } else {
              listHtml += `<li>${formatEntry(item)}</li>`
            }
          }
        }
        listHtml += '</ul>'
        return listHtml
        
      case 'cell':
        // Handle special cell types
        if (entry.roll) {
          if (entry.roll.exact !== undefined) {
            return String(entry.roll.exact)
          } else if (entry.roll.min !== undefined && entry.roll.max !== undefined) {
            return `${entry.roll.min}-${entry.roll.max}`
          }
        }
        return ''
        
      default:
        // Generic handling
        if (entry.entries) {
          return entry.entries.map((e: any) => formatEntry(e)).join('')
        }
        return ''
    }
  }
  
  // Array of entries
  if (Array.isArray(entry)) {
    return entry.map(e => formatEntry(e)).join('')
  }
  
  return ''
}

// Add category styles to the style section
const categoryStyles = `
  .category-madness {
    background: rgba(156, 39, 176, 0.2);
    color: #9c27b0;
    border: 1px solid rgba(156, 39, 176, 0.4);
  }
  
  .category-treasure {
    background: rgba(255, 193, 7, 0.2);
    color: #ffc107;
    border: 1px solid rgba(255, 193, 7, 0.4);
  }
  
  .category-encounters {
    background: rgba(244, 67, 54, 0.2);
    color: #f44336;
    border: 1px solid rgba(244, 67, 54, 0.4);
  }
  
  .category-trinkets {
    background: rgba(0, 188, 212, 0.2);
    color: #00bcd4;
    border: 1px solid rgba(0, 188, 212, 0.4);
  }
  
  .category-wild-magic {
    background: rgba(103, 58, 183, 0.2);
    color: #673ab7;
    border: 1px solid rgba(103, 58, 183, 0.4);
  }
  
  .category-combat {
    background: rgba(255, 87, 34, 0.2);
    color: #ff5722;
    border: 1px solid rgba(255, 87, 34, 0.4);
  }
  
  .category-npcs {
    background: rgba(76, 175, 80, 0.2);
    color: #4caf50;
    border: 1px solid rgba(76, 175, 80, 0.4);
  }
  
  .category-adventures {
    background: rgba(33, 150, 243, 0.2);
    color: #2196f3;
    border: 1px solid rgba(33, 150, 243, 0.4);
  }
  
  .category-magic-items {
    background: rgba(255, 152, 0, 0.2);
    color: #ff9800;
    border: 1px solid rgba(255, 152, 0, 0.4);
  }
  
  .category-misc {
    background: var(--color-surface, #1a1a1a);
    color: var(--color-text-secondary, #999);
    border: 1px solid var(--color-border, #333);
  }
`