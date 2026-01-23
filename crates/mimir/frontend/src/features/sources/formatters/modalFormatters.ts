// Modal content formatters

import { processFormattingTags } from '../utils/textFormatting'

/**
 * Render modal content for a reference.
 * Accepts data object with ref_type field (from useCrossReferences)
 */
export function renderModalContent(data: any): string {
  if (!data) return ''

  let html = '<div class="modal-content">'

  // Title
  if (data.name) {
    html += `<h3>${data.name}</h3>`
  }

  // Type-specific rendering based on ref_type
  const refType = data.ref_type || data.type
  switch (refType) {
    case 'spell':
      html += renderSpellContent(data)
      break
    case 'item':
      html += renderItemContent(data)
      break
    case 'creature':
    case 'monster':
      html += renderMonsterContent(data)
      break
    case 'condition':
      html += renderConditionContent(data)
      break
    case 'action':
      html += renderActionContent(data)
      break
    case 'feat':
      html += renderFeatContent(data)
      break
    case 'background':
      html += renderBackgroundContent(data)
      break
    case 'race':
      html += renderRaceContent(data)
      break
    case 'class':
      html += renderClassContent(data)
      break
    default:
      html += renderGenericContent(data)
  }

  html += '</div>'
  return html
}

function renderSpellContent(data: any): string {
  let html = '<div class="spell-content">'

  if (data.level !== undefined) {
    const levelStr = data.level === 0 ? 'Cantrip' : `${data.level}${getOrdinalSuffix(data.level)}-level`
    const schoolStr = formatSpellSchool(data.school)
    html += `<p><strong>${levelStr} ${schoolStr}</strong></p>`
  }

  if (data.time) {
    html += `<p><strong>Casting Time:</strong> ${formatSpellTime(data.time)}</p>`
  }

  if (data.range) {
    html += `<p><strong>Range:</strong> ${formatSpellRange(data.range)}</p>`
  }

  if (data.components) {
    html += `<p><strong>Components:</strong> ${formatSpellComponents(data.components)}</p>`
  }

  if (data.duration) {
    html += `<p><strong>Duration:</strong> ${formatSpellDuration(data.duration)}</p>`
  }

  if (data.entries && Array.isArray(data.entries)) {
    html += '<div class="spell-description">'
    for (const entry of data.entries) {
      if (typeof entry === 'string') {
        html += `<p>${processFormattingTags(entry)}</p>`
      } else if (entry.type === 'entries' && entry.entries) {
        if (entry.name) html += `<p><strong>${entry.name}.</strong></p>`
        for (const subEntry of entry.entries) {
          if (typeof subEntry === 'string') {
            html += `<p>${processFormattingTags(subEntry)}</p>`
          }
        }
      } else if (entry.type === 'list' && entry.items) {
        html += '<ul>'
        for (const item of entry.items) {
          html += `<li>${processFormattingTags(typeof item === 'string' ? item : item.entry || '')}</li>`
        }
        html += '</ul>'
      }
    }
    html += '</div>'
  }

  // Higher level scaling
  if (data.entriesHigherLevel && Array.isArray(data.entriesHigherLevel)) {
    html += '<div class="spell-higher-levels">'
    for (const entry of data.entriesHigherLevel) {
      if (entry.entries) {
        html += `<p><strong>At Higher Levels.</strong> ${processFormattingTags(entry.entries.join(' '))}</p>`
      }
    }
    html += '</div>'
  }

  html += '</div>'
  return html
}

function renderItemContent(data: any): string {
  let html = '<div class="item-content">'
  
  if (data.type) {
    html += `<p><strong>Type:</strong> ${data.type}</p>`
  }
  
  if (data.rarity && data.rarity !== 'none') {
    html += `<p><strong>Rarity:</strong> ${data.rarity}</p>`
  }
  
  if (data.value) {
    html += `<p><strong>Value:</strong> ${formatCurrency(data.value)}</p>`
  }
  
  if (data.weight) {
    html += `<p><strong>Weight:</strong> ${data.weight} lb.</p>`
  }
  
  if (data.entries && Array.isArray(data.entries)) {
    html += '<div class="item-description">'
    for (const entry of data.entries) {
      if (typeof entry === 'string') {
        html += `<p>${processFormattingTags(entry)}</p>`
      }
    }
    html += '</div>'
  }
  
  html += '</div>'
  return html
}

