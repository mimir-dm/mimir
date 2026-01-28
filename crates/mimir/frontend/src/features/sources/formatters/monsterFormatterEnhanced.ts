import type { MonsterSummary } from '../composables/catalog'
import { formatEntries } from '../utils/textFormatting'
import { invoke } from '@tauri-apps/api/core'
import { formatCR } from '@/utils/formatters'

interface Monster {
  // Core fields
  name: string
  size: string | string[]  // Can be either
  type: string | any  // Can be object
  alignment?: string | any  // Optional like in useCatalog
  cr: string
  hp: string | any
  ac: string | any
  speed: string | any
  source: string
  
  // Ability scores
  str: number
  dex: number
  con: number
  int: number
  wis: number
  cha: number
  
  // Optional fields
  senses?: string | string[]
  languages?: string | string[]
  description?: string
  creature_type?: string
  environment?: string[]
  passive?: number
  save?: any
  skill?: any
  
  // Actions (renamed from trait to avoid JS keyword conflict)
  trait?: any[]
  action?: any[]
  reaction?: any[]
  legendary?: any[]
  
  // Fluff content (camelCase from backend)
  fluffEntries?: any[]
  fluffImages?: any[]
  fluff_images?: any[]  // Alternative naming
  
  // Source information
  page?: number
}

