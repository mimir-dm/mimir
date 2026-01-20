import { processFormattingTags } from '../utils/textFormatting'
import { formatSpellLevel } from '../../../shared/utils/formatters'

interface SpellDetails {
  name: string
  level: number
  school: string
  source: string
  page?: number
  time: Array<{
    number: number
    unit: string
    condition?: string
  }>
  range: {
    type: string
    distance?: {
      type?: string
      amount?: number
    }
  }
  components: {
    v?: boolean
    s?: boolean
    m?: string | { text: string }
  }
  duration: Array<{
    type: string
    duration?: {
      type: string
      amount?: number
    }
    concentration?: boolean
  }>
  entries: string[]
  entriesHigherLevel?: Array<{
    type: string
    name: string
    entries: string[]
  }>
  classes?: {
    fromClassList?: Array<{
      name: string
      source: string
    }>
  }
  meta?: {
    ritual: boolean
  }
  // New fields for enhanced display
  miscTags?: string[]
  scalingLevelDice?: {
    label: string
    scaling: Record<string, string>
  }
  savingThrow?: string[]
  spellAttack?: string[]
  damageInflict?: string[]
  conditionInflict?: string[]
  areaTags?: string[]
}

export function formatSpellDetails(spell: any): string {
  // Handle both summary and full details
  const isFullDetails = spell.time !== undefined
  
  if (!isFullDetails) {
    return formatSpellSummary(spell)
  }
  
  return formatFullSpellDetails(spell as SpellDetails)
}

function formatSpellSummary(spell: any): string {
  let html = '<div class="spell-details">'
  
  // Header with level and school
  html += '<div class="spell-header-section">'
  html += `<div class="spell-level-school">${formatSpellLevel(spell.level)} ${spell.school}</div>`
  if (spell.ritual) {
    html += '<div class="spell-tag ritual">Ritual</div>'
  }
  html += '</div>'
  
  // Properties section
  html += '<div class="spell-properties-grid">'
  html += `<div class="property-item">
    <span class="property-label">Casting Time:</span>
    <span class="property-value">${spell.casting_time}</span>
  </div>`
  html += `<div class="property-item">
    <span class="property-label">Range:</span>
    <span class="property-value">${spell.range}</span>
  </div>`
  html += `<div class="property-item">
    <span class="property-label">Components:</span>
    <span class="property-value">${spell.components}</span>
  </div>`
  html += `<div class="property-item">
    <span class="property-label">Duration:</span>
    <span class="property-value">${spell.concentration ? 'Concentration, up to ' : ''}${spell.description || 'Instantaneous'}</span>
  </div>`
  
  if (spell.classes && spell.classes.length > 0) {
    html += `<div class="property-item full-width">
      <span class="property-label">Classes:</span>
      <span class="property-value">${spell.classes.join(', ')}</span>
    </div>`
  }
  html += '</div>'
  
  // Description
  if (spell.description) {
    html += '<div class="spell-description-section">'
    html += '<h4>Description</h4>'
    html += `<div class="description-text">${processFormattingTags(spell.description)}</div>`
    html += '</div>'
  }
  
  // Footer
  html += `<div class="spell-footer">
    <span class="source-info">Source: ${spell.source}</span>
  </div>`
  html += '</div>'
  
  return html
}