function renderMonsterContent(data: any): string {
  let html = '<div class="monster-content">'
  
  if (data.size && data.type) {
    html += `<p><em>${data.size} ${data.type}, ${data.alignment || 'unaligned'}</em></p>`
  }
  
  if (data.ac) {
    html += `<p><strong>Armor Class:</strong> ${data.ac}</p>`
  }
  
  if (data.hp) {
    html += `<p><strong>Hit Points:</strong> ${data.hp}</p>`
  }
  
  if (data.speed) {
    html += `<p><strong>Speed:</strong> ${data.speed}</p>`
  }
  
  // Ability scores
  if (data.str || data.dex || data.con || data.int || data.wis || data.cha) {
    html += '<table class="ability-scores">'
    html += '<tr>'
    html += `<th>STR</th><th>DEX</th><th>CON</th><th>INT</th><th>WIS</th><th>CHA</th>`
    html += '</tr>'
    html += '<tr>'
    html += `<td>${data.str || 10}</td>`
    html += `<td>${data.dex || 10}</td>`
    html += `<td>${data.con || 10}</td>`
    html += `<td>${data.int || 10}</td>`
    html += `<td>${data.wis || 10}</td>`
    html += `<td>${data.cha || 10}</td>`
    html += '</tr>'
    html += '</table>'
  }
  
  if (data.cr) {
    html += `<p><strong>Challenge:</strong> ${data.cr}</p>`
  }
  
  html += '</div>'
  return html
}

function renderConditionContent(data: any): string {
  let html = '<div class="condition-content">'

  console.log('renderConditionContent data:', data)

  if (data.entries && Array.isArray(data.entries)) {
    for (const entry of data.entries) {
      if (typeof entry === 'string') {
        html += `<p>${processFormattingTags(entry)}</p>`
      } else if (typeof entry === 'object' && entry !== null) {
        // Handle list type entries
        if (entry.type === 'list' && entry.items && Array.isArray(entry.items)) {
          html += '<ul>'
          for (const item of entry.items) {
            if (typeof item === 'string') {
              html += `<li>${processFormattingTags(item)}</li>`
            } else if (typeof item === 'object' && item.entry) {
              html += `<li>${processFormattingTags(item.entry)}</li>`
            }
          }
          html += '</ul>'
        }
        // Handle entries type (nested entries with name)
        else if (entry.type === 'entries' && entry.name) {
          html += `<h4>${entry.name}</h4>`
          if (entry.entries && Array.isArray(entry.entries)) {
            for (const subEntry of entry.entries) {
              if (typeof subEntry === 'string') {
                html += `<p>${processFormattingTags(subEntry)}</p>`
              }
            }
          }
        }
        // Fallback: try to render any text content
        else if (entry.entries) {
          html += renderConditionContent({ entries: entry.entries })
        }
      }
    }
  }

  // If no entries rendered, show a message
  if (html === '<div class="condition-content">') {
    html += '<p><em>No description available.</em></p>'
  }

  html += '</div>'
  return html
}

function renderActionContent(data: any): string {
  let html = '<div class="action-content">'

  console.log('renderActionContent data:', data)

  // Format time if present (array of time objects)
  if (data.time && Array.isArray(data.time)) {
    const timeStr = data.time.map((t: any) => {
      if (t.number && t.unit) {
        return `${t.number} ${t.unit}`
      }
      return JSON.stringify(t)
    }).join(', ')
    html += `<p><strong>Time:</strong> ${timeStr}</p>`
  } else if (data.time) {
    html += `<p><strong>Time:</strong> ${data.time}</p>`
  }

  if (data.entries && Array.isArray(data.entries)) {
    for (const entry of data.entries) {
      if (typeof entry === 'string') {
        html += `<p>${processFormattingTags(entry)}</p>`
      } else if (typeof entry === 'object' && entry !== null) {
        if (entry.type === 'list' && entry.items) {
          html += '<ul>'
          for (const item of entry.items) {
            if (typeof item === 'string') {
              html += `<li>${processFormattingTags(item)}</li>`
            }
          }
          html += '</ul>'
        } else if (entry.type === 'entries' && entry.name) {
          html += `<h4>${entry.name}</h4>`
          if (entry.entries) {
            for (const subEntry of entry.entries) {
              if (typeof subEntry === 'string') {
                html += `<p>${processFormattingTags(subEntry)}</p>`
              }
            }
          }
        }
      }
    }
  }

  html += '</div>'
  return html
}

