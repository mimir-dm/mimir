// The rich editor of the Mimir web app (MIMIR-T-0719): TipTap, framework
// free, behind a small handle that the Leptos app drives through
// wasm-bindgen (src/editor.rs). Markdown in, markdown out.
//
// Sets window.MimirEditor = { Handle, COMMANDS }.

import { Editor, Node } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
import Paragraph from '@tiptap/extension-paragraph'
import { Lexer } from 'marked'
import Placeholder from '@tiptap/extension-placeholder'
import { Markdown } from '@tiptap/markdown'
import { Table, TableRow, TableCell, TableHeader } from '@tiptap/extension-table'
import { TaskList, TaskItem } from '@tiptap/extension-list'

// An HTML comment on its own lines (the module templates have them):
// kept as a block the writer sees but does not edit, and written back as
// it was. Other HTML goes to TipTap's own handling.
const HtmlComment = Node.create({
  name: 'htmlComment',
  group: 'block',
  atom: true,
  selectable: true,
  addAttributes() {
    return { text: { default: '' } }
  },
  parseHTML() {
    return [{ tag: 'div[data-html-comment]', getAttrs: (el) => ({ text: el.getAttribute('data-text') || '' }) }]
  },
  renderHTML({ node }) {
    return ['div', { 'data-html-comment': '', 'data-text': node.attrs.text, class: 'mimir-editor__comment' }, node.attrs.text]
  },
  markdownTokenName: 'html',
  parseMarkdown: (token) => {
    const text = (token.raw || token.text || '').trim()
    if (!token.block || !/^<!--[\s\S]*-->$/.test(text)) return null
    return { type: 'htmlComment', attrs: { text } }
  },
  renderMarkdown: (node) => node.attrs?.text ?? '',
})

// The adventure forms (MIMIR-T-0737, ADR MIMIR-A-0011): a read-aloud box
// `> [!read-aloud]`, a sidebar `> [!sidebar] Title`, and a stat block line
// `{@creature Name|Source}`. The editor shows them as boxes and writes them
// back as they were.
const CALLOUT = /^\[!(read-aloud|sidebar)\](?:[ \t]+([^\n]*))?(?:\n([\s\S]*))?$/
const STATBLOCK_TAGS =
  'creature|creatureFluff|item|spell|background|deck|object|vehicle|trap|hazard|race|feat|optfeature|reward|table'
const STATBLOCK = new RegExp(`^\\{@(${STATBLOCK_TAGS}) ([^{}|]+?)(?:\\|([^{}|]*))?\\}$`)

// @tiptap/markdown renders a node with the first handler registered for
// the markdown token of its name, so the callout, which reads `blockquote`
// tokens before the stock node, also renders the stock blockquote: this is
// the stock renderer. (The callout's priority stays under the paragraph's:
// ProseMirror fills an empty document with the first block type.)
function renderBlockquote(node, h) {
  if (!node.content) return ''
  const result = []
  node.content.forEach((child, index) => {
    const md = h.renderChild?.(child, index) ?? h.renderChildren([child])
    result.push(
      md
        .split('\n')
        .map((line) => (line.trim() === '' ? '>' : `> ${line}`))
        .join('\n'),
    )
  })
  return result.join('\n>\n')
}

// The stock paragraph renderer; a paragraph that is only a stat block tag
// but is text is written with an escaped brace, so it reads back as text.
function renderParagraph(node, h, ctx) {
  const content = Array.isArray(node.content) ? node.content : []
  if (content.length === 0) {
    const prev = ctx?.previousNode
    const prevEmpty = prev?.type === 'paragraph' && (!Array.isArray(prev.content) || prev.content.length === 0)
    return prevEmpty ? '&nbsp;' : ''
  }
  const out = h.renderChildren(content)
  return STATBLOCK.test(out.trim()) ? `\\${out.trimStart()}` : out
}

// The paragraph: one that is only a stat block tag is a stat block line
// (`\{@…}`, with an escaped brace, is text), and the escape of
// `renderParagraph` on the way out.
const MimirParagraph = Paragraph.extend({
  parseMarkdown(token, h) {
    const m = STATBLOCK.exec((token.raw || '').trim())
    if (m) return h.createNode('statblockLine', { tag: m[1], name: m[2].trim(), source: (m[3] || '').trim() })
    return this.parent?.(token, h)
  },
  renderMarkdown: (node, h, ctx) => renderParagraph(node, h, ctx),
})

// The title of a sidebar: the first child of its box.
const CalloutTitle = Node.create({
  name: 'calloutTitle',
  content: 'inline*',
  defining: true,
  parseHTML() {
    return [{ tag: 'div[data-callout-title]' }]
  },
  renderHTML() {
    return ['div', { 'data-callout-title': '', class: 'mimir-callout__title' }, 0]
  },
})

