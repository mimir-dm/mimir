import type { Feat, FeatSummary } from '../composables/catalog'
import { processFormattingTags, formatEntries } from '../utils/textFormatting'

export async function formatFeatDetails(feat: Feat | FeatSummary): Promise<string> {
  const isFull = 'entries' in feat
  
  let html = '<div class="feat-details">'
  
  // Header
  html += '<div class="feat-header">'
  html += `<h2>${feat.name}</h2>`
  
  // Prerequisites
  if ('prerequisite' in feat && feat.prerequisite && feat.prerequisite.length > 0) {
    html += '<div class="feat-prerequisites">'
    html += '<strong>Prerequisites:</strong> '
    
    const prereqStrings: string[] = []
    for (const prereq of feat.prerequisite) {
      if (typeof prereq === 'string') {
        prereqStrings.push(prereq)
      } else if (prereq && typeof prereq === 'object') {
        // Handle complex prerequisite objects
        const parts: string[] = []
        
        if ('level' in prereq && prereq.level) {
          if (prereq.level.class) {
            parts.push(`${prereq.level.class.name} level ${prereq.level.level}`)
          } else if (prereq.level.level) {
            parts.push(`level ${prereq.level.level}`)
          }
        }
        
        if ('ability' in prereq && prereq.ability) {
          for (const [ability, score] of Object.entries(prereq.ability)) {
            if (ability && score) {
              parts.push(`${ability.toUpperCase()} ${score} or higher`)
            }
          }
        }
        
        if ('spell' in prereq && prereq.spell) {
          parts.push(`ability to cast ${prereq.spell} spell`)
        }
        
        if ('feature' in prereq && prereq.feature) {
          parts.push(prereq.feature)
        }
        
        if ('race' in prereq && prereq.race) {
          const races = Array.isArray(prereq.race) ? prereq.race : [prereq.race]
          const raceNames = races.map((r: any) => typeof r === 'string' ? r : r.name).join(' or ')
          parts.push(raceNames)
        }
        
        if (parts.length > 0) {
          prereqStrings.push(parts.join(', '))
        }
      }
    }
    
    html += prereqStrings.join('; ')
    html += '</div>'
  } else if ('prerequisites' in feat && feat.prerequisites) {
    // Handle summary format
    html += '<div class="feat-prerequisites">'
    html += `<strong>Prerequisites:</strong> ${feat.prerequisites}`
    html += '</div>'
  }
  
  html += '</div>'
  
  // Content
  html += '<div class="feat-content">'
  
  if (isFull && feat.entries) {
    // Full feat with entries
    html += formatEntries(feat.entries)
    
    // Ability Score Increase
    if ('ability' in feat && feat.ability && feat.ability.length > 0) {
      html += '<h4>Ability Score Increase</h4>'
      html += '<ul>'
      for (const ability of feat.ability) {
        if (ability.choose) {
          const count = ability.choose.count || 1
          const amount = ability.choose.amount || 1
          const from = ability.choose.from || []
          html += `<li>Increase ${count} ability score${count > 1 ? 's' : ''} of your choice`
          if (from.length > 0) {
            html += ` from ${from.join(', ')}`
          }
          html += ` by ${amount}</li>`
        } else {
          for (const [stat, value] of Object.entries(ability)) {
            if (stat !== 'choose' && value) {
              html += `<li>${stat.toUpperCase()} +${value}</li>`
            }
          }
        }
      }
      html += '</ul>'
    }
    
    // Proficiencies
    if ('skill_proficiencies' in feat && feat.skill_proficiencies) {
      html += '<h4>Skill Proficiencies</h4>'
      html += formatProficiencies(feat.skill_proficiencies)
    }
    
    if ('language_proficiencies' in feat && feat.language_proficiencies) {
      html += '<h4>Language Proficiencies</h4>'
      html += formatProficiencies(feat.language_proficiencies)
    }
    
    if ('tool_proficiencies' in feat && feat.tool_proficiencies) {
      html += '<h4>Tool Proficiencies</h4>'
      html += formatProficiencies(feat.tool_proficiencies)
    }
    
    if ('weapon_proficiencies' in feat && feat.weapon_proficiencies) {
      html += '<h4>Weapon Proficiencies</h4>'
      html += formatProficiencies(feat.weapon_proficiencies)
    }
    
    if ('armor_proficiencies' in feat && feat.armor_proficiencies) {
      html += '<h4>Armor Proficiencies</h4>'
      html += formatProficiencies(feat.armor_proficiencies)
    }
    
    // Additional Spells
    if ('additional_spells' in feat && feat.additional_spells) {
      html += '<h4>Additional Spells</h4>'
      html += formatAdditionalSpells(feat.additional_spells)
    }
    
  } else if ('brief' in feat && feat.brief) {
    // Summary format
    html += `<p>${feat.brief}</p>`
  }
  
  html += '</div>'
  
  // Footer with source
  html += '<div class="feat-footer">'
  html += `<span class="source-info">Source: ${feat.source}`
  if (feat.page) {
    html += `, p. ${feat.page}`
  }
  html += '</span>'
  html += '</div>'
  
  html += '</div>'
  
  return html
}


function formatProficiencies(proficiencies: any[]): string {
  let html = '<ul>'
  
  for (const prof of proficiencies) {
    if (typeof prof === 'string') {
      html += `<li>${prof}</li>`
    } else if (prof.any) {
      html += `<li>Any ${prof.any} of your choice</li>`
    } else if (prof.choose) {
      const choices = prof.choose.from || []
      const count = prof.choose.count || 1
      html += `<li>Choose ${count} from: ${choices.join(', ')}</li>`
    } else {
      // Handle specific proficiency objects
      for (const [key, value] of Object.entries(prof)) {
        if (value && key !== 'choose' && key !== 'any') {
          html += `<li>${key}: ${value}</li>`
        }
      }
    }
  }
  
  html += '</ul>'
  return html
}

function formatAdditionalSpells(spells: any[]): string {
  let html = '<div class="additional-spells">'
  
  for (const spellGroup of spells) {
    if (spellGroup.expanded) {
      html += '<div class="spell-list">'
      if (spellGroup.expanded.s1) {
        html += formatSpellList('1st Level', spellGroup.expanded.s1)
      }
      if (spellGroup.expanded.s2) {
        html += formatSpellList('2nd Level', spellGroup.expanded.s2)
      }
      if (spellGroup.expanded.s3) {
        html += formatSpellList('3rd Level', spellGroup.expanded.s3)
      }
      if (spellGroup.expanded.s4) {
        html += formatSpellList('4th Level', spellGroup.expanded.s4)
      }
      if (spellGroup.expanded.s5) {
        html += formatSpellList('5th Level', spellGroup.expanded.s5)
      }
      html += '</div>'
    }
    
    if (spellGroup.innate) {
      html += '<div class="innate-spells">'
      html += '<strong>Innate Spellcasting:</strong>'
      if (spellGroup.innate._) {
        html += formatSpellList('At Will', spellGroup.innate._)
      }
      if (spellGroup.innate.daily) {
        for (const [uses, spells] of Object.entries(spellGroup.innate.daily)) {
          html += formatSpellList(`${uses}/day each`, spells as string[])
        }
      }
      html += '</div>'
    }
  }
  
  html += '</div>'
  return html
}

function formatSpellList(label: string, spells: string[]): string {
  let html = `<div class="spell-level-group">`
  html += `<strong>${label}:</strong> `
  html += spells.map(spell => `<span class="spell-reference">${spell}</span>`).join(', ')
  html += '</div>'
  return html
}