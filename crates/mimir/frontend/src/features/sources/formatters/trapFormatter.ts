import type { TrapOrHazard } from '../composables/catalog'

export async function formatTrapDetails(trap: TrapOrHazard): Promise<string> {
  if (!trap) return '<div>No trap data available</div>'
  
  let html = '<div class="trap-details">'
  
  // Header with name and type
  html += `
    <div class="detail-header">
      <h2>${trap.name}</h2>
      <div class="header-tags">
        ${trap.trap_haz_type ? `<span class="tag tag-type">${formatTrapType(trap.trap_haz_type)}</span>` : ''}
      </div>
    </div>
  `
  
  // Main content entries
  if (trap.entries && trap.entries.length > 0) {
    html += '<div class="content-section">'
    for (const entry of trap.entries) {
      html += formatEntry(entry)
    }
    html += '</div>'
  }
  
  // Source info
  html += `
    <div class="source-info">
      <strong>Source:</strong> ${trap.source}
      ${trap.page ? `, p. ${trap.page}` : ''}
    </div>
  `
  
  html += '</div>'
  
  return html
}

function formatTrapType(type: string): string {
  const typeMap: Record<string, string> = {
    'MECH': 'Mechanical',
    'MAG': 'Magical',
    'WLD': 'Wilderness',
    'WTH': 'Weather',
    'ENV': 'Environmental'
  }
  return typeMap[type] || type
}

function formatEntry(entry: any): string {
  if (!entry) return ''
  
  // String entry
  if (typeof entry === 'string') {
    return `<p>${entry}</p>`
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
            html += formatEntry(subEntry)
          }
          html += '</div>'
        }
        return html
        
      case 'list':
        let listHtml = '<ul>'
        if (entry.items) {
          for (const item of entry.items) {
            if (typeof item === 'string') {
              listHtml += `<li>${item}</li>`
            } else {
              listHtml += `<li>${formatEntry(item)}</li>`
            }
          }
        }
        listHtml += '</ul>'
        return listHtml
        
      case 'table':
        return formatTable(entry)
        
      case 'quote':
        return `<blockquote>${entry.entries?.map((e: any) => formatEntry(e)).join('') || ''}</blockquote>`
        
      default:
        // Generic handling
        if (entry.entries) {
          return entry.entries.map((e: any) => formatEntry(e)).join('')
        }
        return JSON.stringify(entry)
    }
  }
  
  // Array of entries
  if (Array.isArray(entry)) {
    return entry.map(e => formatEntry(e)).join('')
  }
  
  // Unknown format
  return `<p>${JSON.stringify(entry)}</p>`
}

function formatTable(table: any): string {
  let html = '<table class="trap-table">'
  
  // Headers
  if (table.colLabels) {
    html += '<thead><tr>'
    for (const label of table.colLabels) {
      html += `<th>${label}</th>`
    }
    html += '</tr></thead>'
  }
  
  // Rows
  if (table.rows) {
    html += '<tbody>'
    for (const row of table.rows) {
      html += '<tr>'
      if (Array.isArray(row)) {
        for (const cell of row) {
          html += `<td>${typeof cell === 'string' ? cell : formatEntry(cell)}</td>`
        }
      } else {
        html += `<td>${formatEntry(row)}</td>`
      }
      html += '</tr>'
    }
    html += '</tbody>'
  }
  
  html += '</table>'
  return html
}