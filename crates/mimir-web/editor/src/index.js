// The rich editor of the Mimir web app (MIMIR-T-0719): TipTap, framework
// free, behind a small handle that the Leptos app drives through
// wasm-bindgen (src/editor.rs). Markdown in, markdown out.
//
// Sets window.MimirEditor = { Handle, COMMANDS }.

import { Editor, Node } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
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
        StarterKit,
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
