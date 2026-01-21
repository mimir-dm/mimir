import { formatEntries, processFormattingTags } from '../utils/textFormatting'

interface Vehicle {
  name: string
  source: string
  vehicle_type?: string
  size?: string
  page?: number
  cap_crew?: number
  cap_passenger?: number
  cap_cargo?: number
  ac?: number
  hp?: number
  speed?: any
  pace?: number
  dimensions?: string[]
  immune?: string[]
  resist?: string[]
  vulnerable?: string[]
  terrain?: string[]
  weapon?: any[]
  entries?: any[]
  srd?: string
}

export function formatVehicleDetails(vehicle: Vehicle): string {
  if (!vehicle) return '<div>No vehicle data available</div>'
  
  let html = '<div class="vehicle-details">'
  
  // Header with name and type
  html += `
    <div class="detail-header">
      <h2>${vehicle.name}</h2>
      ${vehicle.srd ? `<div class="srd-name">SRD: ${vehicle.srd}</div>` : ''}
      <div class="header-tags">
        ${vehicle.vehicle_type ? `<span class="tag tag-type">${vehicle.vehicle_type}</span>` : ''}
        ${vehicle.size ? `<span class="tag tag-size">${formatSize(vehicle.size)}</span>` : ''}
        ${vehicle.terrain ? vehicle.terrain.map(t => `<span class="tag tag-terrain">${t}</span>`).join('') : ''}
      </div>
    </div>
  `
  
  // Stats grid
  html += '<div class="stats-grid">'
  
  // Capacity
  if (vehicle.cap_crew || vehicle.cap_passenger || vehicle.cap_cargo) {
    html += '<div class="stat-block">'
    html += '<h4>Capacity</h4>'
    html += '<div class="stat-lines">'
    if (vehicle.cap_crew) {
      html += `<div class="stat-line"><strong>Crew:</strong> ${vehicle.cap_crew}</div>`
    }
    if (vehicle.cap_passenger) {
      html += `<div class="stat-line"><strong>Passengers:</strong> ${vehicle.cap_passenger}</div>`
    }
    if (vehicle.cap_cargo) {
      html += `<div class="stat-line"><strong>Cargo:</strong> ${vehicle.cap_cargo} ton${vehicle.cap_cargo !== 1 ? 's' : ''}</div>`
    }
    html += '</div>'
    html += '</div>'
  }
  
  // Defenses
  if (vehicle.ac || vehicle.hp) {
    html += '<div class="stat-block">'
    html += '<h4>Defenses</h4>'
    html += '<div class="stat-lines">'
    if (vehicle.ac) {
      html += `<div class="stat-line"><strong>Armor Class:</strong> ${vehicle.ac}</div>`
    }
    if (vehicle.hp) {
      html += `<div class="stat-line"><strong>Hit Points:</strong> ${vehicle.hp}</div>`
    }
    html += '</div>'
    html += '</div>'
  }
  
  // Speed
  if (vehicle.speed || vehicle.pace) {
    html += '<div class="stat-block">'
    html += '<h4>Movement</h4>'
    html += '<div class="stat-lines">'
    if (vehicle.speed) {
      const speeds: string[] = []
      if (vehicle.speed.walk) speeds.push(`Walk ${vehicle.speed.walk} ft.`)
      if (vehicle.speed.swim) speeds.push(`Swim ${vehicle.speed.swim} ft.`)
      if (vehicle.speed.fly) speeds.push(`Fly ${vehicle.speed.fly} ft.`)
      if (vehicle.speed.burrow) speeds.push(`Burrow ${vehicle.speed.burrow} ft.`)
      if (vehicle.speed.climb) speeds.push(`Climb ${vehicle.speed.climb} ft.`)
      if (speeds.length > 0) {
        html += `<div class="stat-line"><strong>Speed:</strong> ${speeds.join(', ')}</div>`
      }
      if (vehicle.speed.note) {
        html += `<div class="stat-line note">${vehicle.speed.note}</div>`
      }
    }
    if (vehicle.pace) {
      html += `<div class="stat-line"><strong>Travel Pace:</strong> ${vehicle.pace} miles per hour</div>`
    }
    html += '</div>'
    html += '</div>'
  }
  
  // Dimensions
  if (vehicle.dimensions && vehicle.dimensions.length > 0) {
    html += '<div class="stat-block">'
    html += '<h4>Dimensions</h4>'
    html += '<div class="stat-lines">'
    html += `<div class="stat-line">${vehicle.dimensions.join(' × ')}</div>`
    html += '</div>'
    html += '</div>'
  }
  
  html += '</div>' // End stats-grid
  
  // Damage Immunities/Resistances
  if (vehicle.immune || vehicle.resist || vehicle.vulnerable) {
    html += '<div class="defenses-section">'
    if (vehicle.immune && vehicle.immune.length > 0) {
      html += `<div class="defense-line"><strong>Damage Immunities:</strong> ${vehicle.immune.join(', ')}</div>`
    }
    if (vehicle.resist && vehicle.resist.length > 0) {
      html += `<div class="defense-line"><strong>Damage Resistances:</strong> ${vehicle.resist.join(', ')}</div>`
    }
    if (vehicle.vulnerable && vehicle.vulnerable.length > 0) {
      html += `<div class="defense-line"><strong>Damage Vulnerabilities:</strong> ${vehicle.vulnerable.join(', ')}</div>`
    }
    html += '</div>'
  }
  
  // Weapons
  if (vehicle.weapon && vehicle.weapon.length > 0) {
    html += '<div class="content-section">'
    html += '<h3>Weapons</h3>'
    for (const weapon of vehicle.weapon) {
      html += '<div class="weapon-block">'
      html += `<h4>${weapon.name}${weapon.count && weapon.count > 1 ? ` (${weapon.count})` : ''}</h4>`
      if (weapon.entries) {
        html += formatEntries(weapon.entries)
      }
      html += '</div>'
    }
    html += '</div>'
  }
  
  // Main description
  if (vehicle.entries && vehicle.entries.length > 0) {
    html += '<div class="content-section">'
    html += '<h3>Description</h3>'
    html += formatEntries(vehicle.entries)
    html += '</div>'
  }
  
  // Source info
  html += `
    <div class="source-info">
      <strong>Source:</strong> ${vehicle.source}
      ${vehicle.page ? `, p. ${vehicle.page}` : ''}
    </div>
  `
  
  html += '</div>'
  
  // Add styles
  html += `
    <style>
      .vehicle-details {
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
      
      .srd-name {
        font-style: italic;
        color: var(--color-text-secondary, #999);
        margin-bottom: var(--spacing-sm, 8px);
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
      
      .tag-type {
        background: rgba(33, 150, 243, 0.1);
        color: #2196f3;
        border: 1px solid rgba(33, 150, 243, 0.3);
      }
      
      .tag-size {
        background: rgba(156, 39, 176, 0.1);
        color: #9c27b0;
        border: 1px solid rgba(156, 39, 176, 0.3);
      }
      
      .tag-terrain {
        background: rgba(76, 175, 80, 0.1);
        color: #4caf50;
        border: 1px solid rgba(76, 175, 80, 0.3);
        text-transform: capitalize;
      }
      
      .stats-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: var(--spacing-md, 12px);
        margin: var(--spacing-lg, 16px) 0;
      }
      
      .stat-block {
        background: var(--color-surface, #1a1a1a);
        padding: var(--spacing-md, 12px);
        border-radius: 6px;
        border: 1px solid var(--color-border-light, #262626);
      }
      
      .stat-block h4 {
        margin: 0 0 var(--spacing-sm, 8px) 0;
        color: var(--color-accent, #ff6b6b);
        font-size: 0.9rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
      }
      
      .stat-lines {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-xs, 4px);
      }
      
      .stat-line {
        font-size: 0.9rem;
      }
      
      .stat-line.note {
        font-style: italic;
        color: var(--color-text-secondary, #999);
        font-size: 0.85rem;
      }
      
      .defenses-section {
        margin: var(--spacing-md, 12px) 0;
        padding: var(--spacing-sm, 8px);
        background: rgba(255, 193, 7, 0.05);
        border-left: 3px solid #ffc107;
      }
      
      .defense-line {
        margin: var(--spacing-xs, 4px) 0;
      }
      
      .content-section {
        margin: var(--spacing-lg, 16px) 0;
      }
      
      .content-section h3 {
        color: var(--color-primary, #4a9eff);
        margin-bottom: var(--spacing-sm, 8px);
      }
      
      .weapon-block {
        margin: var(--spacing-md, 12px) 0;
        padding: var(--spacing-sm, 8px);
        background: rgba(255, 87, 34, 0.05);
        border-left: 3px solid #ff5722;
        border-radius: 4px;
      }
      
      .weapon-block h4 {
        margin: 0 0 var(--spacing-xs, 4px) 0;
        color: #ff5722;
      }
      
      .source-info {
        margin-top: var(--spacing-lg, 16px);
        padding-top: var(--spacing-md, 12px);
        border-top: 1px solid var(--color-border, #333);
        color: var(--color-text-secondary, #999);
        font-size: 0.9rem;
      }
      
      /* Table styles for vehicle tables */
      table {
        width: 100%;
        border-collapse: collapse;
        margin: var(--spacing-md, 12px) 0;
      }
      
      table th {
        background: var(--color-surface, #1a1a1a);
        color: var(--color-primary, #4a9eff);
        padding: var(--spacing-sm, 8px);
        text-align: left;
        border: 1px solid var(--color-border, #333);
        font-weight: 600;
      }
      
      table td {
        padding: var(--spacing-sm, 8px);
        border: 1px solid var(--color-border-light, #262626);
      }
      
      table tbody tr:nth-child(even) {
        background: rgba(255, 255, 255, 0.02);
      }
      
      table tbody tr:hover {
        background: var(--color-surface-hover, #262626);
      }
    </style>
  `
  
  return html
}

function formatSize(size: string): string {
  switch (size) {
    case 'T': return 'Tiny'
    case 'S': return 'Small'
    case 'M': return 'Medium'
    case 'L': return 'Large'
    case 'H': return 'Huge'
    case 'G': return 'Gargantuan'
    default: return size
  }
}