import type { Reward } from '../composables/catalog'
import { processFormattingTags } from '../utils/textFormatting'

export async function formatRewardDetails(reward: Reward): Promise<string> {
  if (!reward) return '<div>No reward data available</div>'
  
  let html = '<div class="reward-details">'
  
  // Header with name and type
  html += `
    <div class="detail-header">
      <h2>${reward.name}</h2>
      <div class="header-tags">
        ${reward.reward_type ? `<span class="tag tag-type ${getRewardTypeClass(reward.reward_type)}">${formatRewardType(reward.reward_type)}</span>` : ''}
        ${reward.duration ? `<span class="tag tag-duration">Duration: ${reward.duration}</span>` : ''}
        ${reward.srd ? '<span class="tag tag-srd">SRD</span>' : ''}
        ${reward.basic_rules ? '<span class="tag tag-basic">Basic Rules</span>' : ''}
      </div>
    </div>
  `
  
  // Prerequisites
  if (reward.prerequisite && reward.prerequisite.length > 0) {
    html += '<div class="content-section prerequisites">'
    html += '<h3>Prerequisites</h3>'
    for (const prereq of reward.prerequisite) {
      html += formatEntry(prereq)
    }
    html += '</div>'
  }
  
  // Main description/entries
  if (reward.entries && reward.entries.length > 0) {
    html += '<div class="content-section">'
    html += '<h3>Description</h3>'
    for (const entry of reward.entries) {
      html += formatEntry(entry)
    }
    html += '</div>'
  }
  
  // Additional spells granted
  if (reward.additional_spells && reward.additional_spells.length > 0) {
    html += '<div class="content-section">'
    html += '<h3>Spells Granted</h3>'
    html += formatSpellList(reward.additional_spells)
    html += '</div>'
  }
  
  // Source info
  html += `
    <div class="source-info">
      <strong>Source:</strong> ${reward.source}
      ${reward.page ? `, p. ${reward.page}` : ''}
    </div>
  `
  
  html += '</div>'
  
  // Add styles
  html += `
    <style>
      .reward-details {
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
      
      .tag-type.blessing {
        background: rgba(255, 215, 0, 0.2);
        color: #ffd700;
        border: 1px solid rgba(255, 215, 0, 0.4);
      }
      
      .tag-type.boon {
        background: rgba(147, 112, 219, 0.2);
        color: #9370db;
        border: 1px solid rgba(147, 112, 219, 0.4);
      }
      
      .tag-type.charm {
        background: rgba(255, 105, 180, 0.2);
        color: #ff69b4;
        border: 1px solid rgba(255, 105, 180, 0.4);
      }
      
      .tag-type.feat {
        background: rgba(70, 130, 180, 0.2);
        color: #4682b4;
        border: 1px solid rgba(70, 130, 180, 0.4);
      }
      
      .tag-duration {
        background: rgba(255, 165, 0, 0.1);
        color: #ffa500;
        border: 1px solid rgba(255, 165, 0, 0.3);
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
      
      .content-section h3 {
        color: var(--color-text, #e0e0e0);
        border-bottom: 1px solid var(--color-border, #333);
        padding-bottom: var(--spacing-xs, 4px);
        margin-bottom: var(--spacing-sm, 8px);
      }
      
      .prerequisites {
        background: rgba(255, 193, 7, 0.05);
        padding: var(--spacing-sm, 8px);
        border-radius: 4px;
        border-left: 3px solid #ffc107;
      }
      
      .spell-list {
        margin: var(--spacing-sm, 8px) 0;
      }
      
      .spell-group {
        margin: var(--spacing-xs, 4px) 0;
      }
      
      .spell-frequency {
        font-weight: bold;
        color: var(--color-primary, #4a9eff);
      }
      
      .spell-name {
        color: var(--color-accent, #ff6b6b);
        cursor: pointer;
        text-decoration: underline;
      }
      
      .spell-name:hover {
        color: var(--color-accent-hover, #ff8787);
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

function formatRewardType(type: string): string {
  const typeMap: Record<string, string> = {
    'blessing': 'Blessing',
    'boon': 'Epic Boon',
    'charm': 'Charm',
    'feat': 'Feat'
  }
  return typeMap[type.toLowerCase()] || type
}

function getRewardTypeClass(type: string): string {
  const cleanType = type.toLowerCase().replace(/\s+/g, '-')
  if (cleanType.includes('boon')) return 'boon'
  if (cleanType.includes('blessing')) return 'blessing'
  if (cleanType.includes('charm')) return 'charm'
  if (cleanType.includes('feat')) return 'feat'
  return ''
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
  let html = '<table class="reward-table">'
  
  // Headers
  if (table.colLabels) {
    html += '<thead><tr>'
    for (const label of table.colLabels) {
      html += `<th>${processFormattingTags(label)}</th>`
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

function formatSpellList(spellData: any[]): string {
  let html = '<div class="spell-list">'
  
  for (const spellGroup of spellData) {
    if (spellGroup.innate) {
      html += formatInnateSpells(spellGroup.innate)
    } else if (spellGroup.daily) {
      html += formatDailySpells(spellGroup.daily)
    } else if (spellGroup.rest) {
      html += formatRestSpells(spellGroup.rest)
    }
  }
  
  html += '</div>'
  return html
}

function formatInnateSpells(innate: any): string {
  let html = ''
  
  if (innate._) {
    if (innate._.rest) {
      for (const [freq, spells] of Object.entries(innate._.rest)) {
        html += `<div class="spell-group">`
        html += `<span class="spell-frequency">${formatFrequency(freq)}:</span> `
        html += formatSpellNames(spells as string[])
        html += `</div>`
      }
    }
    if (innate._.daily) {
      for (const [freq, spells] of Object.entries(innate._.daily)) {
        html += `<div class="spell-group">`
        html += `<span class="spell-frequency">${formatFrequency(freq)}:</span> `
        html += formatSpellNames(spells as string[])
        html += `</div>`
      }
    }
  }
  
  return html
}

function formatDailySpells(daily: any): string {
  let html = ''
  
  for (const [freq, spells] of Object.entries(daily)) {
    html += `<div class="spell-group">`
    html += `<span class="spell-frequency">${formatFrequency(freq)}:</span> `
    html += formatSpellNames(spells as string[])
    html += `</div>`
  }
  
  return html
}

function formatRestSpells(rest: any): string {
  let html = ''
  
  for (const [freq, spells] of Object.entries(rest)) {
    html += `<div class="spell-group">`
    html += `<span class="spell-frequency">${formatFrequency(freq)}:</span> `
    html += formatSpellNames(spells as string[])
    html += `</div>`
  }
  
  return html
}

function formatFrequency(freq: string): string {
  if (freq.match(/^\d+$/)) {
    return `${freq}/day`
  }
  if (freq.match(/^\d+e$/)) {
    const num = freq.slice(0, -1)
    return `${num}/day each`
  }
  return freq
}

function formatSpellNames(spells: string[]): string {
  return spells
    .map(spell => `<span class="spell-name reference-link" data-ref-type="spell" data-ref-name="${spell}">${spell}</span>`)
    .join(', ')
}