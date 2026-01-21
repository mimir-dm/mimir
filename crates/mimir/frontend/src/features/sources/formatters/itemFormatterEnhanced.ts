import { processFormattingTags, formatEntries } from '../utils/textFormatting'
import { invoke } from '@tauri-apps/api/core'

interface ItemDetails {
  name: string
  type: string
  typeName?: string
  rarity?: string
  value?: number
  weight?: number
  weightNote?: string
  ac?: number
  strength?: string
  stealth?: boolean
  dmg1?: string
  dmg2?: string
  dmgType?: string
  property?: string[]
  range?: string
  reqAttune?: boolean | string
  reqAttuneTags?: any[]
  source: string
  page?: number
  entries?: (string | object)[]
  additionalEntries?: (string | object)[]
  modifySpeed?: object
  resist?: string[]
  immune?: string[]
  conditionImmune?: string[]
  bonusSpellAttack?: string
  bonusSpellSaveDc?: string
  bonusWeapon?: string
  bonusWeaponAttack?: string
  bonusWeaponDamage?: string
  bonusAc?: string
  grantsProficiency?: boolean
  // Equipment-specific fields
  weapon?: boolean
  weaponCategory?: string
  armor?: boolean
  ammunition?: boolean
  ammoType?: string
  scfType?: string
  group?: string[]
  light?: any[]
  containerCapacity?: any
  carryingCapacity?: number
  speed?: number
  packContents?: any[]
  miscTags?: string[]
  tier?: string
  lootTables?: string[]
  // Fluff content
  hasFluffImages?: boolean
  fluffImages?: any[]
  fluffEntries?: any[]
  poison?: boolean
}

export async function formatItemDetails(item: any): Promise<string> {
  // Handle both summary and full details
  const isFullDetails = item.entries !== undefined || item.additionalEntries !== undefined || 
                        item.bonusWeapon !== undefined || item.fluffImages !== undefined
  
  if (!isFullDetails) {
    return formatItemSummary(item)
  }
  
  return await formatFullItemDetails(item as ItemDetails)
}

function formatItemSummary(item: any): string {
  let html = '<div class="item-details">'
  
  // Header with type and rarity
  html += '<div class="item-header-section">'
  html += `<div class="item-type-rarity">${item.typeName || item.type || 'Item'}`
  if (item.rarity && item.rarity !== 'none') {
    html += ` (${formatRarity(item.rarity)})`
  }
  html += '</div>'
  
  if (item.reqAttune) {
    html += '<div class="item-tag attunement">Requires Attunement</div>'
  }
  html += '</div>'
  
  // Properties section
  html += '<div class="item-properties-grid">'
  
  if (item.value) {
    html += `<div class="property-item">
      <span class="property-label">Cost:</span>
      <span class="property-value">${formatCost(item.value)}</span>
    </div>`
  }
  
  if (item.weight) {
    html += `<div class="property-item">
      <span class="property-label">Weight:</span>
      <span class="property-value">${item.weight} lb</span>
    </div>`
  }
  
  if (item.ac !== undefined) {
    html += `<div class="property-item">
      <span class="property-label">AC:</span>
      <span class="property-value">${item.ac}</span>
    </div>`
  }
  
  if (item.dmg1) {
    html += `<div class="property-item">
      <span class="property-label">Damage:</span>
      <span class="property-value">${item.dmg1} ${item.dmgType || ''}</span>
    </div>`
  }
  
  html += '</div>'
  
  // Basic description if available
  if (item.description) {
    html += '<div class="item-description-section">'
    html += '<h4>Description</h4>'
    html += `<div class="description-text">${processFormattingTags(item.description)}</div>`
    html += '</div>'
  }
  
  // Footer
  html += `<div class="item-footer">
    <span class="source-info">Source: ${item.source}</span>
  </div>`
  html += '</div>'
  
  return html
}

