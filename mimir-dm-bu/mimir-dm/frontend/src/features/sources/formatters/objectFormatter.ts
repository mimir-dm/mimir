import { processFormattingTags, formatEntries } from '../utils/textFormatting'

export async function formatObjectDetails(obj: any): Promise<string> {
  let html = '<div class="object-details">'
  
  // Header with name and type
  html += '<div class="object-header">'
  html += `<h2 class="object-name">${obj.name}</h2>`
  if (obj.object_type || obj.objectType) {
    const type = formatObjectType(obj.object_type || obj.objectType)
    html += `<div class="object-type">${type}</div>`
  }
  html += '</div>'
  
  // Core stats grid
  html += '<div class="stats-grid">'
  
  // Size
  if (obj.size) {
    const sizeText = Array.isArray(obj.size) ? formatSizes(obj.size) : obj.size
    html += `
      <div class="stat-card">
        <div class="stat-label">Size</div>
        <div class="stat-value">${sizeText}</div>
      </div>
    `
  }
  
  // Armor Class
  if (obj.ac !== undefined) {
    const acText = formatAC(obj.ac)
    html += `
      <div class="stat-card">
        <div class="stat-label">Armor Class</div>
        <div class="stat-value ac-value">${acText}</div>
      </div>
    `
  }
  
  // Hit Points
  if (obj.hp !== undefined) {
    html += `
      <div class="stat-card">
        <div class="stat-label">Hit Points</div>
        <div class="stat-value hp-value">${obj.hp}</div>
      </div>
    `
  }
  
  html += '</div>' // Close stats grid
  
  // Damage immunities, resistances, vulnerabilities
  html += '<div class="defenses-section">'
  
  if (obj.immune && obj.immune.length > 0) {
    html += '<div class="defense-row">'
    html += '<strong>Damage Immunities:</strong> '
    html += obj.immune.join(', ')
    html += '</div>'
  }
  
  if (obj.resist && obj.resist.length > 0) {
    html += '<div class="defense-row">'
    html += '<strong>Damage Resistances:</strong> '
    html += obj.resist.join(', ')
    html += '</div>'
  }
  
  if (obj.vulnerable && obj.vulnerable.length > 0) {
    html += '<div class="defense-row">'
    html += '<strong>Damage Vulnerabilities:</strong> '
    html += obj.vulnerable.join(', ')
    html += '</div>'
  }
  
  html += '</div>'
  
  // Action entries (for siege weapons, etc.)
  if (obj.actionEntries && obj.actionEntries.length > 0) {
    html += '<div class="actions-section">'
    html += '<h3 class="section-title">Actions</h3>'
    
    for (const action of obj.actionEntries) {
      html += '<div class="action-block">'
      html += `<h4 class="action-name">${action.name}</h4>`
      
      if (action.entries && action.entries.length > 0) {
        for (const entry of action.entries) {
          if (typeof entry === 'string') {
            html += `<p>${processFormattingTags(entry)}</p>`
          } else if (entry.type === 'attack') {
            html += '<div class="attack-details">'
            
            if (entry.attackEntries) {
              html += '<div class="attack-line">'
              for (const attackEntry of entry.attackEntries) {
                html += processFormattingTags(attackEntry) + ' '
              }
              html += '</div>'
            }
            
            if (entry.hitEntries) {
              html += '<div class="hit-line"><strong>Hit:</strong> '
              for (const hitEntry of entry.hitEntries) {
                html += processFormattingTags(hitEntry) + ' '
              }
              html += '</div>'
            }
            
            html += '</div>'
          } else {
            html += formatEntries([entry])
          }
        }
      }
      
      html += '</div>'
    }
    
    html += '</div>'
  }
  
  // Main entries/description
  if (obj.entries && obj.entries.length > 0) {
    html += '<div class="description-section">'
    html += '<h3 class="section-title">Description</h3>'
    html += '<div class="description-content">'
    html += formatEntries(obj.entries)
    html += '</div>'
    html += '</div>'
  }
  
  // Footer with source info
  html += '<div class="object-footer">'
  if (obj.source) {
    html += `<span class="source-badge">${obj.source}</span>`
  }
  if (obj.page) {
    html += `<span class="page-ref">Page ${obj.page}</span>`
  }
  if (obj.srd) {
    html += '<span class="srd-badge">SRD</span>'
  }
  html += '</div>'
  
  html += '</div>'
  
  // Add styles
  html += `
    <style>
      .object-details {
        padding: 20px;
        color: var(--color-text, #e0e0e0);
        max-width: 900px;
        margin: 0 auto;
      }
      
      .object-header {
        margin-bottom: 24px;
        padding-bottom: 16px;
        border-bottom: 2px solid var(--color-border, #333);
        text-align: center;
      }
      
      .object-name {
        font-size: 2rem;
        color: var(--color-primary, #4a9eff);
        margin: 0 0 8px 0;
        font-weight: 700;
      }
      
      .object-type {
        font-size: 1.1rem;
        color: var(--color-text-secondary, #999);
        font-style: italic;
      }
      
      .stats-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
        gap: 16px;
        margin-bottom: 24px;
      }
      
      .stat-card {
        background: var(--color-surface, #1a1a1a);
        border: 1px solid var(--color-border, #333);
        border-radius: 8px;
        padding: 12px;
        text-align: center;
        transition: all 0.2s ease;
      }
      
      .stat-card:hover {
        background: var(--color-surface-hover, #252525);
        border-color: var(--color-primary, #4a9eff);
        transform: translateY(-2px);
      }
      
      .stat-label {
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 1px;
        color: var(--color-text-secondary, #999);
        margin-bottom: 8px;
        font-weight: 600;
      }
      
      .stat-value {
        font-size: 1.5rem;
        font-weight: 700;
        color: var(--color-text, #e0e0e0);
      }
      
      .ac-value {
        color: #3498db;
      }
      
      .hp-value {
        color: #e74c3c;
      }
      
      .defenses-section {
        margin: 24px 0;
        padding: 16px;
        background: rgba(255, 255, 255, 0.02);
        border-left: 3px solid var(--color-primary, #4a9eff);
        border-radius: 4px;
      }
      
      .defense-row {
        margin: 8px 0;
        line-height: 1.6;
      }
      
      .defense-row strong {
        color: var(--color-primary, #4a9eff);
        margin-right: 8px;
      }
      
      .actions-section {
        margin: 24px 0;
      }
      
      .section-title {
        font-size: 1.3rem;
        color: var(--color-primary, #4a9eff);
        margin: 0 0 16px 0;
        font-weight: 600;
        border-bottom: 1px solid rgba(74, 158, 255, 0.3);
        padding-bottom: 8px;
      }
      
      .action-block {
        margin: 20px 0;
        padding: 16px;
        background: var(--color-surface, #1a1a1a);
        border-radius: 6px;
        border: 1px solid var(--color-border, #333);
      }
      
      .action-name {
        font-size: 1.1rem;
        color: var(--color-primary, #4a9eff);
        margin: 0 0 12px 0;
        font-weight: 600;
      }
      
      .attack-details {
        margin: 12px 0;
        padding: 12px;
        background: rgba(74, 158, 255, 0.05);
        border-left: 3px solid var(--color-primary, #4a9eff);
        border-radius: 4px;
      }
      
      .attack-line,
      .hit-line {
        margin: 6px 0;
        line-height: 1.6;
      }
      
      .hit-line strong {
        color: #e74c3c;
      }
      
      .description-section {
        margin: 24px 0;
      }
      
      .description-content {
        background: rgba(255, 255, 255, 0.02);
        border-left: 3px solid var(--color-primary, #4a9eff);
        padding: 16px 20px;
        border-radius: 4px;
        line-height: 1.8;
      }
      
      .description-content p {
        margin: 12px 0;
      }
      
      .description-content ul,
      .description-content ol {
        margin: 12px 0;
        padding-left: 24px;
      }
      
      .description-content li {
        margin: 6px 0;
        line-height: 1.6;
      }
      
      .description-content table {
        width: 100%;
        margin: 16px 0;
        border-collapse: collapse;
      }
      
      .description-content th {
        background: var(--color-surface, #1a1a1a);
        padding: 8px;
        text-align: left;
        border: 1px solid var(--color-border, #333);
        font-weight: 600;
      }
      
      .description-content td {
        padding: 8px;
        border: 1px solid var(--color-border, #333);
      }
      
      .object-footer {
        margin-top: 24px;
        padding-top: 16px;
        border-top: 1px solid var(--color-border, #333);
        display: flex;
        gap: 16px;
        align-items: center;
        font-size: 0.9rem;
      }
      
      .source-badge {
        padding: 4px 8px;
        background: var(--color-surface, #1a1a1a);
        border: 1px solid var(--color-border, #333);
        border-radius: 4px;
        font-family: monospace;
      }
      
      .page-ref {
        color: var(--color-text-secondary, #999);
        font-style: italic;
      }
      
      .srd-badge {
        padding: 4px 8px;
        background: var(--color-primary, #4a9eff);
        color: var(--color-background, #0d0d0d);
        border-radius: 4px;
        font-weight: 600;
      }
    </style>
  `
  
  return html
}

function formatObjectType(type: string): string {
  switch (type) {
    case 'SW': return 'Siege Weapon'
    case 'GEN': return 'Generic Object'
    default: return type || 'Object'
  }
}

function formatSizes(sizes: string[]): string {
  return sizes.map(s => {
    switch (s) {
      case 'T': return 'Tiny'
      case 'S': return 'Small'
      case 'M': return 'Medium'
      case 'L': return 'Large'
      case 'H': return 'Huge'
      case 'G': return 'Gargantuan'
      default: return s
    }
  }).join('/')
}

function formatAC(ac: any): string {
  if (typeof ac === 'number') {
    return ac.toString()
  }
  if (ac && typeof ac === 'object') {
    if (ac.special) return ac.special
    if (ac.ac) return ac.ac.toString()
  }
  return '—'
}