import type { ConditionWithDetails, ConditionSummary } from '../services/SearchService'
import { processFormattingTags } from '../utils/textFormatting'

function formatConditionSummary(summary: ConditionSummary): string {
  let html = '<div class="condition-details">'
  
  // Header with type badge
  html += '<div class="condition-header">'
  html += `<h1>${summary.name}</h1>`
  const itemType = summary.item_type || 'Condition'
  html += `<span class="type-badge ${itemType.toLowerCase()}">${itemType}</span>`
  html += '</div>'
  
  // Source info
  html += '<div class="source-info">'
  html += `<strong>Source:</strong> ${summary.source}`
  html += '</div>'
  
  // Description
  html += '<div class="condition-description">'
  html += `<p>${processFormattingTags(summary.description)}</p>`
  html += '</div>'
  
  html += '</div>'
  
  return html
}

export function formatConditionDetails(details: ConditionWithDetails | ConditionSummary): string {
  
  // Check if this is summary data (has item_type field)
  if ('item_type' in details) {
    return formatConditionSummary(details as ConditionSummary)
  }
  
  // Check the actual structure - maybe it's wrapped differently
  let item = null
  let itemType = null
  
  // Try the expected structure first
  if ('type' in details) {
    itemType = (details as any).type
    if (itemType === 'Condition' && 'Condition' in details) {
      item = (details as any).Condition
    } else if (itemType === 'Disease' && 'Disease' in details) {
      item = (details as any).Disease
    }
  }
  
  // If that didn't work, check if item is nested under 'item'
  if (!item && 'item' in details) {
    const wrapper = (details as any).item
    if (wrapper && typeof wrapper === 'object') {
      if ('type' in wrapper) {
        itemType = wrapper.type
        item = wrapper[itemType] // Try wrapper.Condition or wrapper.Disease
      }
      // Or maybe the whole wrapper is the item
      if (!item && 'name' in wrapper && 'entries' in wrapper) {
        item = wrapper
        itemType = wrapper.type || 'Condition'
      }
    }
  }
  
  // Last resort - maybe details itself is the item
  if (!item && 'name' in details && 'entries' in details) {
    item = details
    itemType = (details as any).type || 'Condition'
  }

  if (!item || !itemType) {
    // Fallback for unexpected structure
    return formatConditionSummary({
      name: (details as any).name || 'Unknown',
      source: (details as any).source || 'Unknown',
      item_type: 'Condition',
      description: (details as any).description || '',
    })
  }

  let html = '<div class="condition-details">'
  
  // Header with type badge
  html += '<div class="condition-header">'
  html += `<h1>${item.name}</h1>`
  html += `<span class="type-badge ${itemType.toLowerCase()}">${itemType}</span>`
  html += '</div>'
  
  // Source info
  html += '<div class="source-info">'
  html += `<strong>Source:</strong> ${item.source} p.${item.page || 'N/A'}`
  if (item.srd) {
    html += ' <span class="srd-badge">SRD</span>'
  }
  html += '</div>'
  
  // Main description/entries
  html += '<div class="condition-description">'
  if (item && item.entries && item.entries.length > 0) {
    for (const entry of item.entries) {
      if (typeof entry === 'string') {
        html += `<p>${processFormattingTags(entry)}</p>`
      } else if (entry && typeof entry === 'object') {
        // Handle list entries
        if ('type' in entry && entry.type === 'list') {
          if ('style' in entry && entry.style === 'list-hang-notitle') {
            // Special formatting for hanging lists
            html += '<ul class="hanging-list">'
          } else {
            html += '<ul>'
          }
          
          if ('items' in entry && Array.isArray(entry.items)) {
            for (const listItem of entry.items) {
              if (typeof listItem === 'string') {
                html += `<li>${processFormattingTags(listItem)}</li>`
              } else if (typeof listItem === 'object' && listItem) {
                // Handle complex list items with name and entry
                if ('name' in listItem && 'entry' in listItem) {
                  html += `<li><strong>${listItem.name}.</strong> ${processFormattingTags(listItem.entry)}</li>`
                } else if ('name' in listItem && 'entries' in listItem) {
                  html += `<li><strong>${listItem.name}.</strong> `
                  for (const subEntry of listItem.entries) {
                    html += processFormattingTags(String(subEntry))
                  }
                  html += '</li>'
                } else {
                  html += `<li>${processFormattingTags(String(listItem))}</li>`
                }
              }
            }
          }
          html += '</ul>'
        }
        // Handle table entries (for diseases)
        else if ('type' in entry && entry.type === 'table') {
          html += '<table class="condition-table">'
          if ('colLabels' in entry && Array.isArray(entry.colLabels)) {
            html += '<thead><tr>'
            for (const label of entry.colLabels) {
              html += `<th>${label}</th>`
            }
            html += '</tr></thead>'
          }
          if ('rows' in entry && Array.isArray(entry.rows)) {
            html += '<tbody>'
            for (const row of entry.rows) {
              html += '<tr>'
              if (Array.isArray(row)) {
                for (const cell of row) {
                  html += `<td>${processFormattingTags(String(cell))}</td>`
                }
              }
              html += '</tr>'
            }
            html += '</tbody>'
          }
          html += '</table>'
        }
        // Handle section entries
        else if ('entries' in entry) {
          html += '<div class="sub-section">'
          if ('name' in entry) {
            html += `<h3>${entry.name}</h3>`
          }
          if (Array.isArray(entry.entries)) {
            for (const subEntry of entry.entries) {
              if (typeof subEntry === 'string') {
                html += `<p>${processFormattingTags(subEntry)}</p>`
              }
            }
          }
          html += '</div>'
        }
      }
    }
  } else {
    // No entries found - show what we have
    if ((details as any).description) {
      html += `<p>${processFormattingTags((details as any).description)}</p>`
    } else {
      html += '<p><em>No description available</em></p>'
    }
  }
  html += '</div>'
  
  // Fluff/lore if available
  const fluff = (details as any).fluff
  if (fluff) {
    html += '<div class="condition-fluff">'
    html += '<h3>Additional Lore</h3>'
    if (fluff.entries) {
      for (const entry of fluff.entries) {
        if (typeof entry === 'string') {
          html += `<p class="lore-text">${processFormattingTags(entry)}</p>`
        }
      }
    }
    html += '</div>'
  }
  
  html += '</div>'
  
  // Add styles
  html += `
    <style>
      .condition-details {
        padding: 20px;
        max-width: 800px;
        line-height: 1.6;
      }
      
      .condition-header {
        display: flex;
        align-items: center;
        gap: 12px;
        margin-bottom: 16px;
        padding-bottom: 12px;
        border-bottom: 2px solid var(--color-border, #333);
      }
      
      .condition-header h1 {
        margin: 0;
        color: var(--color-primary, #4a9eff);
      }
      
      .type-badge {
        padding: 4px 12px;
        border-radius: 4px;
        font-size: 0.85em;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
      }
      
      .type-badge.condition {
        background: rgba(74, 158, 255, 0.2);
        color: var(--color-primary, #4a9eff);
      }
      
      .type-badge.disease {
        background: rgba(255, 107, 107, 0.2);
        color: #ff6b6b;
      }
      
      .source-info {
        margin-bottom: 20px;
        color: var(--color-text-secondary, #999);
        font-size: 0.9em;
      }
      
      .srd-badge {
        margin-left: 8px;
        padding: 2px 6px;
        background: var(--color-background-tertiary, #262626);
        border-radius: 3px;
        font-size: 0.85em;
        color: var(--color-text, #e0e0e0);
      }
      
      .condition-description {
        margin: 20px 0;
      }
      
      .condition-description p {
        margin: 12px 0;
      }
      
      .condition-description ul {
        margin: 12px 0;
        padding-left: 24px;
      }
      
      .condition-description li {
        margin: 6px 0;
        color: var(--color-text, #e0e0e0);
      }
      
      .sub-section {
        margin: 20px 0;
        padding-left: 16px;
        border-left: 3px solid var(--color-border, #333);
      }
      
      .sub-section h3 {
        margin: 0 0 12px 0;
        color: var(--color-primary, #4a9eff);
        font-size: 1.1em;
      }
      
      .condition-fluff {
        margin-top: 32px;
        padding-top: 20px;
        border-top: 1px solid var(--color-border, #333);
      }
      
      .condition-fluff h3 {
        margin: 0 0 12px 0;
        color: var(--color-text-secondary, #999);
        font-size: 1em;
        text-transform: uppercase;
        letter-spacing: 1px;
      }
      
      .lore-text {
        font-style: italic;
        color: var(--color-text-secondary, #999);
      }
      
      .hanging-list {
        padding-left: 0;
        list-style: none;
      }
      
      .hanging-list li {
        margin: 8px 0;
        padding-left: 20px;
        text-indent: -20px;
      }
      
      .condition-table {
        width: 100%;
        margin: 16px 0;
        border-collapse: collapse;
      }
      
      .condition-table th {
        background: var(--color-background-secondary, #1a1a1a);
        padding: 8px;
        text-align: left;
        border: 1px solid var(--color-border, #333);
        font-weight: 600;
      }
      
      .condition-table td {
        padding: 8px;
        border: 1px solid var(--color-border, #333);
      }
      
      .condition-table tbody tr:hover {
        background: rgba(74, 158, 255, 0.05);
      }
    </style>
  `
  
  return html
}