function renderFeatContent(data: any): string {
  let html = '<div class="feat-content">'

  // Prerequisites
  if (data.prerequisite && Array.isArray(data.prerequisite)) {
    const prereqs = data.prerequisite.map((p: any) => {
      const parts: string[] = []
      if (p.level) parts.push(`Level ${p.level}`)
      if (p.race) parts.push(Array.isArray(p.race) ? p.race.map((r: any) => r.name).join(' or ') : p.race)
      if (p.ability) {
        const abilities = p.ability.map((a: any) => {
          const [stat, val] = Object.entries(a)[0]
          return `${stat.toUpperCase()} ${val}`
        })
        parts.push(abilities.join(' or '))
      }
      if (p.spellcasting) parts.push('Spellcasting ability')
      return parts.join(', ')
    }).filter((s: string) => s.length > 0)

    if (prereqs.length > 0) {
      html += `<p><strong>Prerequisite:</strong> ${prereqs.join('; ')}</p>`
    }
  }

  if (data.entries && Array.isArray(data.entries)) {
    html += '<div class="feat-description">'
    for (const entry of data.entries) {
      if (typeof entry === 'string') {
        html += `<p>${processFormattingTags(entry)}</p>`
      } else if (entry.type === 'list' && entry.items) {
        html += '<ul>'
        for (const item of entry.items) {
          if (typeof item === 'string') {
            html += `<li>${processFormattingTags(item)}</li>`
          }
        }
        html += '</ul>'
      } else if (entry.type === 'entries' && entry.entries) {
        if (entry.name) {
          html += `<p><strong>${entry.name}.</strong></p>`
        }
        for (const subEntry of entry.entries) {
          if (typeof subEntry === 'string') {
            html += `<p>${processFormattingTags(subEntry)}</p>`
          }
        }
      }
    }
    html += '</div>'
  }

  html += '</div>'
  return html
}

function renderBackgroundContent(data: any): string {
  let html = '<div class="background-content">'

  // Skill proficiencies
  if (data.skillProficiencies && Array.isArray(data.skillProficiencies)) {
    const skills = data.skillProficiencies.flatMap((sp: any) => Object.keys(sp))
    if (skills.length > 0) {
      html += `<p><strong>Skill Proficiencies:</strong> ${skills.join(', ')}</p>`
    }
  }

  // Tool proficiencies
  if (data.toolProficiencies && Array.isArray(data.toolProficiencies)) {
    const tools = data.toolProficiencies.flatMap((tp: any) => {
      return Object.entries(tp).map(([tool, val]) => tool)
    })
    if (tools.length > 0) {
      html += `<p><strong>Tool Proficiencies:</strong> ${tools.join(', ')}</p>`
    }
  }

  // Languages
  if (data.languageProficiencies && Array.isArray(data.languageProficiencies)) {
    const langs = data.languageProficiencies.flatMap((lp: any) => {
      if (lp.anyStandard) return [`${lp.anyStandard} of your choice`]
      return Object.keys(lp)
    })
    if (langs.length > 0) {
      html += `<p><strong>Languages:</strong> ${langs.join(', ')}</p>`
    }
  }

  // Starting equipment
  if (data.startingEquipment && Array.isArray(data.startingEquipment)) {
    html += '<p><strong>Equipment:</strong></p><ul>'
    for (const equip of data.startingEquipment) {
      if (typeof equip === 'string') {
        html += `<li>${equip}</li>`
      } else if (equip._) {
        html += `<li>${equip._}</li>`
      }
    }
    html += '</ul>'
  }

  // Entries/Features
  if (data.entries && Array.isArray(data.entries)) {
    html += '<div class="background-features">'
    for (const entry of data.entries) {
      if (typeof entry === 'string') {
        html += `<p>${processFormattingTags(entry)}</p>`
      } else if (entry.type === 'entries' && entry.name) {
        html += `<h4>${entry.name}</h4>`
        if (entry.entries) {
          for (const subEntry of entry.entries) {
            if (typeof subEntry === 'string') {
              html += `<p>${processFormattingTags(subEntry)}</p>`
            }
          }
        }
      }
    }
    html += '</div>'
  }

  html += '</div>'
  return html
}