export async function formatMonsterDetails(monster: Monster | MonsterSummary): Promise<string> {
  // Check if we have full monster data or just summary
  const isFullMonster = 'str' in monster || 'fluffEntries' in monster
  
  if (!isFullMonster) {
    return formatMonsterSummary(monster as MonsterSummary)
  }
  
  const fullMonster = monster as Monster
  let html = '<div class="monster-details">'
  
  // Images (if available)
  if (fullMonster.fluffImages && fullMonster.fluffImages.length > 0) {
    html += '<div class="monster-images">'
    for (const img of fullMonster.fluffImages) {
      if (img.href?.path) {
        // Image path format: "bestiary/MM/Aarakocra.webp"
        const bookSource = fullMonster.source || 'MM'
        try {
          const response = await invoke<any>('serve_book_image', {
            bookId: bookSource,
            imagePath: img.href.path
          })
          // Response is ApiResponse<string> with success/data/error fields
          if (response && response.success && response.data) {
            html += `<img src="${response.data}" alt="${fullMonster.name}" class="monster-image" style="max-width: 300px; max-height: 300px; width: auto; height: auto; object-fit: contain; display: block; margin: 0 auto 1rem;" />`
          } else if (response && !response.success) {
          }
        } catch (e) {
        }
      }
    }
    html += '</div>'
  } else {
  }
  
  // Header with type and CR
  html += '<div class="monster-header" style="border-bottom: 2px solid #8b0000; padding-bottom: 0.5rem; margin-bottom: 1rem;">'
  // Use raw 'type' field if available (full monster), otherwise use formatted 'creature_type' (summary)
  const creatureType = fullMonster.type ? formatCreatureType(fullMonster.type) : formatCreatureType(fullMonster.creature_type)
  html += `<div class="monster-type" style="font-size: 1rem; font-style: italic;">${formatSize(fullMonster.size || 'M')} ${creatureType}, ${formatAlignment(fullMonster.alignment)}</div>`
  html += `<div class="monster-cr" style="font-size: 0.9rem; margin-top: 0.25rem;">Challenge ${formatCR(fullMonster.cr)}</div>`
  html += '</div>'
  
  // Core stats bar
  html += '<div class="stat-block" style="background: rgba(139, 0, 0, 0.1); padding: 0.75rem; border-radius: 4px; margin-bottom: 1rem;">'
  html += '<div class="stat-row" style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 1rem;">'
  html += `<div><strong>AC</strong><br>${formatAC(fullMonster.ac)}</div>`
  html += `<div><strong>HP</strong><br>${formatHP(fullMonster.hp)}</div>`
  html += `<div><strong>Speed</strong><br>${formatSpeed(fullMonster.speed)}</div>`
  html += '</div>'
  html += '</div>'
  
  // Ability scores
  if (fullMonster.str) {
    html += '<div class="ability-scores" style="margin-bottom: 1rem;">'
    html += '<table style="width: 100%; text-align: left; border-collapse: collapse;">'
    html += '<tr style="border-bottom: 1px solid #333;">'
    html += '<th style="padding: 0.5rem; font-weight: bold;">STR</th>'
    html += '<th style="padding: 0.5rem; font-weight: bold;">DEX</th>'
    html += '<th style="padding: 0.5rem; font-weight: bold;">CON</th>'
    html += '<th style="padding: 0.5rem; font-weight: bold;">INT</th>'
    html += '<th style="padding: 0.5rem; font-weight: bold;">WIS</th>'
    html += '<th style="padding: 0.5rem; font-weight: bold;">CHA</th>'
    html += '</tr>'
    html += '<tr>'
    html += `<td style="padding: 0.5rem;">${formatAbilityScore(fullMonster.str)}</td>`
    html += `<td style="padding: 0.5rem;">${formatAbilityScore(fullMonster.dex)}</td>`
    html += `<td style="padding: 0.5rem;">${formatAbilityScore(fullMonster.con)}</td>`
    html += `<td style="padding: 0.5rem;">${formatAbilityScore(fullMonster.int)}</td>`
    html += `<td style="padding: 0.5rem;">${formatAbilityScore(fullMonster.wis)}</td>`
    html += `<td style="padding: 0.5rem;">${formatAbilityScore(fullMonster.cha)}</td>`
    html += '</tr>'
    html += '</table>'
    html += '</div>'
  }
  
  // Skills, saves, senses
  const details: string[] = []
  if (fullMonster.save) {
    details.push(`<strong>Saving Throws:</strong> ${formatSaves(fullMonster.save)}`)
  }
  if (fullMonster.skill) {
    details.push(`<strong>Skills:</strong> ${formatSkills(fullMonster.skill)}`)
  }
  if (fullMonster.senses) {
    const sensesStr = Array.isArray(fullMonster.senses) ? fullMonster.senses.join(', ') : fullMonster.senses
    details.push(`<strong>Senses:</strong> ${sensesStr}`)
  }
  if (fullMonster.languages) {
    const langStr = Array.isArray(fullMonster.languages) ? fullMonster.languages.join(', ') : fullMonster.languages
    details.push(`<strong>Languages:</strong> ${langStr}`)
  }
  
  if (details.length > 0) {
    html += '<div class="monster-details-section" style="margin-bottom: 1rem; padding: 0.5rem 0; border-bottom: 1px solid #333;">'
    html += details.join('<br>')
    html += '</div>'
  }
  
  // Traits
  if (fullMonster.trait && fullMonster.trait.length > 0) {
    html += '<div class="monster-section" style="margin-bottom: 1rem;">'
    html += '<h4 style="color: #8b0000; margin-bottom: 0.5rem; font-size: 1.1rem;">Traits</h4>'
    for (const trait of fullMonster.trait) {
      html += formatAction(trait)
    }
    html += '</div>'
  }
  
  // Actions
  if (fullMonster.action && fullMonster.action.length > 0) {
    html += '<div class="monster-section" style="margin-bottom: 1rem;">'
    html += '<h4 style="color: #8b0000; margin-bottom: 0.5rem; font-size: 1.1rem;">Actions</h4>'
    for (const action of fullMonster.action) {
      html += formatAction(action)
    }
    html += '</div>'
  }
  
  // Reactions
  if (fullMonster.reaction && fullMonster.reaction.length > 0) {
    html += '<div class="monster-section">'
    html += '<h4>Reactions</h4>'
    for (const reaction of fullMonster.reaction) {
      html += formatAction(reaction)
    }
    html += '</div>'
  }
  
  // Legendary Actions
  if (fullMonster.legendary && fullMonster.legendary.length > 0) {
    html += '<div class="monster-section">'
    html += '<h4>Legendary Actions</h4>'
    for (const legendary of fullMonster.legendary) {
      html += formatAction(legendary)
    }
    html += '</div>'
  }
  
  // Fluff/Lore (if available)
  if (fullMonster.fluffEntries && fullMonster.fluffEntries.length > 0) {
    html += '<div class="monster-lore">'
    html += '<h4>Lore</h4>'
    html += formatEntries(fullMonster.fluffEntries)
    html += '</div>'
  }
  
  // Environment
  if (fullMonster.environment && fullMonster.environment.length > 0) {
    html += '<div class="monster-environment">'
    html += `<h4>Environment:</h4>`
    html += `<p>${fullMonster.environment.join(', ')}</p>`
    html += '</div>'
  }
  
  // Source with page - styled to bottom right
  html += '<div style="text-align: right; margin-top: 1.5rem; padding-top: 1rem; border-top: 1px solid var(--color-border-light, #222);">'
  html += `<span class="source-info">Source: ${fullMonster.source}`
  if (fullMonster.page) {
    html += `, p. ${fullMonster.page}`
  }
  html += '</span>'
  html += '</div>'
  html += '</div>'
  
  return html
}

