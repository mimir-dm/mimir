import { processFormattingTags, formatEntries } from '../utils/textFormatting'
import { invoke } from '@tauri-apps/api/core'
import type { Class, ClassWithDetails, ClassSummary, Subclass, ClassFeature, SubclassFeature, ClassFluff, SubclassFluff } from '../composables/catalog'

export async function formatClassDetails(
  classData: ClassWithDetails | ClassSummary | Subclass,
  selectedSubclassName?: string,
  subclassDetails?: Subclass | null
): Promise<string> {
  // Check what type of data we have
  // Handle wrapped ClassWithDetails structure (has 'class' property with actual class data)
  const isWrappedClassDetails = 'class' in classData && typeof (classData as any).class === 'object'
  const isSubclass = 'subclassFeatures' in classData || 'className' in classData

  // If we have a selected subclass, show class + subclass combined view
  if (selectedSubclassName && subclassDetails) {
    return await formatClassWithSelectedSubclass(classData, subclassDetails)
  }

  if (isSubclass && !isWrappedClassDetails) {
    return await formatSubclassDetails(classData as Subclass)
  } else if (isWrappedClassDetails) {
    return await formatFullClassDetails(classData as ClassWithDetails)
  } else {
    // Raw class data (from table row or direct API)
    return formatClassSummary(classData)
  }
}

function formatClassSummary(classData: ClassSummary | any): string {
  let html = '<div class="class-details">'

  // Header section
  html += '<div class="class-header-section">'
  html += `<h2>${classData.name || 'Unknown Class'}</h2>`
  html += '</div>'

  // Basic properties - handle both transformed (hitDice) and raw (hd) formats
  html += '<div class="class-properties-grid">'

  // Hit Dice - handle both formats
  const hitDice = classData.hitDice || (classData.hd?.faces ? `d${classData.hd.faces}` : null)
  if (hitDice) {
    html += `<div class="property-item">
      <span class="property-label">Hit Dice:</span>
      <span class="property-value">${hitDice}</span>
    </div>`
  }

  // Primary Ability - handle array/object format from raw data
  let primaryAbility = classData.primaryAbility
  if (Array.isArray(primaryAbility)) {
    const abilities = primaryAbility
      .map((a: any) => Object.keys(a).filter(k => a[k] === true))
      .flat()
      .map((s: string) => s.toUpperCase())
    primaryAbility = abilities.join(' or ')
  } else if (typeof primaryAbility === 'object' && primaryAbility !== null) {
    const abilities = Object.keys(primaryAbility).filter(k => primaryAbility[k] === true)
    primaryAbility = abilities.map(s => s.toUpperCase()).join(' or ')
  }
  if (primaryAbility) {
    html += `<div class="property-item">
      <span class="property-label">Primary Ability:</span>
      <span class="property-value">${primaryAbility}</span>
    </div>`
  }

  // Saving Throws - handle both formats
  let savingThrows = classData.proficiency
  if (Array.isArray(savingThrows)) {
    savingThrows = savingThrows.map((s: string) => s.toUpperCase()).join(', ')
  } else if (classData.startingProficiencies?.savingThrows) {
    savingThrows = classData.startingProficiencies.savingThrows.map((s: string) => s.toUpperCase()).join(', ')
  }
  if (savingThrows) {
    html += `<div class="property-item">
      <span class="property-label">Saving Throw Proficiencies:</span>
      <span class="property-value">${savingThrows}</span>
    </div>`
  }

  if (classData.spellcastingAbility) {
    html += `<div class="property-item">
      <span class="property-label">Spellcasting Ability:</span>
      <span class="property-value">${formatAbilityScore(classData.spellcastingAbility)}</span>
    </div>`
  }

  if (classData.subclassTitle) {
    html += `<div class="property-item">
      <span class="property-label">Subclass Type:</span>
      <span class="property-value">${classData.subclassTitle}</span>
    </div>`
  }

  html += '</div>'

  // Description
  if (classData.description) {
    html += '<div class="class-description">'
    html += processFormattingTags(classData.description)
    html += '</div>'
  }

  // Entries (from raw 5etools data)
  if (classData.entries && Array.isArray(classData.entries)) {
    html += '<div class="class-entries">'
    html += formatEntries(classData.entries)
    html += '</div>'
  }

  // Source
  html += `<div class="source-info">Source: ${classData.source || 'Unknown'}${classData.page ? `, p. ${classData.page}` : ''}</div>`

  html += '</div>'
  return html
}

/**
 * Format a class with a specific subclass highlighted/expanded.
 * Shows full class info with the selected subclass featured prominently.
 */