const Callout = Node.create({
  name: 'callout',
  // Before the stock blockquote (100), after the paragraph (1000).
  priority: 999,
  group: 'block',
  content: 'calloutTitle? block+',
  defining: true,
  addAttributes() {
    return { kind: { default: 'read-aloud' } }
  },
  parseHTML() {
    return [{ tag: 'aside[data-callout]', getAttrs: (el) => ({ kind: el.getAttribute('data-callout') || 'read-aloud' }) }]
  },
  renderHTML({ node }) {
    return ['aside', { 'data-callout': node.attrs.kind, class: `mimir-callout mimir-callout--${node.attrs.kind}` }, 0]
  },
  markdownTokenName: 'blockquote',
  parseMarkdown: (token, h) => {
    const first = token.tokens?.[0]
    const m = first?.type === 'paragraph' ? CALLOUT.exec(first.text) : null
    if (!m) return null
    const [, kind, title, rest] = m
    const blocks = token.tokens.slice(1)
    if (rest && rest.trim()) {
      blocks.unshift({ type: 'paragraph', raw: rest, text: rest, tokens: Lexer.lexInline(rest) })
    }
    const parseBlocks = h.parseBlockChildren ?? h.parseChildren
    const children = parseBlocks(blocks)
    const content = []
    if (title && title.trim()) content.push(h.createNode('calloutTitle', undefined, h.parseInline(Lexer.lexInline(title.trim()))))
    content.push(...(children.length ? children : [h.createNode('paragraph', undefined, [])]))
    return h.createNode('callout', { kind }, content)
  },
  renderMarkdown: (node, h) => {
    if (node.type !== 'callout') return renderBlockquote(node, h)
    const kids = node.content || []
    let head = `> [!${node.attrs?.kind || 'read-aloud'}]`
    let body = kids
    if (kids[0]?.type === 'calloutTitle') {
      const title = h.renderChildren(kids[0].content || []).trim()
      if (title) head += ` ${title}`
      body = kids.slice(1)
    }
    const lines = [head]
    body.forEach((child) => {
      const md = h.renderChildren([child])
      if (!md.trim()) return
      lines.push('>')
      md.split('\n').forEach((l) => lines.push(l.trim() === '' ? '>' : `> ${l}`))
    })
    return lines.join('\n')
  },
})

// A stat block line: an atom the writer sees as a box with the name.
const StatblockLine = Node.create({
  name: 'statblockLine',
  group: 'block',
  atom: true,
  selectable: true,
  addAttributes() {
    return { tag: { default: 'creature' }, name: { default: '' }, source: { default: '' } }
  },
  parseHTML() {
    return [
      {
        tag: 'div[data-statblock]',
        getAttrs: (el) => ({
          tag: el.getAttribute('data-statblock') || 'creature',
          name: el.getAttribute('data-name') || '',
          source: el.getAttribute('data-source') || '',
        }),
      },
    ]
  },
  renderHTML({ node }) {
    const { tag, name, source } = node.attrs
    const what = tag === 'creatureFluff' ? 'Lore' : 'Stat block'
    return [
      'div',
      { 'data-statblock': tag, 'data-name': name, 'data-source': source, class: 'mimir-editor__statblock' },
      `${what}: ${name}${source ? ` (${source})` : ''}`,
    ]
  },
  // Read by the paragraph (MimirParagraph): a paragraph that is only the tag.
  renderMarkdown: (node) => {
    const { tag, name, source } = node.attrs || {}
    return `{@${tag} ${name}${source ? `|${source}` : ''}}`
  },
})

// The commands the toolbar may run: name -> (chain, args) -> chain.
const COMMANDS = {
  heading: (c, a) => c.toggleHeading({ level: a?.level ?? 1 }),
  bold: (c) => c.toggleBold(),
  italic: (c) => c.toggleItalic(),
  strike: (c) => c.toggleStrike(),
  bulletList: (c) => c.toggleBulletList(),
  orderedList: (c) => c.toggleOrderedList(),
  blockquote: (c) => c.toggleBlockquote(),
  insertTable: (c) => c.insertTable({ rows: 3, cols: 3, withHeaderRow: true }),
  deleteTable: (c) => c.deleteTable(),
  addColumn: (c) => c.addColumnAfter(),
  deleteColumn: (c) => c.deleteColumn(),
  addRow: (c) => c.addRowAfter(),
  deleteRow: (c) => c.deleteRow(),
  rule: (c) => c.setHorizontalRule(),
  readAloud: (c) => c.insertContent({ type: 'callout', attrs: { kind: 'read-aloud' }, content: [{ type: 'paragraph' }] }),
  sidebar: (c) =>
    c.insertContent({
      type: 'callout',
      attrs: { kind: 'sidebar' },
      content: [{ type: 'calloutTitle', content: [{ type: 'text', text: 'Sidebar' }] }, { type: 'paragraph' }],
    }),
  undo: (c) => c.undo(),
  redo: (c) => c.redo(),
}