function formatFullSpellDetails(spell: SpellDetails): string {
  let html = '<div class="spell-details enhanced">'
  
  // Header section
  html += '<div class="spell-header-section">'
  const levelSchool = `${formatSpellLevel(spell.level)} ${spell.school}`
  html += `<div class="spell-level-school">${levelSchool}</div>`
  
  const tags = []
  if (spell.meta?.ritual) tags.push('<span class="spell-tag ritual">Ritual</span>')
  if (spell.duration?.some(d => d.concentration)) tags.push('<span class="spell-tag concentration">Concentration</span>')
  
  // Add miscellaneous tags
  if (spell.miscTags) {
    for (const tag of spell.miscTags) {
      const tagInfo = formatMiscTag(tag)
      if (tagInfo) {
        tags.push(`<span class="spell-tag ${tagInfo.class}">${tagInfo.label}</span>`)
      }
    }
  }
  
  // Add cantrip scaling tag
  if (spell.level === 0 && spell.scalingLevelDice) {
    tags.push('<span class="spell-tag scaling">Scaling</span>')
  }
  
  if (tags.length > 0) {
    html += `<div class="spell-tags">${tags.join(' ')}</div>`
  }
  html += '</div>'
  
  // Properties grid
  html += '<div class="spell-properties-grid">'
  
  // Casting Time
  const castingTime = formatCastingTime(spell.time)
  html += `<div class="property-item">
    <span class="property-label">Casting Time</span>
    <span class="property-value">${castingTime}</span>
  </div>`
  
  // Range
  const range = formatRange(spell.range)
  html += `<div class="property-item">
    <span class="property-label">Range</span>
    <span class="property-value">${range}</span>
  </div>`
  
  // Components
  const components = formatComponents(spell.components)
  html += `<div class="property-item ${spell.components.m ? 'full-width' : ''}">
    <span class="property-label">Components</span>
    <span class="property-value">${components}</span>
  </div>`
  
  // Duration
  const duration = formatDuration(spell.duration)
  html += `<div class="property-item">
    <span class="property-label">Duration</span>
    <span class="property-value">${duration}</span>
  </div>`
  
  // Classes
  if (spell.classes?.fromClassList) {
    const classList = spell.classes.fromClassList.map(c => c.name).join(', ')
    html += `<div class="property-item full-width">
      <span class="property-label">Classes</span>
      <span class="property-value">${classList}</span>
    </div>`
  }
  
  html += '</div>'
  
  // Cantrip Scaling Section (for level 0 spells)
  if (spell.level === 0 && spell.scalingLevelDice) {
    html += '<div class="spell-scaling-section">'
    html += '<h4>Cantrip Scaling</h4>'
    html += '<div class="scaling-display">'
    html += formatCantripScaling(spell.scalingLevelDice)
    html += '</div>'
    html += '</div>'
  }
  
  // Combat Mechanics Section
  const hasCombatMechanics = spell.savingThrow || spell.spellAttack || spell.damageInflict || 
                           spell.conditionInflict || spell.areaTags
  
  if (hasCombatMechanics) {
    html += '<div class="spell-combat-section">'
    html += '<h4>Combat Mechanics</h4>'
    html += '<div class="combat-mechanics-grid">'
    
    // Saving Throws
    if (spell.savingThrow && spell.savingThrow.length > 0) {
      const saves = spell.savingThrow.map((s: string) => formatSaveType(s)).join(', ')
      html += `<div class="combat-item">
        <span class="combat-label">Save</span>
        <span class="combat-value">${saves}</span>
      </div>`
    }
    
    // Spell Attack
    if (spell.spellAttack && spell.spellAttack.length > 0) {
      const attackType = formatAttackType(spell.spellAttack[0])
      html += `<div class="combat-item">
        <span class="combat-label">Attack</span>
        <span class="combat-value">${attackType}</span>
      </div>`
    }
    
    // Damage Types
    if (spell.damageInflict && spell.damageInflict.length > 0) {
      const damageTypes = spell.damageInflict.map((d: string) => 
        `<span class="damage-type ${d}">${formatDamageType(d)}</span>`
      ).join(' ')
      html += `<div class="combat-item">
        <span class="combat-label">Damage</span>
        <span class="combat-value">${damageTypes}</span>
      </div>`
    }
    
    // Conditions
    if (spell.conditionInflict && spell.conditionInflict.length > 0) {
      const conditions = spell.conditionInflict.map((c: string) => 
        `<span class="condition-badge ${c}">${formatCondition(c)}</span>`
      ).join(' ')
      html += `<div class="combat-item">
        <span class="combat-label">Conditions</span>
        <span class="combat-value">${conditions}</span>
      </div>`
    }
    
    // Area Effect
    if (spell.areaTags && spell.areaTags.length > 0) {
      const areaType = formatAreaType(spell.areaTags[0])
      html += `<div class="combat-item">
        <span class="combat-label">Area</span>
        <span class="combat-value">${areaType}</span>
      </div>`
    }
    
    html += '</div>'
    html += '</div>'
  }
  
  // Main description
  if (spell.entries && spell.entries.length > 0) {
    html += '<div class="spell-description-section">'
    html += '<h4>Description</h4>'
    html += '<div class="description-text">'
    for (const entry of spell.entries) {
      if (typeof entry === 'string') {
        html += `<p>${processFormattingTags(entry)}</p>`
      } else if (typeof entry === 'object' && entry !== null) {
        const entryObj = entry as any
        if (entryObj.type === 'list') {
          html += formatList(entryObj)
        } else if (entryObj.type === 'entries' && entryObj.entries) {
          for (const subEntry of entryObj.entries) {
            if (typeof subEntry === 'string') {
              html += `<p>${processFormattingTags(subEntry)}</p>`
            }
          }
        }
      }
    }
    html += '</div>'
    html += '</div>'
  }
  
  // At Higher Levels (enhanced)
  if (spell.entriesHigherLevel && spell.entriesHigherLevel.length > 0) {
    html += '<div class="spell-higher-level-section">'
    html += '<h4><span class="scaling-icon">📈</span> Higher Level Casting</h4>'
    html += '<div class="description-text">'
    for (const section of spell.entriesHigherLevel) {
      for (const entry of section.entries) {
        html += `<p class="higher-level-entry">${processFormattingTags(entry)}</p>`
      }
    }
    html += '</div>'
    html += '</div>'
  }
  
  // Footer
  html += '<div class="spell-footer">'
  html += `<span class="source-info">Source: ${spell.source}`
  if (spell.page) html += `, p. ${spell.page}`
  html += '</span>'
  html += '</div>'
  
  html += '</div>'
  
  return html
}

