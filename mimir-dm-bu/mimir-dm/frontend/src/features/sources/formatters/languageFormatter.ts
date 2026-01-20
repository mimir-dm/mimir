import type { Language } from '../composables/catalog'
import { processFormattingTags } from '../utils/textFormatting'

export async function formatLanguageDetails(language: Language): Promise<string> {
  if (!language) return '<div>No language data available</div>'
  
  let html = '<div class="language-details">'
  
  // Header with name and type
  html += `
    <div class="detail-header">
      <h2>${language.name}</h2>
      <div class="header-tags">
        ${language.language_type ? `<span class="tag tag-type">${formatLanguageType(language.language_type)}</span>` : ''}
        ${language.srd ? '<span class="tag tag-srd">SRD</span>' : ''}
        ${language.basic_rules ? '<span class="tag tag-basic">Basic Rules</span>' : ''}
      </div>
    </div>
  `
  
  // Key Information Grid
  html += '<div class="info-grid">'
  
  // Script
  if (language.script) {
    html += `
      <div class="info-item">
        <div class="info-label">Script</div>
        <div class="info-value">${language.script}</div>
      </div>
    `
  }
  
  // Type
  html += `
    <div class="info-item">
      <div class="info-label">Type</div>
      <div class="info-value">${formatLanguageType(language.language_type || 'standard')}</div>
    </div>
  `
  
  // Typical Speakers
  if (language.typical_speakers && language.typical_speakers.length > 0) {
    html += `
      <div class="info-item full-width">
        <div class="info-label">Typical Speakers</div>
        <div class="info-value">${formatSpeakers(language.typical_speakers)}</div>
      </div>
    `
  }
  
  // Fonts (if any)
  if (language.fonts && language.fonts.length > 0) {
    html += `
      <div class="info-item full-width">
        <div class="info-label">Fonts</div>
        <div class="info-value">${language.fonts.join(', ')}</div>
      </div>
    `
  }
  
  // Dialects (if any)
  if (language.dialects && language.dialects.length > 0) {
    html += `
      <div class="info-item full-width">
        <div class="info-label">Dialects</div>
        <div class="info-value">${language.dialects.join(', ')}</div>
      </div>
    `
  }
  
  html += '</div>' // End info-grid
  
  // Description/Entries
  if (language.entries && language.entries.length > 0) {
    html += '<div class="content-section">'
    html += '<h3>Description</h3>'
    for (const entry of language.entries) {
      html += formatEntry(entry)
    }
    html += '</div>'
  }
  
  // Source info
  html += `
    <div class="source-info">
      <strong>Source:</strong> ${language.source}
      ${language.page ? `, p. ${language.page}` : ''}
    </div>
  `
  
  html += '</div>'
  
  return html
}

function formatLanguageType(type: string | undefined): string {
  if (!type) return 'Standard'
  
  const typeMap: Record<string, string> = {
    'standard': 'Standard',
    'exotic': 'Exotic',
    'secret': 'Secret',
    'dead': 'Dead',
    'primordial': 'Primordial Dialect'
  }
  return typeMap[type.toLowerCase()] || type
}

function formatSpeakers(speakers: string[]): string {
  // Format speakers with clickable cross-references
  return speakers
    .map(s => {
      // Use processFormattingTags to handle all special tags including cross-references
      return processFormattingTags(s);
    })
    .join(', ')
}

function formatEntry(entry: any): string {
  if (!entry) return ''
  
  // String entry
  if (typeof entry === 'string') {
    return `<p>${processFormattingTags(entry)}</p>`
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
              listHtml += `<li>${processFormattingTags(item)}</li>`
            } else {
              listHtml += `<li>${formatEntry(item)}</li>`
            }
          }
        }
        listHtml += '</ul>'
        return listHtml
        
      case 'table':
        return formatTable(entry)
        
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

function formatTable(table: any): string {
  let html = '<table class="language-table">'
  
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
          html += `<td>${typeof cell === 'string' ? processFormattingTags(cell) : formatEntry(cell)}</td>`
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