// The node or mark a command shows as active, with its attributes.
const ACTIVE = {
  heading: (a) => ['heading', { level: a?.level ?? 1 }],
  bold: () => ['bold'],
  italic: () => ['italic'],
  strike: () => ['strike'],
  bulletList: () => ['bulletList'],
  orderedList: () => ['orderedList'],
  blockquote: () => ['blockquote'],
  readAloud: () => ['callout', { kind: 'read-aloud' }],
  sidebar: () => ['callout', { kind: 'sidebar' }],
}

// The serializer writes a text "&" as "&amp;". It renders the same, but
// the stored markdown should read as typed: "&" again, outside code, when
// it does not start an entity. No blank lines at the end.
const AMP = /&amp;(?![A-Za-z][A-Za-z0-9]*;|#[0-9]+;|#[xX][0-9A-Fa-f]+;)/g

export function cleanMarkdown(md) {
  let fenced = false
  return md
    .split('\n')
    .map((line) => {
      if (/^\s*(```|~~~)/.test(line)) {
        fenced = !fenced
        return line
      }
      if (fenced) return line
      // Odd parts are inline code.
      return line
        .split('`')
        .map((part, i) => (i % 2 === 0 ? part.replace(AMP, '&') : part))
        .join('`')
    })
    .join('\n')
    .replace(/\s+$/, '')
}

function parseArgs(json) {
  if (!json) return undefined
  try {
    return JSON.parse(json)
  } catch {
    return undefined
  }
}

export class Handle {
  /**
   * @param {HTMLElement} el where the editor mounts
   * @param {string} markdown the first content
   * @param {string} placeholder shown when empty
   * @param {string} label the accessible name of the editing area
   * @param {(markdown: string) => void} onChange after each change of the content
   * @param {() => void} onState after each transaction (selection, marks)
   */
  constructor(el, markdown, placeholder, label, onChange, onState) {
    this.editor = new Editor({
      element: el,
      extensions: [
        StarterKit.configure({ paragraph: false }),
        MimirParagraph,
        CalloutTitle,
        Callout,
        StatblockLine,
        Placeholder.configure({ placeholder: placeholder || '' }),
        Markdown,
        Table.configure({ resizable: false }),
        TableRow,
        TableCell,
        TableHeader,
        TaskList,
        TaskItem.configure({ nested: true }),
        HtmlComment,
      ],
      content: markdown || '',
      contentType: 'markdown',
      editorProps: {
        attributes: { role: 'textbox', 'aria-multiline': 'true', 'aria-label': label || 'Editor' },
      },
      onUpdate: () => onChange(this.getMarkdown()),
      onTransaction: () => onState(),
    })
  }

  getMarkdown() {
    return cleanMarkdown(this.editor.getMarkdown())
  }

  /** The document as JSON (the round-trip test compares structure). */
  getJson() {
    return JSON.stringify(this.editor.getJSON())
  }

  /** Replace the content; no change event. */
  setMarkdown(markdown) {
    this.editor.commands.setContent(markdown || '', { contentType: 'markdown', emitUpdate: false })
  }

  /** Run a toolbar command; false when unknown or not possible. */
  run(name, argsJson) {
    const f = COMMANDS[name]
    if (!f) return false
    return f(this.editor.chain().focus(), parseArgs(argsJson)).run()
  }

  /** Whether a command can run now. */
  can(name, argsJson) {
    const f = COMMANDS[name]
    if (!f) return false
    return f(this.editor.can().chain(), parseArgs(argsJson)).run()
  }

  /** Whether the selection is in the node or mark of a command. */
  isActive(name, argsJson) {
    const f = ACTIVE[name]
    if (!f) return false
    const [type, attrs] = f(parseArgs(argsJson))
    return this.editor.isActive(type, attrs)
  }

  setEditable(on) {
    this.editor.setEditable(!!on)
  }

  focus() {
    this.editor.commands.focus()
  }

  destroy() {
    this.editor.destroy()
  }
}

/**
 * Markdown through the editor and back (the round-trip test): the markdown
 * out, and the document as JSON.
 */
export function roundTrip(markdown) {
  const el = document.createElement('div')
  const h = new Handle(el, markdown, '', '', () => {}, () => {})
  const out = { markdown: h.getMarkdown(), json: h.getJson() }
  h.destroy()
  return out
}

window.MimirEditor = { Handle, COMMANDS: Object.keys(COMMANDS), roundTrip, cleanMarkdown }
