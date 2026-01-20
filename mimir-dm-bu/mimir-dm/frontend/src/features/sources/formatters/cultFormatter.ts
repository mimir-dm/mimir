import { formatEntries, processFormattingTags } from '../utils/textFormatting'

interface Cult {
  name: string
  source: string
  cult_type?: string
  page?: number
  entries?: any[]
  cultists?: { entry: string }
  goal?: { entry: string }
  signature_spells?: { entry: string }
}

interface Boon {
  name: string
  source: string
  boon_type?: string
  page?: number
  entries?: any[]
  ability?: { entry: string }
  signature_spells?: { entry: string }
}

export function formatCultDetails(cult: Cult): string {
  if (!cult) return '<div>No cult data available</div>'
  
  let html = '<div class="cult-details">'
  
  // Header with name and type
  html += `
    <div class="detail-header">
      <h2>${cult.name}</h2>
      <div class="header-tags">
        <span class="tag tag-cult">Cult</span>
        ${cult.cult_type ? `<span class="tag tag-type">${cult.cult_type}</span>` : ''}
      </div>
    </div>
  `
  
  // Cultists info
  if (cult.cultists) {
    html += '<div class="info-section">'
    html += '<h4>Typical Cultists</h4>'
    html += `<p>${processFormattingTags(cult.cultists.entry)}</p>`
    html += '</div>'
  }
  
  // Goal info
  if (cult.goal) {
    html += '<div class="info-section">'
    html += '<h4>Goal</h4>'
    html += `<p>${processFormattingTags(cult.goal.entry)}</p>`
    html += '</div>'
  }
  
  // Signature spells
  if (cult.signature_spells) {
    html += '<div class="info-section">'
    html += '<h4>Signature Spells</h4>'
    html += `<p>${processFormattingTags(cult.signature_spells.entry)}</p>`
    html += '</div>'
  }
  
  // Main description and traits
  if (cult.entries && cult.entries.length > 0) {
    html += '<div class="content-section">'
    html += '<h3>Description & Traits</h3>'
    html += formatEntries(cult.entries)
    html += '</div>'
  }
  
  // Source info
  html += `
    <div class="source-info">
      <strong>Source:</strong> ${cult.source}
      ${cult.page ? `, p. ${cult.page}` : ''}
    </div>
  `
  
  html += '</div>'
  html += getCultBoonStyles()
  
  return html
}

export function formatBoonDetails(boon: Boon): string {
  if (!boon) return '<div>No boon data available</div>'
  
  let html = '<div class="boon-details">'
  
  // Header with name and type
  html += `
    <div class="detail-header">
      <h2>${boon.name}</h2>
      <div class="header-tags">
        <span class="tag tag-boon">Boon</span>
        ${boon.boon_type ? `<span class="tag tag-type">${boon.boon_type}</span>` : ''}
      </div>
    </div>
  `
  
  // Ability bonus info
  if (boon.ability) {
    html += '<div class="info-section">'
    html += '<h4>Ability Bonus</h4>'
    html += `<p>${processFormattingTags(boon.ability.entry)}</p>`
    html += '</div>'
  }
  
  // Signature spells
  if (boon.signature_spells) {
    html += '<div class="info-section">'
    html += '<h4>Signature Spells</h4>'
    html += `<p>${processFormattingTags(boon.signature_spells.entry)}</p>`
    html += '</div>'
  }
  
  // Main description and granted abilities
  if (boon.entries && boon.entries.length > 0) {
    html += '<div class="content-section">'
    html += '<h3>Granted Abilities</h3>'
    html += formatEntries(boon.entries)
    html += '</div>'
  }
  
  // Source info
  html += `
    <div class="source-info">
      <strong>Source:</strong> ${boon.source}
      ${boon.page ? `, p. ${boon.page}` : ''}
    </div>
  `
  
  html += '</div>'
  html += getCultBoonStyles()
  
  return html
}

function getCultBoonStyles(): string {
  return `
    <style>
      .cult-details, .boon-details {
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
      
      .tag-cult {
        background: rgba(156, 39, 176, 0.1);
        color: #9c27b0;
        border: 1px solid rgba(156, 39, 176, 0.3);
      }
      
      .tag-boon {
        background: rgba(76, 175, 80, 0.1);
        color: #4caf50;
        border: 1px solid rgba(76, 175, 80, 0.3);
      }
      
      .tag-type {
        background: rgba(33, 150, 243, 0.1);
        color: #2196f3;
        border: 1px solid rgba(33, 150, 243, 0.3);
      }
      
      .info-section {
        margin: var(--spacing-md, 12px) 0;
        padding: var(--spacing-sm, 8px);
        background: var(--color-surface, #1a1a1a);
        border-radius: 4px;
        border: 1px solid var(--color-border-light, #262626);
      }
      
      .info-section h4 {
        margin: 0 0 var(--spacing-xs, 4px) 0;
        color: var(--color-accent, #ff6b6b);
        font-size: 0.9rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
      }
      
      .info-section p {
        margin: var(--spacing-xs, 4px) 0;
        line-height: 1.5;
      }
      
      .content-section {
        margin: var(--spacing-lg, 16px) 0;
      }
      
      .content-section h3 {
        color: var(--color-primary, #4a9eff);
        margin-bottom: var(--spacing-sm, 8px);
      }
      
      .content-section h4 {
        color: var(--color-accent, #ff6b6b);
        margin-top: var(--spacing-md, 12px);
        margin-bottom: var(--spacing-xs, 4px);
      }
      
      /* Trait blocks for cult abilities */
      .entries {
        margin: var(--spacing-sm, 8px) 0;
      }
      
      .entries > h4 {
        color: #ffc107;
        margin: var(--spacing-md, 12px) 0 var(--spacing-xs, 4px) 0;
      }
      
      .subsection {
        margin-left: var(--spacing-md, 12px);
        padding-left: var(--spacing-sm, 8px);
        border-left: 2px solid var(--color-border-light, #262626);
      }
      
      .source-info {
        margin-top: var(--spacing-lg, 16px);
        padding-top: var(--spacing-md, 12px);
        border-top: 1px solid var(--color-border, #333);
        color: var(--color-text-secondary, #999);
        font-size: 0.9rem;
      }
      
      /* Special formatting for demonic/diabolical traits */
      .trait-block {
        margin: var(--spacing-md, 12px) 0;
        padding: var(--spacing-sm, 8px);
        background: rgba(255, 87, 34, 0.05);
        border-left: 3px solid #ff5722;
        border-radius: 4px;
      }
      
      .trait-block h4 {
        margin: 0 0 var(--spacing-xs, 4px) 0;
        color: #ff5722;
      }
      
      ul, ol {
        margin: var(--spacing-sm, 8px) 0;
        padding-left: var(--spacing-lg, 16px);
      }
      
      li {
        margin: var(--spacing-xs, 4px) 0;
        line-height: 1.5;
      }
    </style>
  `
}