async function formatClassWithSelectedSubclass(
  classData: ClassWithDetails | ClassSummary | any,
  selectedSubclass: Subclass
): Promise<string> {
  // Extract the actual class data if wrapped
  const cls = 'class' in classData && typeof classData.class === 'object'
    ? classData.class
    : classData

  let html = '<div class="class-details">'

  // Header section with subclass indicator
  html += '<div class="class-header-section">'
  html += `<h2>${cls.name}: ${selectedSubclass.name}</h2>`
  html += '</div>'

  // Basic properties
  html += '<div class="class-properties-grid">'

  // Format hit dice
  if (cls.hd) {
    const hdText = typeof cls.hd === 'object'
      ? `${cls.hd.number || 1}d${cls.hd.faces || 6}`
      : '1d6'
    html += `<div class="property-item">
      <span class="property-label">Hit Dice:</span>
      <span class="property-value">${hdText}</span>
    </div>`
  }

  // Format proficiencies
  if (cls.startingProficiencies?.savingThrows) {
    const saves = cls.startingProficiencies.savingThrows
      .map((s: string) => s.toUpperCase())
      .join(', ')
    html += `<div class="property-item">
      <span class="property-label">Saving Throw Proficiencies:</span>
      <span class="property-value">${saves}</span>
    </div>`
  }

  // Spellcasting - use subclass's if it has one, otherwise class's
  const spellcastingAbility = selectedSubclass.spellcastingAbility || cls.spellcastingAbility
  if (spellcastingAbility) {
    html += `<div class="property-item">
      <span class="property-label">Spellcasting Ability:</span>
      <span class="property-value">${formatAbilityScore(spellcastingAbility)}</span>
    </div>`
  }

  // Caster progression if subclass adds it
  if (selectedSubclass.casterProgression) {
    html += `<div class="property-item">
      <span class="property-label">Caster Progression:</span>
      <span class="property-value">${selectedSubclass.casterProgression}</span>
    </div>`
  }

  if (cls.subclassTitle) {
    html += `<div class="property-item">
      <span class="property-label">Subclass Type:</span>
      <span class="property-value">${cls.subclassTitle}</span>
    </div>`
  }

  html += '</div>'

  // === CLASS FLUFF SECTION (Description and images) ===
  if (cls.fluff) {
    html += '<div class="class-fluff-section">'
    html += '<h3>Description</h3>'

    // Add images if present
    if (cls.fluff.images && cls.fluff.images.length > 0) {
      html += '<div class="class-images">'
      for (const image of cls.fluff.images) {
        if (typeof image === 'object' && image.href && image.href.path) {
          try {
            const response = await invoke<any>('serve_book_image', {
              bookId: cls.source,
              imagePath: image.href.path
            })
            if (response && response.success && response.data) {
              html += `<img src="${response.data}" alt="${cls.name}" class="class-image" style="max-width: 400px; max-height: 400px; width: auto; height: auto; object-fit: contain; display: block; margin: 1rem auto;" />`
            }
          } catch (e) {
            // Silently fail if image can't be loaded
          }
        }
      }
      html += '</div>'
    }

    // Add fluff entries
    if (cls.fluff.entries && cls.fluff.entries.length > 0) {
      html += '<div class="fluff-entries">'
      html += formatEntries(cls.fluff.entries)
      html += '</div>'
    }
    html += '</div>'
  }

  // Class entries (features description from 5etools)
  if (cls.entries && cls.entries.length > 0) {
    html += '<div class="class-entries">'
    html += '<h3>Class Features</h3>'
    for (const entry of cls.entries) {
      html += formatEntry(entry)
    }
    html += '</div>'
  }

  // === SELECTED SUBCLASS SECTION (Featured prominently) ===
  html += '<div class="selected-subclass-section">'
  html += `<h3>${selectedSubclass.name}</h3>`

  if (selectedSubclass.shortName && selectedSubclass.shortName !== selectedSubclass.name) {
    html += `<p class="subclass-short-name">(${selectedSubclass.shortName})</p>`
  }

  // Subclass fluff/description with images
  if (selectedSubclass.fluff) {
    html += '<div class="subclass-fluff-section">'

    // Images
    if (selectedSubclass.fluff.images && selectedSubclass.fluff.images.length > 0) {
      html += '<div class="subclass-images">'
      for (const image of selectedSubclass.fluff.images) {
        if (typeof image === 'object' && image.href && image.href.path) {
          try {
            const response = await invoke<any>('serve_book_image', {
              bookId: selectedSubclass.source,
              imagePath: image.href.path
            })
            if (response && response.success && response.data) {
              html += `<img src="${response.data}" alt="${selectedSubclass.name}" class="subclass-image" style="max-width: 400px; max-height: 400px; width: auto; height: auto; object-fit: contain; display: block; margin: 1rem auto;" />`
            }
          } catch (e) {
            // Silently fail if image can't be loaded
          }
        }
      }
      html += '</div>'
    }

    // Fluff entries - show all of them for richness
    if (selectedSubclass.fluff.entries && selectedSubclass.fluff.entries.length > 0) {
      html += '<div class="subclass-description">'
      html += formatEntries(selectedSubclass.fluff.entries)
      html += '</div>'
    }
    html += '</div>'
  }

  // Subclass features with full details
  if (selectedSubclass.subclassFeatures && selectedSubclass.subclassFeatures.length > 0) {
    html += '<div class="subclass-features-section">'
    html += '<h4>Subclass Features</h4>'

    // Parse and group features by level
    const featuresByLevel = new Map<number, string[]>()

    for (const featureRef of selectedSubclass.subclassFeatures) {
      const parts = typeof featureRef === 'string' ? featureRef.split('|') : []
      if (parts.length >= 6) {
        const featureName = parts[0]
        const level = parseInt(parts[5]) || 1

        if (!featuresByLevel.has(level)) {
          featuresByLevel.set(level, [])
        }
        featuresByLevel.get(level)!.push(featureName)
      }
    }

    const levels = Array.from(featuresByLevel.keys()).sort((a, b) => a - b)

    if (levels.length > 0) {
      html += '<div class="features-list">'
      for (const level of levels) {
        const levelFeatures = featuresByLevel.get(level)!
        html += `<div class="feature-level-group">`
        html += `<strong>${formatOrdinal(level)} Level:</strong> `
        html += levelFeatures.join(', ')
        html += `</div>`
      }
      html += '</div>'
    }

    html += '</div>'
  }

  // Subclass table groups (e.g., spell slots for Eldritch Knight)
  if (selectedSubclass.subclassTableGroups && selectedSubclass.subclassTableGroups.length > 0) {
    html += '<div class="subclass-tables-section">'
    html += '<h4>Subclass Progression</h4>'
    for (const tableGroup of selectedSubclass.subclassTableGroups) {
      html += formatTable(tableGroup)
    }
    html += '</div>'
  }

  html += '</div>' // selected-subclass-section

  // === CLASS FEATURES SECTION ===
  // Show base class features (these apply to all subclasses)
  if (cls.classFeatures && cls.classFeatures.length > 0) {
    html += '<div class="class-features-section">'
    html += '<h3>Class Features</h3>'

    // Group features by level with full reference info
    const featuresByLevel: { [key: number]: Array<{ name: string; className: string; source: string; level: number }> } = {}

    for (const feature of cls.classFeatures) {
      let featureStr = ''
      if (typeof feature === 'string') {
        featureStr = feature
      } else if (typeof feature === 'object' && feature.classFeature) {
        featureStr = feature.classFeature
      }

      if (featureStr) {
        const parts = featureStr.split('|')
        if (parts.length >= 4) {
          const featureName = parts[0]
          const className = parts[1] || cls.name
          const classSource = parts[2] || cls.source
          const level = parseInt(parts[3]) || 1

          if (!featuresByLevel[level]) featuresByLevel[level] = []
          featuresByLevel[level].push({ name: featureName, className, source: classSource, level })
        }
      }
    }

    // Display features by level with clickable links
    html += '<div class="features-list">'
    for (let level = 1; level <= 20; level++) {
      if (featuresByLevel[level] && featuresByLevel[level].length > 0) {
        html += `<div class="feature-level-group">`
        html += `<strong>${formatOrdinal(level)} Level:</strong> `
        const featureLinks = featuresByLevel[level].map(f =>
          `<a href="#" class="cross-ref-link feature-ref" data-ref-type="classFeature" data-ref-name="${f.name}" data-class-name="${f.className}" data-ref-source="${f.source}" data-level="${f.level}">${f.name}</a>`
        )
        html += featureLinks.join(', ')
        html += `</div>`
      }
    }
    html += '</div>'
    html += '</div>'
  }

  // Class Table Groups (includes spell slots for casters)
  if (cls.classTableGroups && cls.classTableGroups.length > 0) {
    html += '<div class="class-tables-section">'
    html += '<h3>Class Progression</h3>'
    for (const tableGroup of cls.classTableGroups) {
      html += formatTable(tableGroup)
    }
    html += '</div>'
  }

  // Source info
  html += `<div class="source-info">`
  html += `Class: ${cls.source}${cls.page ? `, p. ${cls.page}` : ''}`
  if (selectedSubclass.source !== cls.source || selectedSubclass.page !== cls.page) {
    html += ` | Subclass: ${selectedSubclass.source}${selectedSubclass.page ? `, p. ${selectedSubclass.page}` : ''}`
  }
  html += `</div>`

  html += '</div>'
  return html
}

