import { processFormattingTags } from '../utils/textFormatting'

export function formatOptionalFeatureDetails(feature: any): string {
  if (!feature) return '<p>No details available</p>'

  let html = '<div class="optional-feature-details">'
  
  // Header
  html += '<div class="feature-header">'
  html += `<h1>${feature.name}</h1>`
  
  // Feature type badges
  if (feature.feature_type) {
    html += '<div class="feature-types">'
    for (const type of feature.feature_type) {
      const displayName = getFeatureTypeDisplay(type)
      html += `<span class="type-badge">${displayName}</span>`
    }
    html += '</div>'
  }
  html += '</div>'
  
  // Source and page
  html += '<div class="source-info">'
  html += `<strong>Source:</strong> ${feature.source}`
  if (feature.page) {
    html += ` p.${feature.page}`
  }
  html += '</div>'
  
  // Prerequisites
  if (feature.prerequisite && feature.prerequisite.length > 0) {
    html += '<div class="prerequisites">'
    html += '<h3>Prerequisites</h3>'
    html += '<ul>'
    for (const prereq of feature.prerequisite) {
      const prereqText = formatPrerequisite(prereq)
      if (prereqText) {
        html += `<li>${prereqText}</li>`
      }
    }
    html += '</ul>'
    html += '</div>'
  }
  
  // Main description
  if (feature.entries && feature.entries.length > 0) {
    html += '<div class="feature-description">'
    for (const entry of feature.entries) {
      html += formatEntry(entry)
    }
    html += '</div>'
  } else {
    // Fallback - show summary if no entries
    html += '<div class="feature-description">'
    html += '<p><em>No detailed description available.</em></p>'
    html += '</div>'
  }
  
  // Additional spells
  if (feature.additionalSpells && feature.additionalSpells.length > 0) {
    html += '<div class="additional-spells">'
    html += '<h3>Additional Spells</h3>'
    for (const spellGroup of feature.additionalSpells) {
      html += formatAdditionalSpells(spellGroup)
    }
    html += '</div>'
  }
  
  // Consumes resources
  if (feature.consumes) {
    html += '<div class="consumes">'
    html += `<p class="consumes-text">Consumes: ${feature.consumes.name}`
    if (feature.consumes.amount) {
      html += ` (${feature.consumes.amount})`
    }
    html += '</p>'
    html += '</div>'
  }
  
  html += '</div>'
  
  // Add styles
  html += `
    <style>
      .optional-feature-details {
        padding: 20px;
        max-width: 800px;
        line-height: 1.6;
      }
      
      .feature-header {
        margin-bottom: 16px;
        padding-bottom: 12px;
        border-bottom: 2px solid var(--color-border, #333);
      }
      
      .feature-header h1 {
        margin: 0 0 8px 0;
        color: var(--color-primary, #4a9eff);
      }
      
      .feature-types {
        display: flex;
        gap: 8px;
        flex-wrap: wrap;
      }
      
      .type-badge {
        padding: 4px 10px;
        background: rgba(74, 158, 255, 0.2);
        color: var(--color-primary, #4a9eff);
        border-radius: 4px;
        font-size: 0.85em;
        font-weight: 500;
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
      
      .prerequisites {
        margin: 20px 0;
        padding: 16px;
        background: var(--color-background-secondary, #1a1a1a);
        border-radius: 6px;
      }
      
      .prerequisites h3 {
        margin: 0 0 12px 0;
        color: var(--color-text-secondary, #999);
        font-size: 0.9em;
        text-transform: uppercase;
        letter-spacing: 1px;
      }
      
      .prerequisites ul {
        margin: 0;
        padding-left: 20px;
      }
      
      .prerequisites li {
        margin: 4px 0;
        color: var(--color-text, #e0e0e0);
      }
      
      .feature-description {
        margin: 20px 0;
      }
      
      .feature-description p {
        margin: 12px 0;
      }
      
      .feature-description ul {
        margin: 12px 0;
        padding-left: 24px;
      }
      
      .feature-description li {
        margin: 6px 0;
      }
      
      .feature-description .subsection {
        margin: 20px 0;
        padding-left: 16px;
        border-left: 3px solid var(--color-border, #333);
      }
      
      .feature-description .subsection h4 {
        margin: 0 0 12px 0;
        color: var(--color-primary, #4a9eff);
        font-size: 1.1em;
      }
      
      .additional-spells {
        margin: 20px 0;
        padding: 16px;
        background: var(--color-background-secondary, #1a1a1a);
        border-radius: 6px;
      }
      
      .additional-spells h3 {
        margin: 0 0 12px 0;
        color: var(--color-primary, #4a9eff);
      }
      
      .spell-level {
        margin: 12px 0;
      }
      
      .spell-level strong {
        color: var(--color-text-secondary, #999);
      }
      
      .spell-list {
        margin: 4px 0 4px 20px;
        color: var(--color-text, #e0e0e0);
      }
      
      .consumes {
        margin-top: 16px;
        padding: 8px 12px;
        background: rgba(255, 193, 7, 0.1);
        border-left: 3px solid #ffc107;
        border-radius: 3px;
      }
      
      .consumes-text {
        margin: 0;
        color: #ffc107;
        font-size: 0.9em;
      }
      
      .hanging-list {
        list-style: none;
        padding-left: 0;
      }
      
      .hanging-list li {
        text-indent: -1.5em;
        padding-left: 1.5em;
        margin: 8px 0;
      }
      
      .feature-table {
        width: 100%;
        margin: 16px 0;
        border-collapse: collapse;
      }
      
      .feature-table caption {
        padding: 8px;
        font-style: italic;
        color: var(--color-text-secondary, #999);
      }
      
      .feature-table th {
        background: var(--color-background-secondary, #1a1a1a);
        padding: 8px;
        text-align: left;
        border: 1px solid var(--color-border, #333);
        font-weight: 600;
      }
      
      .feature-table td {
        padding: 8px;
        border: 1px solid var(--color-border, #333);
      }
      
      .feature-table tbody tr:hover {
        background: rgba(74, 158, 255, 0.05);
      }
      
      .options-section {
        margin: 20px 0;
        padding: 16px;
        background: var(--color-background-secondary, #1a1a1a);
        border-radius: 6px;
      }
      
      .option-count {
        margin: 0 0 16px 0;
        color: var(--color-primary, #4a9eff);
        font-weight: 600;
      }
      
      .option-item {
        margin: 12px 0;
        padding: 12px;
        background: var(--color-background, #0d0d0d);
        border-left: 3px solid var(--color-primary, #4a9eff);
        border-radius: 3px;
      }
      
      .inset-box {
        margin: 20px 0;
        padding: 16px;
        background: var(--color-background-tertiary, #262626);
        border: 1px solid var(--color-border, #333);
        border-radius: 6px;
      }
      
      .inset-box.insetReadaloud {
        background: rgba(74, 158, 255, 0.05);
        border-color: var(--color-primary, #4a9eff);
        font-style: italic;
      }
      
      .inset-title {
        margin: 0 0 12px 0;
        color: var(--color-primary, #4a9eff);
        font-size: 1.1em;
      }
      
      .ability-calculation {
        margin: 16px 0;
        padding: 12px;
        background: rgba(74, 158, 255, 0.1);
        border-left: 3px solid var(--color-primary, #4a9eff);
        border-radius: 3px;
      }
      
      .ability-calculation p {
        margin: 0;
      }
      
      .feature-quote {
        margin: 20px 0;
        padding: 16px 20px;
        border-left: 4px solid var(--color-primary, #4a9eff);
        background: var(--color-background-secondary, #1a1a1a);
        font-style: italic;
      }
      
      .feature-quote cite {
        display: block;
        margin-top: 8px;
        text-align: right;
        font-style: normal;
        color: var(--color-text-secondary, #999);
      }
      
      .generic-entry {
        margin: 12px 0;
      }
      
      .expanded-spells {
        margin: 12px 0;
      }
      
      .innate-spells {
        margin: 12px 0;
      }
    </style>
  `
  
  return html
}

