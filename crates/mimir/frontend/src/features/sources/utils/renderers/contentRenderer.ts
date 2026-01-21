// Content rendering utilities

import { processFormattingTags } from '../textFormatting'
import type { BookSection, BookEntry, ComplexEntry } from '../../../../types/book'

/**
 * Render a book section to HTML
 */
export function renderSection(section: BookSection): string {
  if (!section) return ''
  
  // If section has entries array, render all entries
  if (section.entries && Array.isArray(section.entries)) {
    return `
      <div class="section-content">
        ${section.name ? `<h1>${section.name}</h1>` : ''}
        ${section.entries.map((e, i) => renderEntry(e, i, 0)).join('')}
      </div>
    `
  }
  
  // Otherwise render section as a single entry
  return renderEntry(section as BookEntry, 0, 0)
}

/**
 * Recursively render a book entry
 */
function renderEntry(entry: BookEntry, index: number = 0, depth: number = 0): string {
  // Handle string entries
  if (typeof entry === 'string') {
    return `<p>${processFormattingTags(entry)}</p>`
  }
  
  // Handle complex entries
  const complexEntry = entry as ComplexEntry
  
  // Section with header
  if (complexEntry.type === 'section') {
    const headerLevel = Math.min(depth + 2, 6) // h2 to h6
    const headerTag = `h${headerLevel}`
    const currentId = complexEntry.id || `section-${index}`
    
    return `
      <div class="section" id="${currentId}">
        <${headerTag}>${processFormattingTags(complexEntry.name || '')}</${headerTag}>
        ${complexEntry.entries ? complexEntry.entries.map((e, i) => renderEntry(e, i, depth + 1)).join('') : ''}
      </div>
    `
  }
  
  // Entries with optional name
  if (complexEntry.type === 'entries') {
    const currentId = complexEntry.id || `entry-${index}`
    let headerTag = ''
    if (complexEntry.name) {
      const headerLevel = depth === 0 ? 'h2' : (depth === 1 ? 'h3' : 'h4')
      headerTag = `<${headerLevel}>${processFormattingTags(complexEntry.name)}</${headerLevel}>`
    }
    return `
      <div class="entries" id="${currentId}">
        ${headerTag}
        ${complexEntry.entries ? complexEntry.entries.map((e, i) => renderEntry(e, i, depth + 1)).join('') : ''}
      </div>
    `
  }
  
  // Inset read-aloud box
  if (complexEntry.type === 'insetReadaloud') {
    return `
      <div class="inset-readaloud">
        ${complexEntry.entries ? complexEntry.entries.map((e, i) => renderEntry(e, i, depth + 1)).join('') : ''}
      </div>
    `
  }
  
  // Generic inset box
  if (complexEntry.type === 'inset') {
    return `
      <div class="inset">
        ${complexEntry.name ? `<h4>${processFormattingTags(complexEntry.name)}</h4>` : ''}
        ${complexEntry.entries ? complexEntry.entries.map((e, i) => renderEntry(e, i, depth + 1)).join('') : ''}
      </div>
    `
  }
  
  // Lists
  if (complexEntry.type === 'list') {
    return `
      <ul class="content-list">
        ${complexEntry.items ? complexEntry.items.map((item, i) => 
          `<li>${typeof item === 'string' ? processFormattingTags(item) : renderEntry(item, i, depth + 1)}</li>`
        ).join('') : ''}
      </ul>
    `
  }
  
  // Tables
  if (complexEntry.type === 'table') {
    return renderTable(complexEntry)
  }
  
  // Images
  if (complexEntry.type === 'image') {
    const imagePath = complexEntry.href?.path || ''
    const imageName = imagePath.split('/').pop() || 'image'
    const imageId = `img-${Math.random().toString(36).substr(2, 9)}`
    
    return `
      <div class="image-container">
        <div id="${imageId}" class="image-wrapper" data-image-path="${imagePath}">
          <div class="image-placeholder" style="max-width: 100%; height: auto; min-height: 100px;">
            <p>Loading image: ${imageName}</p>
          </div>
        </div>
        ${complexEntry.title ? `<p class="image-caption">${complexEntry.title}</p>` : ''}
      </div>
    `
  }
  
  // Quote blocks
  if (complexEntry.type === 'quote') {
    return `
      <blockquote>
        ${complexEntry.entries ? complexEntry.entries.map((e, i) => renderEntry(e, i, depth + 1)).join('') : ''}
        ${complexEntry.by ? `<cite>— ${complexEntry.by}</cite>` : ''}
      </blockquote>
    `
  }
  
  // Default: render as JSON for unknown types (for debugging)
  return `<pre>${JSON.stringify(entry, null, 2)}</pre>`
}

/**
 * Render a table entry
 */
function renderTable(table: ComplexEntry): string {
  if (!table) return ''
  
  let html = '<div class="table-wrapper"><table class="content-table">'
  
  // Caption
  if (table.caption) {
    html += `<caption>${processFormattingTags(table.caption)}</caption>`
  }
  
  // Column headers
  if (table.colLabels && table.colLabels.length > 0) {
    html += '<thead><tr>'
    for (const label of table.colLabels) {
      html += `<th>${processFormattingTags(label)}</th>`
    }
    html += '</tr></thead>'
  }
  
  // Table body
  if (table.rows && table.rows.length > 0) {
    html += '<tbody>'
    for (const row of table.rows) {
      html += '<tr>'
      if (Array.isArray(row)) {
        for (const cell of row) {
          html += `<td>${renderTableCell(cell)}</td>`
        }
      }
      html += '</tr>'
    }
    html += '</tbody>'
  }
  
  html += '</table></div>'
  return html
}

/**
 * Render individual table cell content
 */
function renderTableCell(cell: any): string {
  // String cells
  if (typeof cell === 'string') {
    return processFormattingTags(cell)
  }
  
  // Dice rolls
  if (cell?.roll) {
    const { min, max } = cell.roll
    if (min !== undefined && max !== undefined) {
      if (min === max) {
        return `<span class="dice-roll">${min}</span>`
      }
      return `<span class="dice-roll">${min}-${max}</span>`
    }
    return ''
  }
  
  // Complex entries in cells
  if (cell?.type === 'entries' && cell.entries) {
    return cell.entries.map((entry: any) => {
      if (typeof entry === 'string') {
        return `<div>${processFormattingTags(entry)}</div>`
      }
      if (entry?.entries) {
        return entry.entries.map((e: any) => 
          typeof e === 'string' ? `<div>${processFormattingTags(e)}</div>` : ''
        ).join('')
      }
      return ''
    }).join('')
  }
  
  // Other complex types - render as entry
  if (cell?.type) {
    return renderEntry(cell, 0, 0)
  }
  
  // Default: convert to string
  return processFormattingTags(String(cell))
}