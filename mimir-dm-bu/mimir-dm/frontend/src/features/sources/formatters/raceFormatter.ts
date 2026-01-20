import type { Race, RaceSummary, RaceWithDetails, Subrace } from '../composables/catalog'
import { processFormattingTags, formatEntries } from '../utils/textFormatting'

export async function formatRaceDetails(raceData: RaceWithDetails | RaceSummary): Promise<string> {
  // Handle both summary and full details
  const isFullDetails = 'race' in raceData || 'subrace' in raceData
  
  if (!isFullDetails) {
    return formatRaceSummary(raceData as RaceSummary)
  }
  
  return await formatFullRaceDetails(raceData as RaceWithDetails)
}

function formatRaceSummary(raceSummary: RaceSummary): string {
  let html = '<div class="race-details">'
  
  // Header
  html += '<div class="race-header">'
  html += `<h2>${raceSummary.name}</h2>`
  if (raceSummary.isSubrace && raceSummary.parentRace) {
    html += `<div class="race-type">Subrace of ${raceSummary.parentRace}</div>`
  }
  html += '</div>'
  
  // Properties
  html += '<div class="race-properties">'
  html += `<div class="property-item">
    <span class="property-label">Size:</span>
    <span class="property-value">${raceSummary.size}</span>
  </div>`
  html += `<div class="property-item">
    <span class="property-label">Speed:</span>
    <span class="property-value">${raceSummary.speed} ft.</span>
  </div>`
  html += `<div class="property-item">
    <span class="property-label">Ability Score Increase:</span>
    <span class="property-value">${raceSummary.abilityBonuses}</span>
  </div>`
  html += `<div class="property-item">
    <span class="property-label">Racial Traits:</span>
    <span class="property-value">${raceSummary.traitsCount} traits</span>
  </div>`
  html += '</div>'
  
  // Source
  html += `<div class="source-info">Source: ${raceSummary.source}</div>`
  
  html += '</div>'
  return html
}