function formatCastingTime(time: any[]): string {
  if (!time || time.length === 0) return 'Unknown'
  
  const t = time[0]
  let result = `${t.number} ${t.unit}`
  if (t.condition) result += ` ${t.condition}`
  return result
}

function formatRange(range: any): string {
  if (!range) return 'Unknown'
  
  if (range.type === 'point') {
    if (range.distance?.type === 'self') return 'Self'
    if (range.distance?.type === 'touch') return 'Touch'
    if (range.distance?.type === 'sight') return 'Sight'
    if (range.distance?.type === 'unlimited') return 'Unlimited'
    if (range.distance?.amount) {
      return `${range.distance.amount} ${range.distance.type || 'feet'}`
    }
  } else if (range.type === 'radius') {
    return `Self (${range.distance?.amount || '?'}-foot radius)`
  } else if (range.type === 'sphere') {
    return `Self (${range.distance?.amount || '?'}-foot sphere)`
  } else if (range.type === 'cone') {
    return `Self (${range.distance?.amount || '?'}-foot cone)`
  } else if (range.type === 'line') {
    return `Self (${range.distance?.amount || '?'}-foot line)`
  }
  
  return range.type || 'Special'
}

function formatComponents(components: any): string {
  if (!components) return 'None'
  
  const parts = []
  if (components.v) parts.push('V')
  if (components.s) parts.push('S')
  if (components.m) {
    const material = typeof components.m === 'string' ? components.m : components.m.text
    parts.push(`M (${material})`)
  }
  
  return parts.join(', ') || 'None'
}

function formatDuration(duration: any[]): string {
  if (!duration || duration.length === 0) return 'Instantaneous'
  
  const d = duration[0]
  if (d.type === 'instant') return 'Instantaneous'
  if (d.type === 'permanent') return 'Until dispelled'
  if (d.type === 'special') return 'Special'
  
  if (d.duration) {
    let result = ''
    if (d.concentration) result += 'Concentration, up to '
    result += `${d.duration.amount} ${d.duration.type}`
    return result
  }
  
  return d.type || 'Unknown'
}

function formatList(listObj: any): string {
  let html = '<ul class="spell-list">'
  if (listObj.items) {
    for (const item of listObj.items) {
      html += `<li>${processFormattingTags(item)}</li>`
    }
  }
  html += '</ul>'
  return html
}