async function formatFullItemDetails(item: ItemDetails): Promise<string> {
  let html = '<div class="item-details enhanced">'
  
  // Images (if available)
  if (item.fluffImages && item.fluffImages.length > 0) {
    html += '<div class="item-images">'
    for (const img of item.fluffImages) {
      if (img.href?.path) {
        const bookSource = item.source || 'DMG'
        try {
          const response = await invoke<any>('serve_book_image', {
            bookId: bookSource,
            imagePath: img.href.path
          })
          if (response && response.success && response.data) {
            html += `<img src="${response.data}" alt="${item.name}" class="item-image" style="max-width: 300px; max-height: 300px; width: auto; height: auto; object-fit: contain; display: block; margin: 0 auto 1rem;" />`
          }
        } catch (e) {
        }
      }
    }
    html += '</div>'
  }
  
  // Header section
  html += '<div class="item-header-section">'
  const typeInfo = formatItemTypeInfo(item)
  html += `<div class="item-type-rarity">${typeInfo}`
  if (item.rarity && item.rarity !== 'none') {
    html += ` (${formatRarity(item.rarity)})`
  }
  html += '</div>'
  
  const tags = []
  if (item.reqAttune) {
    const attunementText = typeof item.reqAttune === 'string' 
      ? `Requires Attunement (${item.reqAttune})`
      : 'Requires Attunement'
    tags.push(`<span class="item-tag attunement">${attunementText}</span>`)
  }
  if (item.tier) {
    tags.push(`<span class="item-tag tier">${formatTier(item.tier)}</span>`)
  }
  if (item.miscTags) {
    item.miscTags.forEach((tag: string) => {
      tags.push(`<span class="item-tag misc">${formatMiscTag(tag)}</span>`)
    })
  }
  if (tags.length > 0) {
    html += `<div class="item-tags">${tags.join(' ')}</div>`
  }
  html += '</div>'
  
  // Build properties grid content first
  let propertiesContent = ''
  
  // Basic properties
  if (item.value) {
    propertiesContent += `<div class="property-item">
      <span class="property-label">Cost</span>
      <span class="property-value">${formatCost(item.value)}</span>
    </div>`
  }
  
  if (item.weight) {
    const weightText = item.weightNote ? `${item.weight} lb ${item.weightNote}` : `${item.weight} lb`
    propertiesContent += `<div class="property-item">
      <span class="property-label">Weight</span>
      <span class="property-value">${weightText}</span>
    </div>`
  }
  
  // Only show armor properties for armor items
  if (item.armor || item.type === 'LA' || item.type === 'MA' || item.type === 'HA' || item.type === 'S') {
    if (item.ac !== undefined) {
      propertiesContent += `<div class="property-item">
        <span class="property-label">Armor Class</span>
        <span class="property-value">${item.ac}</span>
      </div>`
    }
    
    if (item.strength) {
      propertiesContent += `<div class="property-item">
        <span class="property-label">Strength Req.</span>
        <span class="property-value">${item.strength}</span>
      </div>`
    }
    
    if (item.stealth === true) {
      propertiesContent += `<div class="property-item">
        <span class="property-label">Stealth</span>
        <span class="property-value">Disadvantage</span>
      </div>`
    }
  }
  
  // Only show weapon properties for weapons
  if (item.weapon || item.type === 'M' || item.type === 'R' || item.dmg1) {
    if (item.dmg1) {
      const damageType = formatDamageType(item.dmgType)
      propertiesContent += `<div class="property-item">
        <span class="property-label">Damage</span>
        <span class="property-value">
          <span class="damage-dice">${item.dmg1}</span>
          ${damageType ? `<span class="damage-type">${damageType}</span>` : ''}
        </span>
      </div>`
    }
    
    if (item.dmg2) {
      const damageType = formatDamageType(item.dmgType)
      propertiesContent += `<div class="property-item">
        <span class="property-label">Versatile</span>
        <span class="property-value">
          <span class="damage-dice">${item.dmg2}</span>
          ${damageType ? `<span class="damage-type">${damageType}</span>` : ''}
        </span>
      </div>`
    }
    
    if (item.range) {
      propertiesContent += `<div class="property-item">
        <span class="property-label">Range</span>
        <span class="property-value">${item.range}</span>
      </div>`
    }
    
    if (item.property && item.property.length > 0) {
      propertiesContent += `<div class="property-item full-width">
        <span class="property-label">Properties</span>
        <span class="property-value">${formatWeaponProperties(item.property)}</span>
      </div>`
    }
  }
  
  // Show magic bonuses for magic items
  if (item.bonusWeapon || item.bonusWeaponAttack || item.bonusWeaponDamage) {
    if (item.bonusWeapon) {
      propertiesContent += `<div class="property-item">
        <span class="property-label">Weapon Bonus</span>
        <span class="property-value">+${item.bonusWeapon}</span>
      </div>`
    }
    
    if (item.bonusWeaponAttack) {
      propertiesContent += `<div class="property-item">
        <span class="property-label">Attack Bonus</span>
        <span class="property-value">+${item.bonusWeaponAttack}</span>
      </div>`
    }
    
    if (item.bonusWeaponDamage) {
      propertiesContent += `<div class="property-item">
        <span class="property-label">Damage Bonus</span>
        <span class="property-value">+${item.bonusWeaponDamage}</span>
      </div>`
    }
  }
  
  // Only add the properties grid if there's content
  if (propertiesContent) {
    html += '<div class="item-properties-grid">'
    html += propertiesContent
    html += '</div>'
  }
  
  // Don't show redundant combat section - properties are already displayed above
  
  // Magic item properties section
  if (item.bonusSpellAttack || item.bonusSpellSaveDc || item.bonusAc || item.lootTables) {
    html += formatMagicItemSection(item)
  }
  
  // Container/transport section
  if (item.containerCapacity || item.carryingCapacity || item.speed || item.packContents) {
    html += formatContainerSection(item)
  }
  
  // Light source section
  if (item.light) {
    html += formatLightSection(item)
  }
  
  // Main description
  if (item.entries && item.entries.length > 0) {
    html += '<div class="item-description-section">'
    html += '<h4>Description</h4>'
    html += '<div class="description-text">'
    html += formatEntries(item.entries)
    html += '</div>'
    html += '</div>'
  }
  
  // Additional entries
  if (item.additionalEntries && item.additionalEntries.length > 0) {
    html += '<div class="item-additional-section">'
    html += '<h4>Additional Properties</h4>'
    html += '<div class="description-text">'
    html += formatEntries(item.additionalEntries)
    html += '</div>'
    html += '</div>'
  }
  
  // Fluff entries (lore)
  if (item.fluffEntries && item.fluffEntries.length > 0) {
    html += '<div class="item-lore-section">'
    html += '<h4>Lore</h4>'
    html += '<div class="description-text">'
    html += formatEntries(item.fluffEntries)
    html += '</div>'
    html += '</div>'
  }
  
  // Footer
  html += '<div class="item-footer">'
  html += `<span class="source-info">Source: ${item.source}`
  if (item.page) html += `, p. ${item.page}`
  html += '</span>'
  html += '</div>'
  
  html += '</div>'
  
  return html
}
function formatCost(value: number): string {
  if (value >= 100) {
    return `${value / 100} gp`
  } else if (value >= 10) {
    return `${value / 10} sp`
  } else {
    return `${value} cp`
  }
}