async function formatFullRaceDetails(details: RaceWithDetails): Promise<string> {
  // For subraces, we want to show BOTH base race and subrace info
  const primaryRace = details.subrace ? details.race : details.race
  const subrace = details.subrace
  
  if (!primaryRace && !subrace) {
    return '<div class="race-details">No race data available</div>'
  }
  
  let html = '<div class="race-details">'
  
  // Header
  html += '<div class="race-header">'
  if (subrace) {
    html += `<h2>${subrace.name || subrace.raceName} ${subrace.raceName}</h2>`
    html += `<div class="race-type">Subrace of ${subrace.raceName}</div>`
  } else if (primaryRace) {
    html += `<h2>${primaryRace.name}</h2>`
  }
  html += '</div>'
  
  // If this is a subrace, show base race traits first
  if (subrace && primaryRace) {
    // Base Race Section
    html += '<div class="base-race-section">'
    html += `<h3>Base Race Traits (${primaryRace.name})</h3>`
    html += '<div class="race-properties">'
    
    // Base race properties
    if (primaryRace.size) {
      const sizeText = Array.isArray(primaryRace.size) 
        ? primaryRace.size.map((s: string) => getSizeName(s)).join(', ')
        : 'Medium'
      html += `<div class="property-item">
        <span class="property-label">Size:</span>
        <span class="property-value">${sizeText}</span>
      </div>`
    }
    
    if (primaryRace.speed) {
      html += formatSpeed(primaryRace.speed)
    }
    
    if (primaryRace.ability) {
      html += formatAbilityScores(primaryRace.ability)
    }
    
    if (primaryRace.age) {
      html += formatAge(primaryRace.age)
    }
    
    if (primaryRace.darkvision) {
      html += `<div class="property-item">
        <span class="property-label">Darkvision:</span>
        <span class="property-value">${primaryRace.darkvision} ft.</span>
      </div>`
    }
    
    if (primaryRace.languageProficiencies) {
      html += formatLanguages(primaryRace.languageProficiencies)
    }
    
    if (primaryRace.resist) {
      html += formatResistances(primaryRace.resist)
    }
    
    html += '</div>'
    
    // Base race traits
    if (primaryRace.entries && primaryRace.entries.length > 0) {
      html += '<div class="race-traits">'
      html += formatEntries(primaryRace.entries)
      html += '</div>'
    }
    
    html += '</div>'
    
    // Subrace Section
    html += '<div class="subrace-section">'
    html += `<h3>Subrace Traits (${subrace.name || 'Variant'})</h3>`
    html += '<div class="race-properties">'
    
    // Show subrace-specific modifications
    if (subrace.ability) {
      html += '<div class="property-item subrace-mod">'
      html += '<span class="property-label">Additional Ability Scores:</span>'
      html += '<span class="property-value">'
      html += formatAbilityScoresSimple(subrace.ability)
      html += '</span></div>'
    }
    
    if (subrace.speed && subrace.speed !== primaryRace.speed) {
      html += '<div class="property-item subrace-mod">'
      html += formatSpeed(subrace.speed)
      html += '</div>'
    }
    
    if (subrace.darkvision && subrace.darkvision !== primaryRace.darkvision) {
      html += `<div class="property-item subrace-mod">
        <span class="property-label">Darkvision:</span>
        <span class="property-value">${subrace.darkvision} ft.</span>
      </div>`
    }
    
    if (subrace.weaponProficiencies) {
      html += formatWeaponProficiencies(subrace.weaponProficiencies)
    }
    
    if (subrace.armorProficiencies) {
      html += formatArmorProficiencies(subrace.armorProficiencies)
    }
    
    if (subrace.toolProficiencies) {
      html += formatToolProficiencies(subrace.toolProficiencies)
    }
    
    html += '</div>'
    
    // Subrace traits
    if (subrace.entries && subrace.entries.length > 0) {
      html += '<div class="race-traits">'
      html += formatEntries(subrace.entries)
      html += '</div>'
    }
    
    html += '</div>'
  } else {
    // Single race (not a subrace)
    const race = primaryRace || subrace
    if (!race) {
      return '<div class="race-details">No race data available</div>'
    }
    
    html += '<div class="race-properties">'
    
    // Size
    if ('size' in race && race.size) {
      const sizeText = Array.isArray(race.size) 
        ? race.size.map(s => getSizeName(s)).join(', ')
        : 'Medium'
      html += `<div class="property-item">
        <span class="property-label">Size:</span>
        <span class="property-value">${sizeText}</span>
      </div>`
    }
    
    // Speed
    if ('speed' in race && race.speed) {
      html += formatSpeed(race.speed)
    }
    
    // Ability Scores
    if ('ability' in race && race.ability) {
      html += formatAbilityScores(race.ability)
    }
    
    // Age
    if ('age' in race && race.age) {
      html += formatAge(race.age)
    }
    
    // Darkvision
    if ('darkvision' in race && race.darkvision) {
      html += `<div class="property-item">
        <span class="property-label">Darkvision:</span>
        <span class="property-value">${race.darkvision} ft.</span>
      </div>`
    }
    
    // Languages
    if ('languageProficiencies' in race && race.languageProficiencies) {
      html += formatLanguages(race.languageProficiencies)
    }
    
    // Resistances
    if ('resist' in race && race.resist) {
      html += formatResistances(race.resist)
    }
    
    html += '</div>'
    
    // Racial Traits
    if ('entries' in race && race.entries && race.entries.length > 0) {
      html += '<div class="race-traits">'
      html += '<h3>Racial Traits</h3>'
      html += formatEntries(race.entries)
      html += '</div>'
    }
  }
  
  // Related Subraces
  if (details.relatedSubraces && details.relatedSubraces.length > 0) {
    html += '<div class="related-subraces">'
    html += '<h3>Subraces</h3>'
    html += '<ul>'
    for (const subrace of details.relatedSubraces) {
      html += `<li>${subrace.name || 'Variant'} (${subrace.source})</li>`
    }
    html += '</ul>'
    html += '</div>'
  }
  
  // Fluff/Lore
  if (details.fluff && details.fluff.entries) {
    html += '<div class="race-lore">'
    html += '<h3>Description</h3>'
    html += formatEntries(details.fluff.entries)
    html += '</div>'
  }
  
  // Source
  const sourceRace = primaryRace || subrace
  if (sourceRace) {
    html += `<div class="source-info">Source: ${sourceRace.source}`
    if ('page' in sourceRace && sourceRace.page) {
      html += `, p. ${sourceRace.page}`
    }
    html += '</div>'
  }
  
  html += '</div>'
  return html
}