function renderRaceContent(data: any): string {
  let html = '<div class="race-content">'

  // Size
  if (data.size && Array.isArray(data.size)) {
    const sizeMap: Record<string, string> = { S: 'Small', M: 'Medium', L: 'Large' }
    const sizes = data.size.map((s: string) => sizeMap[s] || s)
    html += `<p><strong>Size:</strong> ${sizes.join(' or ')}</p>`
  }

  // Speed
  if (data.speed) {
    if (typeof data.speed === 'number') {
      html += `<p><strong>Speed:</strong> ${data.speed} ft.</p>`
    } else if (typeof data.speed === 'object') {
      const speeds: string[] = []
      if (data.speed.walk) speeds.push(`${data.speed.walk} ft.`)
      if (data.speed.fly) speeds.push(`fly ${data.speed.fly} ft.`)
      if (data.speed.swim) speeds.push(`swim ${data.speed.swim} ft.`)
      if (data.speed.climb) speeds.push(`climb ${data.speed.climb} ft.`)
      html += `<p><strong>Speed:</strong> ${speeds.join(', ')}</p>`
    }
  }

  // Darkvision
  if (data.darkvision) {
    html += `<p><strong>Darkvision:</strong> ${data.darkvision} ft.</p>`
  }

  // Entries/Traits
  if (data.entries && Array.isArray(data.entries)) {
    html += '<div class="race-traits">'
    for (const entry of data.entries) {
      if (typeof entry === 'string') {
        html += `<p>${processFormattingTags(entry)}</p>`
      } else if (entry.type === 'entries' && entry.name) {
        html += `<h4>${entry.name}</h4>`
        if (entry.entries) {
          for (const subEntry of entry.entries) {
            if (typeof subEntry === 'string') {
              html += `<p>${processFormattingTags(subEntry)}</p>`
            }
          }
        }
      }
    }
    html += '</div>'
  }

  html += '</div>'
  return html
}

function renderClassContent(data: any): string {
  let html = '<div class="class-content">'

  // Hit die
  if (data.hd && data.hd.faces) {
    html += `<p><strong>Hit Die:</strong> d${data.hd.faces}</p>`
  }

  // Primary ability
  if (data.primaryAbility) {
    const abilities = Array.isArray(data.primaryAbility)
      ? data.primaryAbility.map((a: any) => Object.keys(a)[0]).join(' or ')
      : Object.keys(data.primaryAbility)[0]
    if (abilities) {
      html += `<p><strong>Primary Ability:</strong> ${abilities.toUpperCase()}</p>`
    }
  }

  // Saving throws
  if (data.proficiency && Array.isArray(data.proficiency)) {
    const saves = data.proficiency.filter((p: string) =>
      ['str', 'dex', 'con', 'int', 'wis', 'cha'].includes(p.toLowerCase())
    ).map((s: string) => s.toUpperCase())
    if (saves.length > 0) {
      html += `<p><strong>Saving Throws:</strong> ${saves.join(', ')}</p>`
    }
  }

  // Armor/weapon proficiencies
  if (data.startingProficiencies) {
    const sp = data.startingProficiencies
    if (sp.armor && sp.armor.length > 0) {
      html += `<p><strong>Armor:</strong> ${sp.armor.join(', ')}</p>`
    }
    if (sp.weapons && sp.weapons.length > 0) {
      html += `<p><strong>Weapons:</strong> ${sp.weapons.join(', ')}</p>`
    }
  }

  // Class features (limited for modal)
  if (data.classFeatures && Array.isArray(data.classFeatures)) {
    html += '<p><strong>Features at 1st Level:</strong></p><ul>'
    const level1Features = data.classFeatures.filter((cf: any) => {
      if (typeof cf === 'string') {
        return cf.includes('|1|') || cf.startsWith('1|')
      }
      return cf.level === 1
    }).slice(0, 5)
    for (const feature of level1Features) {
      const name = typeof feature === 'string' ? feature.split('|')[0] : feature.name
      html += `<li>${name}</li>`
    }
    html += '</ul>'
  }

  html += '</div>'
  return html
}

function renderGenericContent(data: any): string {
  let html = '<div class="generic-content">'
  
  if (data.entries && Array.isArray(data.entries)) {
    for (const entry of data.entries) {
      if (typeof entry === 'string') {
        html += `<p>${processFormattingTags(entry)}</p>`
      }
    }
  } else if (data.text) {
    html += `<p>${processFormattingTags(data.text)}</p>`
  } else if (data.description) {
    html += `<p>${processFormattingTags(data.description)}</p>`
  }
  
  html += '</div>'
  return html
}

// =============================================================================
// Spell formatting helpers
// =============================================================================

const SPELL_SCHOOL_MAP: Record<string, string> = {
  A: 'Abjuration',
  C: 'Conjuration',
  D: 'Divination',
  E: 'Enchantment',
  V: 'Evocation',
  I: 'Illusion',
  N: 'Necromancy',
  T: 'Transmutation',
}

function formatSpellSchool(school: any): string {
  if (!school) return ''
  if (typeof school === 'string') {
    return SPELL_SCHOOL_MAP[school.toUpperCase()] || school
  }
  return ''
}