async function formatFullClassDetails(classDetails: ClassWithDetails): Promise<string> {
  // Handle both wrapped and direct class data
  const classData = 'class' in classDetails ? classDetails.class : classDetails
  let html = '<div class="class-details">'
  
  // Header section
  html += '<div class="class-header-section">'
  html += `<h2>${classData.name}</h2>`
  html += '</div>'
  
  // Basic properties
  html += '<div class="class-properties-grid">'
  
  // Format hit dice
  if (classData.hd) {
    const hdText = typeof classData.hd === 'object' 
      ? `${classData.hd.number || 1}d${classData.hd.faces || 6}`
      : '1d6'
    html += `<div class="property-item">
      <span class="property-label">Hit Dice:</span>
      <span class="property-value">${hdText}</span>
    </div>`
  }
  
  // Format proficiencies
  if (classData.startingProficiencies?.savingThrows) {
    const saves = classData.startingProficiencies.savingThrows
      .map((s: string) => s.toUpperCase())
      .join(', ')
    html += `<div class="property-item">
      <span class="property-label">Saving Throw Proficiencies:</span>
      <span class="property-value">${saves}</span>
    </div>`
  }
  
  if (classData.spellcastingAbility) {
    html += `<div class="property-item">
      <span class="property-label">Spellcasting Ability:</span>
      <span class="property-value">${formatAbilityScore(classData.spellcastingAbility)}</span>
    </div>`
  }
  
  if (classData.subclassTitle) {
    html += `<div class="property-item">
      <span class="property-label">Subclass Type:</span>
      <span class="property-value">${classData.subclassTitle}</span>
    </div>`
  }
  
  html += '</div>'
  
  // Add fluff description if available
  if (classData.fluff) {
    html += '<div class="class-fluff-section">'
    html += '<h3>Description</h3>'
    
    // Add images if present
    if (classData.fluff.images && classData.fluff.images.length > 0) {
      html += '<div class="class-images">'
      for (const image of classData.fluff.images) {
        if (typeof image === 'object' && image.href && image.href.path) {
          try {
            const response = await invoke<any>('serve_book_image', {
              bookId: classData.source,
              imagePath: image.href.path
            })
            if (response && response.success && response.data) {
              html += `<img src="${response.data}" alt="${classData.name}" class="class-image" style="max-width: 400px; max-height: 400px; width: auto; height: auto; object-fit: contain; display: block; margin: 1rem auto;" />`
            }
          } catch (e) {
          }
        }
      }
      html += '</div>'
    }
    
    // Add fluff entries
    if (classData.fluff.entries && classData.fluff.entries.length > 0) {
      html += '<div class="fluff-entries">'
      html += formatEntries(classData.fluff.entries)
      html += '</div>'
    }
    html += '</div>'
  }
  
  // Class features sections from entries
  if (classData.entries && classData.entries.length > 0) {
    html += '<div class="class-entries">'
    html += '<h3>Class Features</h3>'
    for (const entry of classData.entries) {
      html += formatEntry(entry)
    }
    html += '</div>'
  }
  
  // Subclasses
  if (classDetails.subclasses && classDetails.subclasses.length > 0) {
    html += '<div class="subclasses-section">'
    html += `<h3>${classData.subclassTitle || 'Subclasses'}</h3>`
    
    for (const subclass of classDetails.subclasses) {
      // Find matching fluff for this subclass
      const subclassFluff = classDetails.subclass_fluff?.find(f => 
        f.name === subclass.name && f.source === subclass.source
      )
      
      // Find the introductory feature for this subclass with more flexible matching
      const subclassIntroFeature = classDetails.subclass_features?.find(f => {
        // Try both snake_case and camelCase field names
        const shortName = (f as any).subclassShortName || f.subclass_short_name
        const subclassSource = (f as any).subclassSource || f.subclass_source
        
        // Check if this feature belongs to this subclass
        const nameMatch = shortName === subclass.shortName || 
                         shortName === subclass.name ||
                         (subclass.shortName && shortName?.toLowerCase() === subclass.shortName.toLowerCase()) ||
                         // Also try matching against the feature name if it's the main path description
                         (f.name === subclass.name && (f.level === 3 || f.level === 1 || f.level === 2))
        
        const sourceMatch = subclassSource === subclass.source
        const levelMatch = f.level === 3 || f.level === 1 || f.level === 2
        
        if (nameMatch && sourceMatch && levelMatch) {
          return true
        }
        return false
      })
      
      // Get all features for this subclass
      const subclassFeatures = classDetails.subclass_features?.filter(f => {
        const shortName = (f as any).subclassShortName || f.subclass_short_name
        const subclassSource = (f as any).subclassSource || f.subclass_source
        
        return (shortName === subclass.shortName || 
                shortName === subclass.name ||
                // For Path of the Berserker, shortName might be "Berserker"
                (subclass.name.includes(shortName) && shortName)) &&
               subclassSource === subclass.source
      })
      
      html += await formatSubclass(subclass, subclassFluff, subclassIntroFeature, subclassFeatures)
    }
    html += '</div>'
  }
  
  // Class Features Table
  if (classDetails.features && classDetails.features.length > 0) {
    html += '<div class="features-section">'
    html += '<h3>Features by Level</h3>'
    html += formatFeaturesTable(classDetails.features)
    html += '</div>'
  }
  
  // Class Table Groups (includes spell slots for casters)
  if (classData.classTableGroups && classData.classTableGroups.length > 0) {
    html += '<div class="class-tables-section">'
    html += '<h3>Class Progression</h3>'
    for (const tableGroup of classData.classTableGroups) {
      html += formatTable(tableGroup)
    }
    html += '</div>'
  }
  
  // Class Features - parse from string references
  // Format: "Feature Name|ClassName|ClassSource|Level" or object with classFeature property
  if (classData.classFeatures && classData.classFeatures.length > 0) {
    html += '<div class="class-features-section">'
    html += '<h3>Class Features</h3>'

    // Group features by level with full reference info
    const featuresByLevel: { [key: number]: Array<{ name: string; className: string; source: string; level: number }> } = {}

    for (const feature of classData.classFeatures) {
      let featureStr = ''
      if (typeof feature === 'string') {
        featureStr = feature
      } else if (typeof feature === 'object' && feature.classFeature) {
        featureStr = feature.classFeature
      }

      if (featureStr) {
        const parts = featureStr.split('|')
        if (parts.length >= 4) {
          const featureName = parts[0]
          const className = parts[1] || classData.name
          const classSource = parts[2] || classData.source
          const level = parseInt(parts[3]) || 1

          if (!featuresByLevel[level]) featuresByLevel[level] = []
          featuresByLevel[level].push({ name: featureName, className, source: classSource, level })
        }
      }
    }

    // Display features by level with clickable links
    html += '<div class="features-list">'
    for (let level = 1; level <= 20; level++) {
      if (featuresByLevel[level] && featuresByLevel[level].length > 0) {
        html += `<div class="feature-level-group">`
        html += `<strong>${formatOrdinal(level)} Level:</strong> `
        const featureLinks = featuresByLevel[level].map(f =>
          `<a href="#" class="cross-ref-link feature-ref" data-ref-type="classFeature" data-ref-name="${f.name}" data-class-name="${f.className}" data-ref-source="${f.source}" data-level="${f.level}">${f.name}</a>`
        )
        html += featureLinks.join(', ')
        html += `</div>`
      }
    }
    html += '</div>'
    html += '</div>'
  }
  
  // Source
  html += `<div class="source-info">Source: ${classData.source}${classData.page ? `, p. ${classData.page}` : ''}</div>`
  
  html += '</div>'
  return html
}

