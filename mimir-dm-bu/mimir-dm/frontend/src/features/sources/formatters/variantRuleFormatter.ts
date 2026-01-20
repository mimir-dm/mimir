import { formatEntries, processFormattingTags } from '../utils/textFormatting'

interface VariantRule {
  name: string
  source: string
  rule_type: string | null
  page: number | null
  entries: any[] | null
}

export function formatVariantRuleDetails(rule: VariantRule): string {
  if (!rule) return '<div>No variant rule data available</div>'
  
  let html = '<div class="variant-rule-details">'
  
  // Header with name and type
  html += `
    <div class="detail-header">
      <h2>${rule.name}</h2>
      <div class="header-tags">
        ${rule.rule_type ? `<span class="tag tag-type ${getTypeClass(rule.rule_type)}">${rule.rule_type}</span>` : ''}
      </div>
    </div>
  `
  
  // Main content
  if (rule.entries && rule.entries.length > 0) {
    html += '<div class="content-section">'
    html += formatEntries(rule.entries)
    html += '</div>'
  } else {
    html += '<div class="content-section"><p>No detailed rules provided.</p></div>'
  }
  
  // Source info
  html += `
    <div class="source-info">
      <strong>Source:</strong> ${rule.source}
      ${rule.page ? `, p. ${rule.page}` : ''}
    </div>
  `
  
  html += '</div>'
  
  // Add styles
  html += `
    <style>
      .variant-rule-details {
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
      
      .tag-type {
        background: rgba(156, 39, 176, 0.1);
        color: #9c27b0;
        border: 1px solid rgba(156, 39, 176, 0.3);
      }
      
      .tag-type.type-general {
        background: var(--color-surface, #1a1a1a);
        color: var(--color-text-secondary, #999);
        border: 1px solid var(--color-border, #333);
      }
      
      .tag-type.type-action-options {
        background: rgba(255, 87, 34, 0.1);
        color: #ff5722;
        border: 1px solid rgba(255, 87, 34, 0.3);
      }
      
      .tag-type.type-v {
        background: rgba(156, 39, 176, 0.1);
        color: #9c27b0;
        border: 1px solid rgba(156, 39, 176, 0.3);
      }
      
      .tag-type.type-o {
        background: rgba(33, 150, 243, 0.1);
        color: #2196f3;
        border: 1px solid rgba(33, 150, 243, 0.3);
      }
      
      .content-section {
        margin: var(--spacing-lg, 16px) 0;
        line-height: 1.6;
      }
      
      .content-section h3 {
        color: var(--color-primary, #4a9eff);
        margin-top: var(--spacing-lg, 16px);
        margin-bottom: var(--spacing-sm, 8px);
      }
      
      .content-section h4 {
        color: var(--color-accent, #ff6b6b);
        margin-top: var(--spacing-md, 12px);
        margin-bottom: var(--spacing-xs, 4px);
      }
      
      .subsection {
        margin-left: var(--spacing-md, 12px);
        padding-left: var(--spacing-sm, 8px);
        border-left: 2px solid var(--color-border-light, #262626);
      }
      
      .source-info {
        margin-top: var(--spacing-lg, 16px);
        padding-top: var(--spacing-md, 12px);
        border-top: 1px solid var(--color-border, #333);
        color: var(--color-text-secondary, #999);
        font-size: 0.9rem;
      }
      
      /* Table styles for rules that contain tables */
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
      
      ul, ol {
        margin: var(--spacing-sm, 8px) 0;
        padding-left: var(--spacing-lg, 16px);
      }
      
      li {
        margin: var(--spacing-xs, 4px) 0;
      }
      
      blockquote {
        margin: var(--spacing-md, 12px) 0;
        padding: var(--spacing-sm, 8px) var(--spacing-md, 12px);
        border-left: 3px solid var(--color-accent, #ff6b6b);
        background: rgba(255, 107, 107, 0.05);
        font-style: italic;
      }
    </style>
  `
  
  return html
}

function getTypeClass(type: string): string {
  if (!type) return 'type-general'
  const normalized = type.toLowerCase().replace(/\s+/g, '-')
  return `type-${normalized}`
}