function formatMonsterSummary(monster: MonsterSummary): string {
  let html = '<div class="monster-details">'
  
  // Header with type and CR
  html += '<div class="monster-header">'
  html += `<div class="monster-type">${formatSize(monster.size)} ${monster.creature_type}, ${monster.alignment}</div>`
  html += `<div class="monster-cr">Challenge ${monster.cr}</div>`
  html += '</div>'
  
  // Core stats
  html += '<div class="creature-stats">'
  html += '<div class="stat">'
  html += '<div class="stat-label">AC</div>'
  html += `<div class="stat-value">${monster.ac}</div>`
  html += '</div>'
  html += '<div class="stat">'
  html += '<div class="stat-label">HP</div>'
  html += `<div class="stat-value">${monster.hp}</div>`
  html += '</div>'
  html += '<div class="stat">'
  html += '<div class="stat-label">CR</div>'
  html += `<div class="stat-value">${monster.cr}</div>`
  html += '</div>'
  html += '</div>'
  
  // Environment
  if (monster.environment && monster.environment.length > 0) {
    html += '<div class="monster-environment">'
    html += `<h4>Environment:</h4>`
    html += `<p>${monster.environment.join(', ')}</p>`
    html += '</div>'
  }
  
  // Source with page - styled to bottom right
  html += '<div style="text-align: right; margin-top: 1.5rem; padding-top: 1rem; border-top: 1px solid var(--color-border-light, #222);">'
  html += `<span class="source-info">Source: ${monster.source}</span>`
  html += '</div>'
  html += '</div>'
  
  return html
}

function formatSize(size: string | string[]): string {
  const sizeValue = Array.isArray(size) ? size[0] : size
  const sizeMap: Record<string, string> = {
    'T': 'Tiny',
    'S': 'Small',
    'M': 'Medium',
    'L': 'Large',
    'H': 'Huge',
    'G': 'Gargantuan'
  }
  return sizeMap[sizeValue] || sizeValue
}

function formatSpeed(speed: any): string {
  if (!speed) return '—'
  if (typeof speed === 'string') return speed
  if (typeof speed === 'object') {
    const speeds: string[] = []
    if (speed.walk) speeds.push(`${speed.walk} ft.`)
    if (speed.fly) speeds.push(`fly ${speed.fly} ft.`)
    if (speed.swim) speeds.push(`swim ${speed.swim} ft.`)
    if (speed.climb) speeds.push(`climb ${speed.climb} ft.`)
    if (speed.burrow) speeds.push(`burrow ${speed.burrow} ft.`)
    return speeds.join(', ') || '—'
  }
  return '—'
}

function formatAbilityScore(score?: number): string {
  if (!score) return '10 (+0)'
  const modifier = Math.floor((score - 10) / 2)
  const sign = modifier >= 0 ? '+' : ''
  return `${score} (${sign}${modifier})`
}

