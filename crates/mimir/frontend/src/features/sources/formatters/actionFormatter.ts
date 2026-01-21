import { processFormattingTags, formatEntries } from '../utils/textFormatting'

export async function formatActionDetails(action: any): Promise<string> {
  if (!action) {
    return `<div class="error">Action details not available</div>`
  }

  let content = ''
  
  // Add styles for better formatting
  content += `<style>
    .action-header {
      border-bottom: 2px solid var(--color-border, #333);
      padding-bottom: 1em;
      margin-bottom: 1em;
    }
    .action-header h1 {
      margin: 0 0 0.5em 0;
    }
    .source-info {
      font-size: 0.9em;
      color: var(--color-text-secondary, #999);
    }
    .time-section {
      background: var(--color-background-secondary, #1a1a1a);
      padding: 0.75em 1em;
      border-radius: 4px;
      margin: 1em 0;
      border-left: 3px solid var(--color-primary, #4a9eff);
    }
    .time-section strong {
      color: var(--color-primary, #4a9eff);
    }
    .description-section {
      margin: 1.5em 0;
      line-height: 1.6;
    }
    .description-section p {
      margin: 0.75em 0;
    }
    .see-also-section {
      background: var(--color-background-secondary, #1a1a1a);
      padding: 1em;
      border-radius: 4px;
      margin-top: 1.5em;
    }
    .see-also-section h3 {
      margin-top: 0;
      color: var(--color-primary, #4a9eff);
      font-size: 1.1em;
    }
    .see-also-section ul {
      margin: 0.5em 0 0 1.5em;
      padding: 0;
    }
    .see-also-section li {
      margin: 0.25em 0;
    }
    .tags-section {
      margin-top: 1.5em;
      padding-top: 1em;
      border-top: 1px solid var(--color-border, #333);
    }
    .tags {
      display: flex;
      gap: 0.5em;
    }
    .tag {
      background: var(--color-background-tertiary, #262626);
      padding: 0.25em 0.75em;
      border-radius: 12px;
      font-size: 0.85em;
      color: var(--color-text-secondary, #999);
    }
  </style>`
  
  // Header with name and source
  content += `<div class="action-header">`
  content += `<h1>${action.name}</h1>`
  content += `<div class="source-info">Source: ${action.source}${action.page ? `, p. ${action.page}` : ''}</div>`
  content += `</div>`
  
  // Time required
  if (action.time && action.time.length > 0) {
    content += `<div class="time-section">`
    content += `<strong>Time:</strong> ${formatTime(action.time)}`
    content += `</div>`
  }
  
  // Main description
  if (action.entries && action.entries.length > 0) {
    content += `<div class="description-section">`
    for (const entry of action.entries) {
      content += formatEntry(entry)
    }
    content += `</div>`
  }
  
  // See Also section
  if (action.see_also_action && action.see_also_action.length > 0) {
    content += `<div class="see-also-section">`
    content += `<h3>See Also</h3>`
    content += `<ul>`
    for (const related of action.see_also_action) {
      // Split by | to separate name from source if present
      const [name, source] = related.split('|')
      if (source) {
        content += `<li>${name} (${source})</li>`
      } else {
        content += `<li>${name}</li>`
      }
    }
    content += `</ul>`
    content += `</div>`
  }
  
  // Rules tags
  const tags = []
  if (action.srd) tags.push('SRD')
  if (action.basicRules || action.basic_rules) tags.push('Basic Rules')
  
  if (tags.length > 0) {
    content += `<div class="tags-section">`
    content += `<div class="tags">`
    for (const tag of tags) {
      content += `<span class="tag">${tag}</span>`
    }
    content += `</div>`
    content += `</div>`
  }
  
  return content
}

function formatTime(time: any): string {
  // Handle if time is already a formatted string from the database
  if (typeof time === 'string') {
    return time
  }
  
  // Handle if time is null/undefined or empty array
  if (!time || (Array.isArray(time) && time.length === 0)) {
    return '1 action'
  }
  
  // Handle if time is not an array
  if (!Array.isArray(time)) {
    return '1 action'
  }
  
  const times: string[] = []
  
  for (const t of time) {
    if (typeof t === 'string') {
      times.push(t)
    } else if (typeof t === 'object' && t.number && t.unit) {
      const num = t.number
      const unit = t.unit
      if (num === 1) {
        times.push(`1 ${unit}`)
      } else {
        times.push(`${num} ${unit}s`)
      }
    }
  }
  
  return times.join(', ') || '1 action'
}

function formatEntry(entry: any): string {
  if (!entry) return ''
  
  if (typeof entry === 'string') {
    return `<p>${processFormattingTags(entry)}</p>`
  }
  
  if (typeof entry === 'object') {
    let content = ''
    
    if (entry.type === 'entries') {
      if (entry.name) {
        content += `<h3>${entry.name}</h3>`
      }
      if (entry.entries) {
        for (const subEntry of entry.entries) {
          content += formatEntry(subEntry)
        }
      }
    } else if (entry.type === 'list') {
      content += '<ul>'
      if (entry.items) {
        for (const item of entry.items) {
          if (typeof item === 'string') {
            content += `<li>${processFormattingTags(item)}</li>`
          } else {
            content += `<li>${formatEntry(item)}</li>`
          }
        }
      }
      content += '</ul>'
    } else if (entry.entries) {
      for (const subEntry of entry.entries) {
        content += formatEntry(subEntry)
      }
    }
    
    return content
  }
  
  return ''
}