function formatEntry(entry: any): string {
  if (typeof entry === 'string') {
    return `<p>${processFormattingTags(entry)}</p>`
  }
  
  if (entry.type === 'entries') {
    let html = ''
    if (entry.name) {
      html += `<h4>${entry.name}</h4>`
    }
    if (entry.entries) {
      for (const subEntry of entry.entries) {
        html += formatEntry(subEntry)
      }
    }
    return html
  }
  
  if (entry.type === 'list') {
    let html = '<ul>'
    for (const item of entry.items) {
      html += `<li>${processFormattingTags(item)}</li>`
    }
    html += '</ul>'
    return html
  }
  
  if (entry.type === 'table') {
    return formatTable(entry)
  }
  
  return ''
}

async function formatSubclass(subclass: Subclass, fluff?: SubclassFluff, introFeature?: SubclassFeature, allSubclassFeatures?: SubclassFeature[]): Promise<string> {
  let html = '<div class="subclass-item">'
  html += `<h4>${subclass.name}</h4>`
  
  // Only show short_name if it exists and is different from name
  if (subclass.shortName && subclass.shortName !== subclass.name) {
    html += `<p class="subclass-short-name">Also known as: ${subclass.shortName}</p>`
  }
  
  // Add intro feature description if available
  if (introFeature && introFeature.entries && introFeature.entries.length > 0) {
    html += '<div class="subclass-description">'
    // Show just the first entry as the main description
    const firstEntry = introFeature.entries[0]
    if (typeof firstEntry === 'string') {
      html += `<p>${processFormattingTags(firstEntry)}</p>`
    } else {
      html += formatEntry(firstEntry)
    }
    html += '</div>'
  }
  
  // Add spellcasting info if present
  if (subclass.spellcastingAbility || subclass.casterProgression) {
    html += '<div class="subclass-spellcasting">'
    if (subclass.spellcastingAbility) {
      html += `<span>Spellcasting: ${formatAbilityScore(subclass.spellcastingAbility)}</span>`
    }
    if (subclass.casterProgression) {
      html += `<span> (${subclass.casterProgression} caster)</span>`
    }
    html += '</div>'
  }
  
  // Add fluff description if available
  if (fluff) {
    // Add images if present
    if (fluff.images && fluff.images.length > 0) {
      html += '<div class="subclass-images">'
      for (const image of fluff.images) {
        if (typeof image === 'object' && image.href && image.href.path) {
          try {
            const response = await invoke<any>('serve_book_image', {
              bookId: subclass.source,
              imagePath: image.href.path
            })
            if (response && response.success && response.data) {
              html += `<img src="${response.data}" alt="${subclass.name}" class="subclass-image" style="max-width: 300px; max-height: 300px; width: auto; height: auto; object-fit: contain; display: block; margin: 0.5rem auto;" />`
            }
          } catch (e) {
          }
        }
      }
      html += '</div>'
    }
    
    // Add fluff entries
    if (fluff.entries && fluff.entries.length > 0) {
      html += '<div class="subclass-fluff">'
      // Only show first 2 entries as preview to keep it concise
      const entriesToShow = fluff.entries.slice(0, 2)
      html += formatEntries(entriesToShow)
      if (fluff.entries.length > 2) {
        html += '<p class="more-content">...</p>'
      }
      html += '</div>'
    }
  }
  
  // Add list of subclass features if available
  if (allSubclassFeatures && allSubclassFeatures.length > 0) {
    // Group features by level
    const featuresByLevel = new Map<number, SubclassFeature[]>()
    
    for (const feature of allSubclassFeatures) {
      // Skip the intro feature we already displayed
      if (feature === introFeature) continue
      
      if (!featuresByLevel.has(feature.level)) {
        featuresByLevel.set(feature.level, [])
      }
      featuresByLevel.get(feature.level)!.push(feature)
    }
    
    // Sort levels and display features
    const levels = Array.from(featuresByLevel.keys()).sort((a, b) => a - b)
    
    if (levels.length > 0) {
      html += '<div class="subclass-features">'
      html += '<h5>Subclass Features</h5>'
      html += '<ul class="feature-list">'
      
      for (const level of levels) {
        const levelFeatures = featuresByLevel.get(level)!
        for (const feature of levelFeatures) {
          html += `<li><strong>${formatOrdinal(level)} Level:</strong> ${feature.name}</li>`
        }
      }
      
      html += '</ul>'
      html += '</div>'
    }
  }
  
  // Add subclass features summary if present (legacy)
  if (subclass.subclassFeatures && !allSubclassFeatures) {
    html += '<div class="subclass-features-preview">'
    if (Array.isArray(subclass.subclassFeatures)) {
      const featureCount = subclass.subclassFeatures.length
      html += `<p class="feature-count">${featureCount} unique feature${featureCount !== 1 ? 's' : ''}</p>`
    }
    html += '</div>'
  }
  
  html += `<p class="source-info">Source: ${subclass.source}${subclass.page ? `, p. ${subclass.page}` : ''}</p>`
  html += '</div>'
  return html
}

