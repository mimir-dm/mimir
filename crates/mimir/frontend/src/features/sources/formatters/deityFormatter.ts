import type { Deity } from '../composables/catalog'
import { formatEntries } from '../utils/textFormatting'

export function formatDeityContent(deity: Deity): string {
  let html = '<div class="deity-content">'
  
  // Enhanced header with visual hierarchy
  html += '<div class="deity-header">'
  
  // Main title section
  html += '<div class="deity-title-section">'
  html += `<h2 class="deity-name">${deity.name}</h2>`
  if (deity.title) {
    html += `<div class="deity-title">${deity.title}</div>`
  }
  html += '</div>'
  
  // Core attributes grid
  html += '<div class="deity-attributes-grid">'
  
  if (deity.pantheon) {
    html += `
      <div class="attribute-card">
        <div class="attribute-label">Pantheon</div>
        <div class="attribute-value pantheon-value">${deity.pantheon}</div>
      </div>
    `
  }
  
  if (deity.alignment && deity.alignment.length > 0) {
    const alignmentText = formatAlignment(deity.alignment)
    const alignmentClass = getAlignmentClass(deity.alignment)
    html += `
      <div class="attribute-card">
        <div class="attribute-label">Alignment</div>
        <div class="attribute-value alignment-value ${alignmentClass}">${alignmentText}</div>
      </div>
    `
  }
  
  if (deity.symbol) {
    html += `
      <div class="attribute-card symbol-card">
        <div class="attribute-label">Holy Symbol</div>
        <div class="attribute-value symbol-value">${deity.symbol}</div>
      </div>
    `
  }
  
  html += '</div>' // Close attributes grid
  
  // Domains section with enhanced styling
  if (deity.domains && deity.domains.length > 0) {
    html += '<div class="deity-domains-section">'
    html += '<h3 class="section-title">Divine Domains</h3>'
    html += '<div class="domains-list">'
    deity.domains.forEach(domain => {
      const domainClass = getDomainClass(domain)
      html += `<span class="domain-tag ${domainClass}">${domain}</span>`
    })
    html += '</div>'
    html += '</div>'
  }
  
  html += '</div>' // Close header
  
  // Main content/description with rich formatting
  if (deity.entries && deity.entries.length > 0) {
    html += '<div class="deity-description">'
    html += '<h3 class="section-title">Description & Lore</h3>'
    html += '<div class="description-content">'
    html += formatEntries(deity.entries)
    html += '</div>'
    html += '</div>'
  } else {
    // No description available - provide helpful message
    html += '<div class="deity-description">'
    html += '<h3 class="section-title">Description & Lore</h3>'
    html += '<div class="description-content no-description">'
    html += '<p class="no-description-message">No detailed description available for this deity.</p>'
    html += '<p class="description-hint">The source material may only provide basic information about this deity\'s alignment, domains, and symbols.</p>'
    if (deity.page) {
      html += `<p class="description-reference">Refer to ${deity.source} page ${deity.page} for more information.</p>`
    }
    html += '</div>'
    html += '</div>'
  }
  
  // Additional sources section if available
  if (deity.additionalSources && deity.additionalSources.length > 0) {
    html += '<div class="additional-sources">'
    html += '<h4 class="subsection-title">Additional References</h4>'
    html += '<ul class="sources-list">'
    deity.additionalSources.forEach((src: any) => {
      html += `<li>${src.source}${src.page ? `, page ${src.page}` : ''}</li>`
    })
    html += '</ul>'
    html += '</div>'
  }
  
  // Footer with enhanced information
  html += '<div class="deity-footer">'
  
  html += '<div class="footer-left">'
  if (deity.source) {
    html += `<span class="source-badge">${deity.source}</span>`
  }
  if (deity.page) {
    html += `<span class="page-reference">Page ${deity.page}</span>`
  }
  html += '</div>'
  
  html += '<div class="footer-right">'
  if (deity.hasFluff) {
    html += '<span class="has-lore-indicator">📖</span>'
  }
  if (deity.hasFluffImages) {
    html += '<span class="has-images-indicator">🖼️</span>'
  }
  html += '</div>'
  
  html += '</div>'
  html += '</div>'
  
  // Add enhanced custom styles
  html += `
    <style>
      .deity-content {
        padding: 20px;
        color: var(--color-text, #e0e0e0);
        max-width: 900px;
        margin: 0 auto;
      }
      
      .deity-header {
        margin-bottom: 24px;
        padding-bottom: 20px;
        border-bottom: 2px solid var(--color-border, #333);
      }
      
      .deity-title-section {
        margin-bottom: 20px;
        text-align: center;
      }
      
      .deity-name {
        font-size: 2rem;
        color: var(--color-primary, #4a9eff);
        margin: 0 0 8px 0;
        font-weight: 700;
        letter-spacing: 0.5px;
      }
      
      .deity-title {
        font-size: 1.1rem;
        color: var(--color-text-secondary, #999);
        font-style: italic;
        margin: 0;
      }
      
      .deity-attributes-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 16px;
        margin-bottom: 24px;
      }
      
      .attribute-card {
        background: var(--color-surface, #1a1a1a);
        border: 1px solid var(--color-border, #333);
        border-radius: 8px;
        padding: 12px;
        transition: all 0.2s ease;
      }
      
      .attribute-card:hover {
        background: var(--color-surface-hover, #252525);
        border-color: var(--color-primary, #4a9eff);
      }
      
      .attribute-label {
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 1px;
        color: var(--color-text-secondary, #999);
        margin-bottom: 6px;
        font-weight: 600;
      }
      
      .attribute-value {
        font-size: 1rem;
        font-weight: 500;
      }
      
      .pantheon-value {
        color: #9b59b6;
      }
      
      .alignment-value {
        font-weight: 600;
      }
      
      .alignment-value.good { color: #3498db; }
      .alignment-value.evil { color: #e74c3c; }
      .alignment-value.lawful { color: #f39c12; }
      .alignment-value.chaotic { color: #9b59b6; }
      .alignment-value.neutral { color: #95a5a6; }
      
      .symbol-card {
        grid-column: span 2;
      }
      
      .symbol-value {
        color: #f1c40f;
        font-style: italic;
      }
      
      .deity-domains-section {
        margin-bottom: 24px;
      }
      
      .section-title {
        font-size: 1.1rem;
        color: var(--color-primary, #4a9eff);
        margin: 0 0 12px 0;
        font-weight: 600;
        letter-spacing: 0.5px;
        border-bottom: 1px solid rgba(74, 158, 255, 0.3);
        padding-bottom: 6px;
      }
      
      .domains-list {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
      }
      
      .domain-tag {
        padding: 6px 12px;
        background: var(--color-surface, #1a1a1a);
        border: 1px solid var(--color-border, #333);
        border-radius: 20px;
        font-size: 0.9rem;
        font-weight: 500;
        transition: all 0.2s ease;
      }
      
      .domain-tag:hover {
        background: var(--color-surface-hover, #252525);
        transform: translateY(-2px);
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
      }
      
      .domain-tag.knowledge { border-color: #3498db; color: #3498db; }
      .domain-tag.war { border-color: #e74c3c; color: #e74c3c; }
      .domain-tag.death { border-color: #8e44ad; color: #8e44ad; }
      .domain-tag.life { border-color: #2ecc71; color: #2ecc71; }
      .domain-tag.light { border-color: #f1c40f; color: #f1c40f; }
      .domain-tag.nature { border-color: #27ae60; color: #27ae60; }
      .domain-tag.tempest { border-color: #34495e; color: #34495e; }
      .domain-tag.trickery { border-color: #9b59b6; color: #9b59b6; }
      .domain-tag.forge { border-color: #e67e22; color: #e67e22; }
      .domain-tag.grave { border-color: #7f8c8d; color: #7f8c8d; }
      .domain-tag.order { border-color: #d35400; color: #d35400; }
      .domain-tag.peace { border-color: #16a085; color: #16a085; }
      .domain-tag.twilight { border-color: #2c3e50; color: #2c3e50; }
      
      .deity-description {
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
      
      .description-content blockquote {
        margin: 16px 0;
        padding: 12px 16px;
        border-left: 3px solid var(--color-primary, #4a9eff);
        background: rgba(74, 158, 255, 0.1);
        font-style: italic;
      }
      
      .description-content.no-description {
        background: rgba(255, 255, 255, 0.01);
        border-left-color: var(--color-text-secondary, #999);
        font-style: italic;
        color: var(--color-text-secondary, #999);
      }
      
      .no-description-message {
        font-size: 1rem;
        margin-bottom: 8px;
      }
      
      .description-hint {
        font-size: 0.9rem;
        opacity: 0.8;
      }
      
      .description-reference {
        font-size: 0.9rem;
        color: var(--color-primary, #4a9eff);
        margin-top: 12px;
      }
      
      .additional-sources {
        margin: 20px 0;
        padding: 12px;
        background: var(--color-surface, #1a1a1a);
        border-radius: 6px;
      }
      
      .subsection-title {
        font-size: 0.9rem;
        color: var(--color-text-secondary, #999);
        margin: 0 0 8px 0;
        font-weight: 600;
      }
      
      .sources-list {
        margin: 0;
        padding-left: 20px;
        font-size: 0.85rem;
      }
      
      .sources-list li {
        margin: 4px 0;
      }
      
      .deity-footer {
        margin-top: 24px;
        padding-top: 16px;
        border-top: 1px solid var(--color-border, #333);
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-size: 0.85rem;
      }
      
      .footer-left,
      .footer-right {
        display: flex;
        align-items: center;
        gap: 12px;
      }
      
      .source-badge {
        padding: 4px 8px;
        background: var(--color-surface, #1a1a1a);
        border: 1px solid var(--color-border, #333);
        border-radius: 4px;
        font-family: monospace;
        font-weight: 600;
      }
      
      .page-reference {
        color: var(--color-text-secondary, #999);
        font-style: italic;
      }
      
      .srd-indicator {
        padding: 4px 8px;
        background: var(--color-primary, #4a9eff);
        color: var(--color-background, #0d0d0d);
        border-radius: 4px;
        font-weight: 600;
        font-size: 0.8rem;
      }
      
      .has-lore-indicator,
      .has-images-indicator {
        font-size: 1.1rem;
        opacity: 0.7;
        transition: opacity 0.2s;
      }
      
      .has-lore-indicator:hover,
      .has-images-indicator:hover {
        opacity: 1;
      }
    </style>
  `
  
  return html
}