function formatSaveType(save: string): string {
  const saveMap: Record<string, string> = {
    'strength': 'Strength',
    'dexterity': 'Dexterity', 
    'constitution': 'Constitution',
    'intelligence': 'Intelligence',
    'wisdom': 'Wisdom',
    'charisma': 'Charisma'
  }
  return saveMap[save] || save.charAt(0).toUpperCase() + save.slice(1)
}

function formatAttackType(attack: string): string {
  const attackMap: Record<string, string> = {
    'M': 'Melee spell attack',
    'R': 'Ranged spell attack',
    'O': 'Special attack'
  }
  return attackMap[attack] || 'Spell attack'
}

function formatDamageType(damage: string): string {
  const damageMap: Record<string, string> = {
    'acid': 'Acid',
    'bludgeoning': 'Bludgeoning',
    'cold': 'Cold', 
    'fire': 'Fire',
    'force': 'Force',
    'lightning': 'Lightning',
    'necrotic': 'Necrotic',
    'piercing': 'Piercing',
    'poison': 'Poison',
    'psychic': 'Psychic',
    'radiant': 'Radiant',
    'slashing': 'Slashing',
    'thunder': 'Thunder'
  }
  return damageMap[damage] || damage.charAt(0).toUpperCase() + damage.slice(1)
}

function formatCondition(condition: string): string {
  const conditionMap: Record<string, string> = {
    'blinded': 'Blinded',
    'charmed': 'Charmed',
    'deafened': 'Deafened',
    'frightened': 'Frightened',
    'grappled': 'Grappled',
    'incapacitated': 'Incapacitated',
    'invisible': 'Invisible',
    'paralyzed': 'Paralyzed',
    'petrified': 'Petrified',
    'poisoned': 'Poisoned',
    'prone': 'Prone',
    'restrained': 'Restrained',
    'stunned': 'Stunned',
    'unconscious': 'Unconscious'
  }
  return conditionMap[condition] || condition.charAt(0).toUpperCase() + condition.slice(1)
}

function formatAreaType(area: string): string {
  const areaMap: Record<string, string> = {
    'ST': 'Single target',
    'MT': 'Multiple targets',
    'S': 'Sphere',
    'C': 'Cone',
    'L': 'Line', 
    'Y': 'Cylinder',
    'H': 'Hemisphere',
    'Q': 'Square',
    'R': 'Rectangle',
    'N': 'Square/Rectangle',
    'W': 'Wall'
  }
  return areaMap[area] || area
}

function formatMiscTag(tag: string): { label: string; class: string } | null {
  const tagMap: Record<string, { label: string; class: string }> = {
    'SCL': { label: 'Scaling', class: 'scaling' },
    'HEL': { label: 'Healing', class: 'healing' },
    'SMN': { label: 'Summoning', class: 'summoning' },
    'LGT': { label: 'Light', class: 'light' },
    'THP': { label: 'Temp HP', class: 'temp-hp' },
    'UBA': { label: 'Bonus Action', class: 'bonus-action' },
    'PRM': { label: 'Permanent', class: 'permanent' },
    'OBJ': { label: 'Affects Objects', class: 'objects' },
    'FMV': { label: 'Forced Movement', class: 'movement' },
    'MAC': { label: 'Multiple Attacks', class: 'attacks' }
  }
  return tagMap[tag] || null
}

function formatCantripScaling(scalingData: any): string {
  if (!scalingData || !scalingData.scaling) return ''
  
  const scaling = scalingData.scaling
  const label = scalingData.label || 'damage'
  
  // Build progression display: "1d10 → 2d10 (5th) → 3d10 (11th) → 4d10 (17th)"
  const levels = ['1', '5', '11', '17']
  const progression = levels
    .filter(level => scaling[level])
    .map((level, index) => {
      const dice = scaling[level]
      if (index === 0) {
        return `<span class="scaling-dice">${dice}</span>`
      } else {
        return `<span class="scaling-dice">${dice}</span> <span class="scaling-level">(${level}th)</span>`
      }
    })
    .join(' → ')
  
  return `<div class="cantrip-scaling">
    <span class="scaling-label">${label.charAt(0).toUpperCase() + label.slice(1)}:</span>
    <span class="scaling-progression">${progression}</span>
  </div>`
}