function formatFeaturesTable(features: ClassFeature[]): string {
  // Group features by level
  const featuresByLevel = new Map<number, ClassFeature[]>()
  for (const feature of features) {
    if (!featuresByLevel.has(feature.level)) {
      featuresByLevel.set(feature.level, [])
    }
    featuresByLevel.get(feature.level)!.push(feature)
  }

  // Sort levels
  const levels = Array.from(featuresByLevel.keys()).sort((a, b) => a - b)

  let html = '<div class="features-by-level">'

  for (const level of levels) {
    const levelFeatures = featuresByLevel.get(level)!
    html += `<div class="level-section">`
    html += `<h4>${formatOrdinal(level)} Level</h4>`

    for (const feature of levelFeatures) {
      // Use details/summary for expandable content if feature has entries
      if (feature.entries && feature.entries.length > 0) {
        html += `<details class="feature-details">`
        html += `<summary class="feature-name">${feature.name}</summary>`
        html += `<div class="feature-content">`
        html += formatEntries(feature.entries)
        html += `</div>`
        html += `</details>`
      } else {
        // No entries, just show the name
        html += `<p class="feature-name-only"><strong>${feature.name}</strong></p>`
      }
    }

    html += `</div>`
  }

  html += '</div>'
  return html
}
export function formatTable(table: any): string {
  // Handle both 'rows' format and 'rowsSpellProgression' format (spell slots)
  const hasRows = table.rows && table.rows.length > 0
  const hasSpellProgressionRows = table.rowsSpellProgression && table.rowsSpellProgression.length > 0

  // Skip tables without any row data
  if (!hasRows && !hasSpellProgressionRows) {
    return ''
  }

  let html = '<table class="entry-table">'

  // Caption
  if (table.caption) {
    html += `<caption>${table.caption}</caption>`
  }

  // Determine data source and row count
  const dataRows = hasRows ? table.rows : table.rowsSpellProgression
  const rowCount = dataRows.length

  // Headers - Add "Level" as first column if not present
  if (table.colLabels) {
    html += '<thead><tr>'
    // Check if first label is for level column
    const hasLevelColumn = table.colLabels[0]?.toLowerCase().includes('level')
    if (!hasLevelColumn && rowCount === 20) {
      // This is likely a class progression table, add Level header
      html += '<th>Level</th>'
    }
    for (const label of table.colLabels) {
      // Process formatting tags in header labels
      html += `<th>${processFormattingTags(label)}</th>`
    }
    html += '</tr></thead>'
  }

  // Body
  html += '<tbody>'
  const hasLevelColumn = table.colLabels && table.colLabels[0]?.toLowerCase().includes('level')
  const isProgressionTable = !hasLevelColumn && rowCount === 20

  // Check if first column is a dice roll column
  const isDiceColumn = table.colLabels && table.colLabels[0]?.includes('{@dice')

  for (let i = 0; i < rowCount; i++) {
    const row = dataRows[i]
    html += '<tr>'

    // Add level column if this is a progression table without level column
    if (isProgressionTable) {
      html += `<td>${i + 1}</td>`
    }

    for (let j = 0; j < row.length; j++) {
      const cell = row[j]
      // Handle various cell types
      let cellContent = ''

      // If this is the first column and it's a dice column, and the cell is empty/undefined,
      // fill it with the row number
      if (j === 0 && isDiceColumn && (!cell || cell === '')) {
        cellContent = String(i + 1)
      } else if (typeof cell === 'object' && cell !== null) {
        // Handle bonus objects like {"type":"bonus","value":2}
        if (cell.type === 'bonus' && cell.value !== undefined) {
          cellContent = `+${cell.value}`
        }
        // Handle cell objects with roll property {"roll": {"exact": 1}, "type": "cell"}
        else if (cell.type === 'cell' && cell.roll) {
          if (cell.roll.exact !== undefined) {
            cellContent = String(cell.roll.exact)
          } else if (cell.roll.min !== undefined && cell.roll.max !== undefined) {
            cellContent = `${cell.roll.min}-${cell.roll.max}`
          } else {
            cellContent = JSON.stringify(cell.roll)
          }
        }
        // Handle simple roll property
        else if (cell.roll) {
          cellContent = cell.roll
        }
        // Handle other object types
        else {
          cellContent = JSON.stringify(cell)
        }
      } else if (typeof cell === 'number') {
        // For spell progression, 0 means no slots - display as '-'
        cellContent = cell === 0 ? '—' : String(cell)
      } else {
        cellContent = cell || ''
      }
      html += `<td>${processFormattingTags(cellContent)}</td>`
    }
    html += '</tr>'
  }
  html += '</tbody>'

  html += '</table>'
  return html
}