function getSizeName(size: string): string {
  const sizeMap: Record<string, string> = {
    'T': 'Tiny',
    'S': 'Small',
    'M': 'Medium',
    'L': 'Large',
    'H': 'Huge',
    'G': 'Gargantuan'
  }
  return sizeMap[size] || size
}

function formatSpeed(speed: any): string {
  let html = '<div class="property-item"><span class="property-label">Speed:</span><span class="property-value">'
  
  if (typeof speed === 'number') {
    html += `${speed} ft.`
  } else if (typeof speed === 'object' && speed) {
    const speeds: string[] = []
    if (speed.walk) speeds.push(`${speed.walk} ft.`)
    if (speed.fly) speeds.push(`fly ${speed.fly} ft.`)
    if (speed.swim) speeds.push(`swim ${speed.swim} ft.`)
    if (speed.climb) speeds.push(`climb ${speed.climb} ft.`)
    if (speed.burrow) speeds.push(`burrow ${speed.burrow} ft.`)
    html += speeds.join(', ')
  }
  
  html += '</span></div>'
  return html
}

function formatAbilityScores(abilities: any[]): string {
  let html = '<div class="property-item"><span class="property-label">Ability Score Increase:</span><span class="property-value">'
  
  const parts: string[] = []
  for (const ability of abilities) {
    if (typeof ability === 'object' && ability) {
      const abilityParts: string[] = []
      
      if (ability.str) abilityParts.push(`STR +${ability.str}`)
      if (ability.dex) abilityParts.push(`DEX +${ability.dex}`)
      if (ability.con) abilityParts.push(`CON +${ability.con}`)
      if (ability.int) abilityParts.push(`INT +${ability.int}`)
      if (ability.wis) abilityParts.push(`WIS +${ability.wis}`)
      if (ability.cha) abilityParts.push(`CHA +${ability.cha}`)
      
      if (ability.choose) {
        const count = ability.choose.count || 1
        const amount = ability.choose.amount || 1
        const from = ability.choose.from || []
        
        let choiceText = `Choose ${count}`
        if (from.length > 0) {
          choiceText += ` from ${from.join(', ')}`
        }
        choiceText += ` +${amount}`
        abilityParts.push(choiceText)
      }
      
      if (abilityParts.length > 0) {
        parts.push(abilityParts.join(', '))
      }
    }
  }
  
  html += parts.join('; ') || 'None'
  html += '</span></div>'
  return html
}

function formatAge(age: any): string {
  if (!age) return ''
  
  let html = '<div class="property-item"><span class="property-label">Age:</span><span class="property-value">'
  const parts: string[] = []
  
  if (age.mature) parts.push(`Mature at ${age.mature}`)
  if (age.max) parts.push(`Live to ${age.max}`)
  
  html += parts.join(', ') || 'Varies'
  html += '</span></div>'
  return html
}