function getFeatureTypeDisplay(type: string): string {
  const typeMap: Record<string, string> = {
    'AI': 'Artificer Infusion',
    'ED': 'Elemental Discipline',
    'EI': 'Eldritch Invocation',
    'MM': 'Metamagic',
    'MV': 'Maneuver',
    'MV:B': 'Battle Master Maneuver',
    'AS': 'Arcane Shot',
    'FS:F': 'Fighting Style (Fighter)',
    'FS:B': 'Fighting Style (Bard)',
    'FS:P': 'Fighting Style (Paladin)',
    'FS:R': 'Fighting Style (Ranger)',
    'PB': 'Pact Boon',
    'OR': 'Onomancy Resonant',
    'RN': 'Rune Knight Rune',
    'AF': 'Alchemical Formula',
    'TT': "Traveler's Trick",
    'OTH': 'Other'
  }
  return typeMap[type] || type
}

function formatPrerequisite(prereq: any): string {
  const parts: string[] = []
  
  if (prereq.level) {
    if (typeof prereq.level === 'number') {
      parts.push(`Level ${prereq.level}`)
    } else if (prereq.level.level) {
      let text = ''
      if (prereq.level.class) {
        text += prereq.level.class.name
        if (prereq.level.subclass) {
          text += ` (${prereq.level.subclass.name})`
        }
      }
      text += ` Level ${prereq.level.level}`
      parts.push(text)
    }
  }
  
  if (prereq.pact) {
    parts.push(`Pact of the ${prereq.pact}`)
  }
  
  if (prereq.patron) {
    parts.push(`${prereq.patron} patron`)
  }
  
  if (prereq.spell && prereq.spell.length > 0) {
    const spells = prereq.spell.map((s: string) => 
      s.replace('#c', ' cantrip').replace('#', '')
    )
    parts.push(spells.join(' or '))
  }
  
  if (prereq.feature && prereq.feature.length > 0) {
    parts.push(prereq.feature.join(', '))
  }
  
  if (prereq.item && prereq.item.length > 0) {
    parts.push(`Requires: ${prereq.item.join(' or ')}`)
  }
  
  if (prereq.otherSummary) {
    parts.push(prereq.otherSummary)
  }
  
  return parts.join('; ')
}