function formatRarity(rarity: string): string {
  return rarity.charAt(0).toUpperCase() + rarity.slice(1)
}

function formatDamageType(damageType?: string): string {
  if (!damageType) return ''
  
  const damageTypeMap: Record<string, string> = {
    'A': 'acid',
    'B': 'bludgeoning', 
    'C': 'cold',
    'F': 'fire',
    'O': 'force',
    'L': 'lightning',
    'N': 'necrotic',
    'P': 'piercing',
    'I': 'poison',
    'Y': 'psychic',
    'R': 'radiant',
    'S': 'slashing',
    'T': 'thunder'
  }
  
  return damageTypeMap[damageType] || damageType.toLowerCase()
}

function formatWeaponProperties(properties: string[]): string {
  const propertyDescriptions: Record<string, string> = {
    'A': 'Ammunition',
    'F': 'Finesse',
    'H': 'Heavy',
    'L': 'Light',
    'LD': 'Loading',
    'R': 'Reach',
    'RLD': 'Reload',
    'S': 'Special',
    'T': 'Thrown',
    'TH': 'Two-Handed',
    'V': 'Versatile',
    '2H': 'Two-Handed',
    'AF': 'Ammunition, Finesse',
    'RN': 'Range',
    'BF': 'Burst Fire',
    'REL': 'Reload'
  }
  
  return properties.map(prop => {
    // Handle range properties like "RN|20/60"
    if (prop.startsWith('RN|')) {
      const range = prop.split('|')[1]
      return `<span class="weapon-property range">Range (${range} ft.)</span>`
    }
    
    // Handle thrown properties like "T|20/60"  
    if (prop.startsWith('T|')) {
      const range = prop.split('|')[1]
      return `<span class="weapon-property thrown">Thrown (range ${range} ft.)</span>`
    }
    
    // Handle versatile properties like "V|1d8"
    if (prop.startsWith('V|')) {
      const damage = prop.split('|')[1]
      return `<span class="weapon-property versatile">Versatile (${damage})</span>`
    }
    
    // Handle ammunition with damage like "AF|DMG"
    if (prop.startsWith('AF|')) {
      const details = prop.split('|')[1]
      if (details === 'DMG') {
        return `<span class="weapon-property ammunition">Ammunition (deals damage)</span>`
      }
      return `<span class="weapon-property ammunition">Ammunition, Finesse (${details})</span>`
    }
    
    // Handle burst fire with damage like "BF|DMG"
    if (prop.startsWith('BF|')) {
      const details = prop.split('|')[1]
      if (details === 'DMG') {
        return `<span class="weapon-property burstfire">Burst Fire (extra damage)</span>`
      }
      return `<span class="weapon-property burstfire">Burst Fire (${details})</span>`
    }
    
    // Handle reload properties like "RLD|6"
    if (prop.startsWith('RLD|')) {
      const shots = prop.split('|')[1]
      return `<span class="weapon-property reload">Reload (${shots} shots)</span>`
    }
    
    // Handle loading properties with details
    if (prop.startsWith('LD|')) {
      const details = prop.split('|')[1]
      return `<span class="weapon-property loading">Loading (${details})</span>`
    }
    
    // Standard properties
    const description = propertyDescriptions[prop] || prop
    const className = prop.toLowerCase().replace(/[^a-z]/g, '')
    return `<span class="weapon-property ${className}">${description}</span>`
  }).join(' ')
}