function formatLanguages(languages: any[]): string {
  let html = '<div class="property-item"><span class="property-label">Languages:</span><span class="property-value">'
  
  const langList: string[] = []
  for (const lang of languages) {
    if (typeof lang === 'object' && lang) {
      // Extract language names from the object keys
      for (const key of Object.keys(lang)) {
        if (lang[key] === true) {
          // Capitalize first letter
          langList.push(key.charAt(0).toUpperCase() + key.slice(1))
        }
      }
    }
  }
  
  html += langList.join(', ') || 'Common'
  html += '</span></div>'
  return html
}

function formatResistances(resistances: any[]): string {
  let html = '<div class="property-item"><span class="property-label">Damage Resistance:</span><span class="property-value">'
  
  const resistList: string[] = []
  for (const resist of resistances) {
    if (typeof resist === 'string') {
      resistList.push(resist)
    } else if (resist && resist.choose) {
      const from = resist.choose.from || []
      resistList.push(`Choose from: ${from.join(', ')}`)
    }
  }
  
  html += resistList.join(', ') || 'None'
  html += '</span></div>'
  return html
}

function formatAbilityScoresSimple(abilities: any[]): string {
  const parts: string[] = []
  for (const ability of abilities) {
    if (typeof ability === 'object' && ability) {
      const abilityParts: string[] = []
      
      if (ability.str) abilityParts.push(`STR +${ability.str}`)
      if (ability.dex) abilityParts.push(`DEX +${ability.dex}`)
      if (ability.con) abilityParts.push(`CON +${ability.con}`)
      if (ability.int) abilityParts.push(`INT +${ability.int}`)
      if (ability.wis) abilityParts.push(`WIS +${ability.wis}`)
      if (ability.cha) abilityParts.push(`CHA +${ability.cha}`)
      
      if (abilityParts.length > 0) {
        parts.push(abilityParts.join(', '))
      }
    }
  }
  return parts.join('; ') || 'None'
}

function formatWeaponProficiencies(proficiencies: any[]): string {
  let html = '<div class="property-item subrace-mod">'
  html += '<span class="property-label">Weapon Proficiencies:</span>'
  html += '<span class="property-value">'
  
  const weapons: string[] = []
  for (const prof of proficiencies) {
    if (typeof prof === 'object' && prof) {
      for (const key of Object.keys(prof)) {
        if (prof[key] === true) {
          // Remove source suffix like |phb
          const weaponName = key.split('|')[0]
          weapons.push(weaponName)
        }
      }
    }
  }
  
  html += weapons.join(', ') || 'None'
  html += '</span></div>'
  return html
}

function formatArmorProficiencies(proficiencies: any[]): string {
  let html = '<div class="property-item subrace-mod">'
  html += '<span class="property-label">Armor Proficiencies:</span>'
  html += '<span class="property-value">'
  
  const armors: string[] = []
  for (const prof of proficiencies) {
    if (typeof prof === 'object' && prof) {
      for (const key of Object.keys(prof)) {
        if (prof[key] === true) {
          const armorName = key.split('|')[0]
          armors.push(armorName)
        }
      }
    }
  }
  
  html += armors.join(', ') || 'None'
  html += '</span></div>'
  return html
}

function formatToolProficiencies(proficiencies: any[]): string {
  let html = '<div class="property-item subrace-mod">'
  html += '<span class="property-label">Tool Proficiencies:</span>'
  html += '<span class="property-value">'
  
  const tools: string[] = []
  for (const prof of proficiencies) {
    if (typeof prof === 'object' && prof) {
      // Handle choice
      if (prof.choose) {
        const count = prof.choose.count || 1
        const from = prof.choose.from || []
        tools.push(`Choose ${count} from: ${from.join(', ')}`)
      } else {
        // Handle specific tools
        for (const key of Object.keys(prof)) {
          if (prof[key] === true && key !== 'choose') {
            const toolName = key.split('|')[0]
            tools.push(toolName)
          }
        }
      }
    }
  }
  
  html += tools.join(', ') || 'None'
  html += '</span></div>'
  return html
}