function formatEntry(entry: any): string {
  if (typeof entry === 'string') {
    return `<p>${processFormattingTags(entry)}</p>`
  }
  
  if (entry && typeof entry === 'object') {
    let html = ''
    
    if (entry.type === 'entries') {
      html += '<div class="subsection">'
      if (entry.name) {
        html += `<h4>${entry.name}</h4>`
      }
      if (entry.entries) {
        for (const subEntry of entry.entries) {
          html += formatEntry(subEntry)
        }
      }
      html += '</div>'
    } else if (entry.type === 'list') {
      // Handle different list styles
      const listStyle = entry.style || 'list'
      
      if (listStyle === 'list-hang-notitle') {
        html += '<ul class="hanging-list">'
      } else {
        html += '<ul>'
      }
      
      if (entry.items) {
        for (const item of entry.items) {
          if (typeof item === 'string') {
            html += `<li>${processFormattingTags(item)}</li>`
          } else if (item.name && item.entry) {
            html += `<li><strong>${item.name}.</strong> ${processFormattingTags(item.entry)}</li>`
          } else if (item.name && item.entries) {
            html += `<li><strong>${item.name}.</strong> `
            for (const subEntry of item.entries) {
              if (typeof subEntry === 'string') {
                html += processFormattingTags(subEntry)
              } else {
                html += formatEntry(subEntry)
              }
            }
            html += '</li>'
          } else if (typeof item === 'object') {
            // Handle complex list items
            html += '<li>'
            html += formatEntry(item)
            html += '</li>'
          }
        }
      }
      html += '</ul>'
    } else if (entry.type === 'table') {
      html += formatTable(entry)
    } else if (entry.type === 'options') {
      // Handle options entries (for maneuvers, etc.)
      html += '<div class="options-section">'
      if (entry.count) {
        html += `<p class="option-count">Choose ${entry.count} of the following:</p>`
      }
      if (entry.entries) {
        for (const option of entry.entries) {
          html += '<div class="option-item">'
          html += formatEntry(option)
          html += '</div>'
        }
      }
      html += '</div>'
    } else if (entry.type === 'inset' || entry.type === 'insetReadaloud') {
      // Handle inset boxes
      html += `<div class="inset-box ${entry.type}">`
      if (entry.name) {
        html += `<h4 class="inset-title">${entry.name}</h4>`
      }
      if (entry.entries) {
        for (const insetEntry of entry.entries) {
          html += formatEntry(insetEntry)
        }
      }
      html += '</div>'
    } else if (entry.type === 'abilityDc' || entry.type === 'abilityAttackMod') {
      // Handle ability DCs and attack modifiers
      html += '<div class="ability-calculation">'
      html += `<p><strong>${entry.name || 'Save DC'}:</strong> ${entry.attributes ? entry.attributes.join(' + ') : '8 + proficiency + modifier'}</p>`
      html += '</div>'
    } else if (entry.type === 'quote') {
      html += '<blockquote class="feature-quote">'
      if (entry.entries) {
        for (const quoteEntry of entry.entries) {
          html += formatEntry(quoteEntry)
        }
      }
      if (entry.by) {
        html += `<cite>— ${entry.by}</cite>`
      }
      html += '</blockquote>'
    } else {
      // Fallback for unknown types - try to display what we can
      if (entry.name) {
        html += `<div class="generic-entry"><strong>${entry.name}:</strong> `
      }
      if (entry.entries) {
        for (const subEntry of entry.entries) {
          html += formatEntry(subEntry)
        }
      } else if (entry.entry) {
        html += processFormattingTags(entry.entry)
      }
      if (entry.name) {
        html += '</div>'
      }
    }
    
    return html
  }
  
  return ''
}