function formatItemTypeInfo(item: ItemDetails): string {
  if (item.weapon && item.weaponCategory) {
    return `${formatWeaponCategory(item.weaponCategory)} Weapon`
  }
  if (item.armor) {
    return 'Armor'
  }
  if (item.scfType) {
    return `${formatSpellcastingFocus(item.scfType)} Focus`
  }
  return item.typeName || formatItemType(item.type) || 'Item'
}

function formatWeaponCategory(category: string): string {
  return category.charAt(0).toUpperCase() + category.slice(1)
}

function formatSpellcastingFocus(focusType: string): string {
  const focusMap: Record<string, string> = {
    'holy': 'Holy Symbol',
    'druidic': 'Druidic Focus',
    'arcane': 'Arcane Focus'
  }
  return focusMap[focusType] || focusType
}

function formatItemType(type: string): string {
  // Handle complex type formats like "$G|DMG", "EXP|DMG", etc.
  // Extract the base type code before any | separator
  const baseType = type.includes('|') ? type.split('|')[0] : type
  
  // Handle treasure types with $ prefix - these are special and should NOT be stripped
  const typeMap: Record<string, string> = {
    // Treasure types (with $ prefix)
    '$': 'Treasure',                    // $ = TREASURE
    '$A': 'Art Object',                 // $A = TREASURE_ART_OBJECT
    '$C': 'Coinage',                    // $C = TREASURE_COINAGE  
    '$G': 'Gemstone',                   // $G = TREASURE_GEMSTONE
    // Regular item types (without $ prefix)
    'A': 'Ammunition',                  // A = AMMUNITION
    'AF': 'Futuristic Ammunition',      // AF = AMMUNITION_FUTURISTIC
    'AIR': 'Aircraft',                  // AIR = VEHICLE_AIR
    'AT': "Artisan's Tools",            // AT = ARTISAN_TOOL
    'EXP': 'Explosive',                 // EXP = EXPLOSIVE
    'FD': 'Food & Drink',               // FD = FOOD_AND_DRINK
    'G': 'Adventuring Gear',            // G = ADVENTURING_GEAR
    'GS': 'Gaming Set',                 // GS = GAMING_SET
    'GV': 'Generic Variant',            // GV = GENERIC_VARIANT
    'HA': 'Heavy Armor',                // HA = HEAVY_ARMOR
    'IDG': 'Illegal Drug',              // IDG = ILLEGAL_DRUG
    'INS': 'Musical Instrument',        // INS = INSTRUMENT
    'LA': 'Light Armor',                // LA = LIGHT_ARMOR
    'M': 'Melee Weapon',                // M = MELEE_WEAPON
    'MA': 'Medium Armor',               // MA = MEDIUM_ARMOR
    'MNT': 'Mount',                     // MNT = MOUNT
    'OTH': 'Other',                     // OTH = OTHER
    'P': 'Potion',                      // P = POTION
    'R': 'Ranged Weapon',               // R = RANGED_WEAPON
    'RD': 'Rod',                        // RD = ROD
    'RG': 'Ring',                       // RG = RING
    'S': 'Shield',                      // S = SHIELD
    'SC': 'Scroll',                     // SC = SCROLL
    'SCF': 'Spellcasting Focus',        // SCF = SPELLCASTING_FOCUS
    'SHP': 'Ship',                      // SHP = VEHICLE_WATER
    'SPC': 'Spacecraft',                // SPC = VEHICLE_SPACE
    'T': 'Tool',                        // T = TOOL
    'TAH': 'Tack & Harness',            // TAH = TACK_AND_HARNESS
    'TB': 'Trade Bar',                  // TB = TRADE_BAR
    'TG': 'Trade Good',                 // TG = TRADE_GOOD
    'VEH': 'Vehicle (Land)',            // VEH = VEHICLE_LAND
    'WD': 'Wand',                       // WD = WAND
    'W': 'Wondrous Item',               // W = Not in 5etools but commonly used
  }
  
  return typeMap[baseType] || type
}