function formatSpellTime(time: any): string {
  if (!time) return ''
  if (typeof time === 'string') return time

  if (Array.isArray(time)) {
    return time.map((t: any) => {
      if (typeof t === 'string') return t
      const num = t.number || 1
      const unit = t.unit || 'action'
      const condition = t.condition ? `, ${t.condition}` : ''
      return `${num} ${unit}${num > 1 ? 's' : ''}${condition}`
    }).join(' or ')
  }

  if (typeof time === 'object') {
    const num = time.number || 1
    const unit = time.unit || 'action'
    return `${num} ${unit}${num > 1 ? 's' : ''}`
  }

  return String(time)
}

function formatSpellRange(range: any): string {
  if (!range) return ''
  if (typeof range === 'string') return range

  if (typeof range === 'object') {
    const type = range.type
    if (type === 'point') {
      const dist = range.distance
      if (!dist) return 'Point'
      if (dist.type === 'self') return 'Self'
      if (dist.type === 'touch') return 'Touch'
      if (dist.type === 'sight') return 'Sight'
      if (dist.type === 'unlimited') return 'Unlimited'
      if (dist.amount) return `${dist.amount} ${dist.type || 'feet'}`
      return dist.type || 'Point'
    }
    if (type === 'special') return 'Special'
    if (type === 'radius' || type === 'sphere' || type === 'cone' || type === 'line' || type === 'cube' || type === 'hemisphere') {
      const dist = range.distance
      if (dist && dist.amount) {
        return `Self (${dist.amount}-${dist.type || 'foot'} ${type})`
      }
      return `Self (${type})`
    }
    // Fallback for other object structures
    if (range.distance) {
      const dist = range.distance
      if (dist.amount) return `${dist.amount} ${dist.type || 'feet'}`
      return dist.type || ''
    }
  }

  return String(range)
}

function formatSpellComponents(components: any): string {
  if (!components) return ''
  if (typeof components === 'string') return components

  if (typeof components === 'object') {
    const parts: string[] = []
    if (components.v) parts.push('V')
    if (components.s) parts.push('S')
    if (components.m) {
      if (typeof components.m === 'string') {
        parts.push(`M (${components.m})`)
      } else if (typeof components.m === 'object') {
        const text = components.m.text || components.m.consume || ''
        parts.push(`M (${text})`)
      } else {
        parts.push('M')
      }
    }
    return parts.join(', ')
  }

  return String(components)
}

function formatSpellDuration(duration: any): string {
  if (!duration) return ''
  if (typeof duration === 'string') return duration

  if (Array.isArray(duration)) {
    return duration.map((d: any) => formatSingleDuration(d)).join(' or ')
  }

  return formatSingleDuration(duration)
}

function formatSingleDuration(d: any): string {
  if (typeof d === 'string') return d
  if (!d || typeof d !== 'object') return ''

  const type = d.type
  if (type === 'instant') return 'Instantaneous'
  if (type === 'permanent') {
    if (d.ends && Array.isArray(d.ends)) {
      const endings = d.ends.map((e: string) => {
        if (e === 'dispel') return 'dispelled'
        if (e === 'trigger') return 'triggered'
        return e
      })
      return `Until ${endings.join(' or ')}`
    }
    return 'Permanent'
  }
  if (type === 'special') return 'Special'
  if (type === 'timed') {
    const amount = d.duration?.amount || d.amount || ''
    const unit = d.duration?.type || d.unit || ''
    const conc = d.concentration ? 'Concentration, up to ' : ''
    return `${conc}${amount} ${unit}${amount > 1 ? 's' : ''}`
  }

  // Fallback
  if (d.duration) {
    const amount = d.duration.amount || ''
    const unit = d.duration.type || ''
    return `${amount} ${unit}${amount > 1 ? 's' : ''}`
  }

  return ''
}

// =============================================================================
// Currency formatting
// =============================================================================

/**
 * Format a value in copper pieces to the most appropriate denomination.
 * 5etools stores all values in copper pieces (cp).
 * 1 pp = 1000 cp, 1 gp = 100 cp, 1 sp = 10 cp
 */
function formatCurrency(copperValue: number): string {
  if (copperValue >= 100) {
    const gp = copperValue / 100
    // Show as whole number if no remainder, otherwise 1 decimal
    return gp % 1 === 0 ? `${gp} gp` : `${gp.toFixed(1)} gp`
  } else if (copperValue >= 10) {
    const sp = copperValue / 10
    return sp % 1 === 0 ? `${sp} sp` : `${sp.toFixed(1)} sp`
  } else {
    return `${copperValue} cp`
  }
}

function getOrdinalSuffix(num: number): string {
  const j = num % 10
  const k = num % 100
  
  if (j === 1 && k !== 11) {
    return 'st'
  }
  if (j === 2 && k !== 12) {
    return 'nd'
  }
  if (j === 3 && k !== 13) {
    return 'rd'
  }
  return 'th'
}