function formatAlignment(alignment: string[] | string): string {
  // Handle both array format (from JSON) and string format (from database)
  if (typeof alignment === 'string') {
    return alignment // Already formatted by database
  }
  
  if (!Array.isArray(alignment)) {
    return 'Unknown'
  }
  
  return alignment.map(a => {
    switch (a) {
      case 'L': return 'Lawful'
      case 'N': return 'Neutral'
      case 'C': return 'Chaotic'
      case 'G': return 'Good'
      case 'E': return 'Evil'
      case 'U': return 'Unaligned'
      case 'A': return 'Any'
      default: return a
    }
  }).join(' ')
}

function getAlignmentClass(alignment: string[] | string): string {
  // Handle both array format (from JSON) and string format (from database)
  let alignStr: string
  if (typeof alignment === 'string') {
    alignStr = alignment.toLowerCase()
  } else if (Array.isArray(alignment)) {
    alignStr = alignment.join('').toLowerCase()
  } else {
    return 'neutral'
  }
  
  if (alignStr.includes('g') || alignStr.includes('good')) return 'good'
  if (alignStr.includes('e') || alignStr.includes('evil')) return 'evil'
  if (alignStr.includes('l') || alignStr.includes('lawful')) return 'lawful'
  if (alignStr.includes('c') || alignStr.includes('chaotic')) return 'chaotic'
  return 'neutral'
}

function getDomainClass(domain: string): string {
  const lowerDomain = domain.toLowerCase()
  const domainClasses: Record<string, string> = {
    'knowledge': 'knowledge',
    'war': 'war',
    'death': 'death',
    'life': 'life',
    'light': 'light',
    'nature': 'nature',
    'tempest': 'tempest',
    'trickery': 'trickery',
    'forge': 'forge',
    'grave': 'grave',
    'order': 'order',
    'peace': 'peace',
    'twilight': 'twilight'
  }
  return domainClasses[lowerDomain] || ''
}