function formatAbilityScore(ability: string): string {
  const abilityMap: Record<string, string> = {
    str: 'Strength',
    dex: 'Dexterity',
    con: 'Constitution',
    int: 'Intelligence',
    wis: 'Wisdom',
    cha: 'Charisma'
  }
  return abilityMap[ability.toLowerCase()] || ability.toUpperCase()
}

function formatOrdinal(n: number): string {
  const suffixes = ['th', 'st', 'nd', 'rd']
  const v = n % 100
  return n + (suffixes[(v - 20) % 10] || suffixes[v] || suffixes[0])
}

async function formatSubclassDetails(subclass: Subclass): Promise<string> {
  let html = '<div class="subclass-details">'
  
  // Header section
  html += '<div class="subclass-header-section">'
  html += `<h2>${subclass.className}: ${subclass.name}</h2>`
  if (subclass.shortName && subclass.shortName !== subclass.name) {
    html += `<p class="subclass-short-name">(${subclass.shortName})</p>`
  }
  html += '</div>'
  
  // Intro description
  if (subclass.introDescription) {
    html += '<div class="subclass-intro-section">'
    html += '<h3>Description</h3>'
    html += `<p class="subclass-intro">${processFormattingTags(subclass.introDescription)}</p>`
    html += '</div>'
  }
  
  // Basic properties
  html += '<div class="subclass-properties-grid">'
  html += `<div class="property-item">
    <span class="property-label">Class:</span>
    <span class="property-value">${subclass.className}</span>
  </div>`
  
  if (subclass.spellcastingAbility) {
    html += `<div class="property-item">
      <span class="property-label">Spellcasting Ability:</span>
      <span class="property-value">${formatAbilityScore(subclass.spellcastingAbility)}</span>
    </div>`
  }
  
  if (subclass.casterProgression) {
    html += `<div class="property-item">
      <span class="property-label">Caster Progression:</span>
      <span class="property-value">${subclass.casterProgression}</span>
    </div>`
  }
  html += '</div>'
  
  // Subclass features
  if (subclass.subclassFeatures && subclass.subclassFeatures.length > 0) {
    html += '<div class="subclass-features-section">'
    html += '<h3>Subclass Features</h3>'
    
    // Parse and group features by level
    const featuresByLevel = new Map<number, string[]>()
    
    for (const featureRef of subclass.subclassFeatures) {
      // Parse the feature reference: "FeatureName|ClassName||SubclassShortName||Level"
      const parts = featureRef.split('|')
      if (parts.length >= 6) {
        const featureName = parts[0]
        const level = parseInt(parts[5]) || 1
        
        if (!featuresByLevel.has(level)) {
          featuresByLevel.set(level, [])
        }
        featuresByLevel.get(level)!.push(featureName)
      }
    }
    
    // Sort levels and display
    const levels = Array.from(featuresByLevel.keys()).sort((a, b) => a - b)
    
    html += '<div class="feature-progression">'
    for (const level of levels) {
      const levelFeatures = featuresByLevel.get(level)!
      html += `<div class="feature-level-group">`
      html += `<h4>${formatOrdinal(level)} Level</h4>`
      html += '<ul>'
      for (const featureName of levelFeatures) {
        html += `<li><strong>${featureName}</strong></li>`
      }
      html += '</ul>'
      html += `</div>`
    }
    html += '</div>'
    
    html += '</div>'
  }
  
  // Add fluff description if available
  if (subclass.fluff) {
    html += '<div class="subclass-fluff-section">'
    html += '<h3>Description</h3>'
    
    // Add images if present
    if (subclass.fluff.images && subclass.fluff.images.length > 0) {
      html += '<div class="subclass-images">'
      for (const image of subclass.fluff.images) {
        if (typeof image === 'object' && image.href && image.href.path) {
          try {
            const response = await invoke<any>('serve_book_image', {
              bookId: subclass.source,
              imagePath: image.href.path
            })
            if (response && response.success && response.data) {
              html += `<img src="${response.data}" alt="${subclass.name}" class="subclass-image" style="max-width: 400px; max-height: 400px; width: auto; height: auto; object-fit: contain; display: block; margin: 1rem auto;" />`
            }
          } catch (e) {
            // Silently fail if image can't be loaded
          }
        }
      }
      html += '</div>'
    }
    
    // Add fluff entries
    if (subclass.fluff.entries && subclass.fluff.entries.length > 0) {
      html += '<div class="fluff-entries">'
      html += formatEntries(subclass.fluff.entries)
      html += '</div>'
    }
    html += '</div>'
  }
  
  // Source info
  html += `<div class="source-info">Source: ${subclass.source}`
  if (subclass.page) {
    html += `, p. ${subclass.page}`
  }
  html += '</div>'

  html += '</div>'
  return html
}

/**
 * Format a class feature for display in a modal.
 */
export async function formatClassFeatureDetails(feature: any, className?: string, level?: string): Promise<string> {
  let html = '<div class="class-feature-details">'

  // Header with level info
  html += '<div class="feature-header">'
  if (level) {
    html += `<span class="feature-level">Level ${level}</span>`
  }
  if (className) {
    html += `<span class="feature-class">${className} Feature</span>`
  }
  html += '</div>'

  // Feature entries (the actual description)
  if (feature.entries && Array.isArray(feature.entries)) {
    html += '<div class="feature-content">'
    html += formatEntries(feature.entries)
    html += '</div>'
  }

  // Source info
  if (feature.source) {
    html += `<div class="source-info">Source: ${feature.source}`
    if (feature.page) {
      html += `, p. ${feature.page}`
    }
    html += '</div>'
  }

  html += '</div>'
  return html
}