function formatTable(table: any): string {
  let html = '<table class="feature-table">'
  
  if (table.caption) {
    html += `<caption>${table.caption}</caption>`
  }
  
  if (table.colLabels) {
    html += '<thead><tr>'
    for (const label of table.colLabels) {
      html += `<th>${label}</th>`
    }
    html += '</tr></thead>'
  }
  
  if (table.rows) {
    html += '<tbody>'
    for (const row of table.rows) {
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
  return html
}

function formatAdditionalSpells(spellGroup: any): string {
  let html = ''
  
  if (spellGroup.expanded) {
    html += '<div class="expanded-spells">'
    if (spellGroup.expanded.s1) {
      html += formatSpellLevel('1st', spellGroup.expanded.s1)
    }
    if (spellGroup.expanded.s2) {
      html += formatSpellLevel('2nd', spellGroup.expanded.s2)
    }
    if (spellGroup.expanded.s3) {
      html += formatSpellLevel('3rd', spellGroup.expanded.s3)
    }
    if (spellGroup.expanded.s4) {
      html += formatSpellLevel('4th', spellGroup.expanded.s4)
    }
    if (spellGroup.expanded.s5) {
      html += formatSpellLevel('5th', spellGroup.expanded.s5)
    }
    html += '</div>'
  } else if (spellGroup.innate) {
    html += '<div class="innate-spells">'
    if (spellGroup.innate._) {
      const atWillSpells = Array.isArray(spellGroup.innate._) 
        ? spellGroup.innate._.join(', ') 
        : spellGroup.innate._
      html += '<div class="spell-list">At will: ' + atWillSpells + '</div>'
    }
    for (const [key, value] of Object.entries(spellGroup.innate)) {
      if (key !== '_' && key !== 'ritual') {
        const uses = key.replace('e', '/long rest').replace('s', '/short rest')
        const spellList = Array.isArray(value) ? value.join(', ') : value
        html += `<div class="spell-list">${uses}: ${spellList}</div>`
      }
    }
    html += '</div>'
  }
  
  return html
}

function formatSpellLevel(level: string, spells: string[]): string {
  return `
    <div class="spell-level">
      <strong>${level} Level:</strong>
      <span class="spell-list">${spells.join(', ')}</span>
    </div>
  `
}