function formatSaves(saves: any): string {
  if (!saves) return '—'
  if (typeof saves === 'string') return saves
  if (typeof saves === 'object') {
    const saveList: string[] = []
    for (const [key, value] of Object.entries(saves)) {
      saveList.push(`${key.toUpperCase()} ${value}`)
    }
    return saveList.join(', ')
  }
  return '—'
}

function formatSkills(skills: any): string {
  if (!skills) return '—'
  if (typeof skills === 'string') return skills
  if (typeof skills === 'object') {
    const skillList: string[] = []
    for (const [key, value] of Object.entries(skills)) {
      const skillName = key.charAt(0).toUpperCase() + key.slice(1)
      skillList.push(`${skillName} ${value}`)
    }
    return skillList.join(', ')
  }
  return '—'
}

function formatAction(action: any): string {
  if (!action) return ''
  
  let html = '<div class="action-block" style="margin-bottom: 0.75rem;">'
  if (action.name) {
    html += `<p style="margin: 0;"><strong>${action.name}.</strong> `
  }
  
  if (action.entries) {
    if (Array.isArray(action.entries)) {
      const entriesHtml = formatEntries(action.entries)
      // Remove wrapping <p> tags from entries since we're already in a <p>
      const cleanedEntries = entriesHtml.replace(/<\/?p[^>]*>/g, ' ').trim()
      html += cleanedEntries
    } else {
      html += action.entries
    }
  }
  
  html += '</p></div>'
  return html
}

function formatCreatureType(type: any): string {
  if (!type) return 'Unknown'
  if (typeof type === 'string') return type
  if (typeof type === 'object' && type.type) {
    // Handle complex type object like {type: "humanoid", tags: ["any race"]}
    let result = type.type
    if (type.tags && Array.isArray(type.tags) && type.tags.length > 0) {
      result += ` (${type.tags.join(', ')})`
    }
    return result
  }
  return 'Unknown'
}

function formatAlignment(alignment: any): string {
  if (!alignment) return 'Unaligned'
  if (typeof alignment === 'string') return alignment
  if (Array.isArray(alignment)) {
    const alignments = alignment.map(a => {
      if (typeof a === 'string') {
        // Handle special alignment codes
        const alignmentMap: Record<string, string> = {
          'A': 'Any alignment',
          'G': 'Good',
          'E': 'Evil',
          'L': 'Lawful',
          'C': 'Chaotic',
          'N': 'Neutral',
          'U': 'Unaligned',
          'L G': 'Lawful Good',
          'N G': 'Neutral Good',
          'C G': 'Chaotic Good',
          'L N': 'Lawful Neutral',
          'N N': 'True Neutral',
          'C N': 'Chaotic Neutral',
          'L E': 'Lawful Evil',
          'N E': 'Neutral Evil',
          'C E': 'Chaotic Evil'
        }
        return alignmentMap[a] || a
      }
      if (a.alignment) {
        // Handle complex alignment objects
        const alignmentStr = Array.isArray(a.alignment) ? a.alignment.join(' ') : a.alignment
        return formatAlignment(alignmentStr)
      }
      return ''
    }).filter(a => a).join(' or ')
    return alignments || 'Unaligned'
  }
  return 'Unaligned'
}

function formatAC(ac: any): string {
  if (!ac) return '—'
  if (typeof ac === 'number') return ac.toString()
  if (Array.isArray(ac) && ac.length > 0) {
    const first = ac[0]
    if (typeof first === 'number') return first.toString()
    if (first.ac) return first.ac.toString()
  }
  if (ac.ac) return ac.ac.toString()
  return '—'
}

function formatHP(hp: any): string {
  if (!hp) return '—'
  if (typeof hp === 'number') return hp.toString()
  if (hp.average) {
    const formula = hp.formula ? ` (${hp.formula})` : ''
    return `${hp.average}${formula}`
  }
  return '—'
}