function formatTier(tier: string): string {
  return tier.charAt(0).toUpperCase() + tier.slice(1)
}

function formatMiscTag(tag: string): string {
  const tagMap: Record<string, string> = {
    'CNS': 'Consumable'
  }
  return tagMap[tag] || tag
}
function formatMagicItemSection(item: ItemDetails): string {
  let html = '<div class="item-magic-section">'
  html += '<h4>Magical Properties</h4>'
  html += '<div class="magic-properties-grid">'
  
  if (item.bonusSpellAttack) {
    html += `<div class="magic-item">
      <span class="magic-label">Spell Attack</span>
      <span class="magic-value">${item.bonusSpellAttack} bonus</span>
    </div>`
  }
  
  if (item.bonusSpellSaveDc) {
    html += `<div class="magic-item">
      <span class="magic-label">Spell Save DC</span>
      <span class="magic-value">${item.bonusSpellSaveDc} bonus</span>
    </div>`
  }
  
  if (item.bonusAc) {
    html += `<div class="magic-item">
      <span class="magic-label">AC Bonus</span>
      <span class="magic-value">+${item.bonusAc}</span>
    </div>`
  }
  
  if (item.lootTables) {
    html += `<div class="magic-item full-width">
      <span class="magic-label">Loot Tables</span>
      <span class="magic-value">${item.lootTables.join(', ')}</span>
    </div>`
  }
  
  html += '</div>'
  html += '</div>'
  return html
}

function formatContainerSection(item: ItemDetails): string {
  let html = '<div class="item-container-section">'
  html += '<h4>Capacity & Transport</h4>'
  html += '<div class="container-properties-grid">'
  
  if (item.containerCapacity) {
    if (item.containerCapacity.weight) {
      html += `<div class="container-item">
        <span class="container-label">Capacity</span>
        <span class="container-value">${item.containerCapacity.weight[0]} lb</span>
      </div>`
    }
  }
  
  if (item.carryingCapacity) {
    html += `<div class="container-item">
      <span class="container-label">Carrying Capacity</span>
      <span class="container-value">${item.carryingCapacity} lb</span>
    </div>`
  }
  
  if (item.speed) {
    html += `<div class="container-item">
      <span class="container-label">Speed</span>
      <span class="container-value">${item.speed} ft.</span>
    </div>`
  }
  
  if (item.packContents && item.packContents.length > 0) {
    html += `<div class="container-item full-width">
      <span class="container-label">Contents</span>
      <div class="container-value">
        ${formatPackContents(item.packContents)}
      </div>
    </div>`
  }
  
  html += '</div>'
  html += '</div>'
  return html
}

function formatLightSection(item: ItemDetails): string {
  let html = '<div class="item-light-section">'
  html += '<h4>Light Properties</h4>'
  html += '<div class="light-properties-grid">'
  
  item.light?.forEach((lightSource: any) => {
    if (lightSource.bright) {
      html += `<div class="light-item">
        <span class="light-label">Bright Light</span>
        <span class="light-value">${lightSource.bright} ft.</span>
      </div>`
    }
    
    if (lightSource.dim) {
      html += `<div class="light-item">
        <span class="light-label">Dim Light</span>
        <span class="light-value">${lightSource.dim} ft.</span>
      </div>`
    }
    
    if (lightSource.shape) {
      html += `<div class="light-item">
        <span class="light-label">Shape</span>
        <span class="light-value">${lightSource.shape}</span>
      </div>`
    }
  })
  
  html += '</div>'
  html += '</div>'
  return html
}

function formatPackContents(contents: any[]): string {
  let html = '<ul class="pack-contents-list">'
  
  contents.forEach((item: any) => {
    if (typeof item === 'string') {
      html += `<li>${processFormattingTags(item)}</li>`
    } else if (item.item) {
      const quantity = item.quantity ? `${item.quantity}× ` : ''
      html += `<li>${quantity}${processFormattingTags(item.item)}</li>`
    } else if (item.special) {
      html += `<li>${processFormattingTags(item.special)}</li>`
    }
  })
  
  html += '</ul>'
  return html
}

function formatAmmoType(ammoType: string): string {
  const cleanType = ammoType.replace(/\|.*$/, '')
  return cleanType.split(' ').map(word => 
    word.charAt(0).toUpperCase() + word.slice(1)
  ).join(' ')
}