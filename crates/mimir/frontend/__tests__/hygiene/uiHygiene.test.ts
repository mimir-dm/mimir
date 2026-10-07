/**
 * UI hygiene guards (COLLIERY-I-0465, UI Cohesion Pass). Each check keeps a
 * kind of drift from coming back after the sweep that removed it.
 */
import { describe, it, expect } from 'vitest'
import fs from 'node:fs'
import path from 'node:path'

const SRC = path.resolve(__dirname, '../../src')

/** Every .vue, .ts and .css file under src (tests excluded), as [relative path, text]. */
function sources(): [string, string][] {
  const out: [string, string][] = []
  const walk = (dir: string) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name)
      if (entry.isDirectory()) {
        if (entry.name !== '__tests__') walk(full)
      } else if (/\.(vue|ts|css)$/.test(entry.name) && !entry.name.endsWith('.d.ts')) {
        out.push([path.relative(SRC, full), fs.readFileSync(full, 'utf8')])
      }
    }
  }
  walk(SRC)
  return out
}

/** Lines of the files that match, as "file:line: text". */
function matches(re: RegExp, skip: (file: string) => boolean = () => false): string[] {
  const hits: string[] = []
  for (const [file, text] of sources()) {
    if (skip(file)) continue
    text.split('\n').forEach((line, i) => {
      if (re.test(line)) hits.push(`${file}:${i + 1}: ${line.trim()}`)
    })
  }
  return hits
}

describe('UI hygiene', () => {
  it('uses the app dialogs, never native confirm() or alert() (MIMIR-T-0684)', () => {
    // useDialog.ts falls back to the native dialogs only when a window has no DialogHost.
    const hits = matches(/(^|[^.\w])(window\.)?(confirm|alert)\(/, (f) => f === path.join('composables', 'useDialog.ts'))
    expect(hits, 'use useDialog().confirm / .alert instead').toEqual([])
  })

  it('has no hard-coded color fallbacks in var() (MIMIR-T-0686)', () => {
    // A fallback hides a token that no theme defines. Define the token instead.
    expect(matches(/var\(--[\w-]+\s*,\s*(#[0-9a-fA-F]{3,8}|rgba?\(|hsla?\()/)).toEqual([])
  })

  it('defines every legacy color it uses in the shared root (MIMIR-T-0686)', () => {
    const main = fs.readFileSync(path.join(SRC, 'assets/styles/main.css'), 'utf8')
    const defined = new Set([...main.matchAll(/(--legacy-[\w-]+)\s*:/g)].map((m) => m[1]))
    const used = new Set(sources().flatMap(([, text]) => [...text.matchAll(/var\((--legacy-[\w-]+)\)/g)].map((m) => m[1])))
    expect([...used].filter((t) => !defined.has(t))).toEqual([])
    // A legacy color that nothing uses any more is deleted.
    expect([...defined].filter((t) => !used.has(t))).toEqual([])
  })

  it('keeps theme differences in the theme files, not in components (MIMIR-T-0685)', () => {
    // A component that needs a different color per theme uses a token, for
    // example --color-primary-tint, that each theme file defines.
    const hits = matches(/\.theme-(light|dark|hyper)\b/, (f) => !f.endsWith('.vue'))
    expect(hits, 'use a themed token instead of a .theme-* selector').toEqual([])
  })

  it('draws UI icons with @lucide/vue, not inline SVG (MIMIR-T-0682)', () => {
    // Map layers draw their own SVG with a computed viewBox (:viewBox), so
    // only a static icon viewBox is flagged.
    const hits = matches(/<svg\b[^>]*\sviewBox="0 0 (24 24|20 20|16 16|12 12)"/, (f) => !f.endsWith('.vue'))
    expect(hits, 'import the icon from @lucide/vue').toEqual([])
  })

  it('renders empty states with EmptyState (MIMIR-T-0683)', () => {
    // EmptyState gives the icon, title, description and action slot.
    expect(matches(/empty-icon/), 'use <EmptyState variant=… title=…>').toEqual([])
  })

  it('uses only color tokens that the shared root or every theme defines (MIMIR-T-0687)', () => {
    // var() of an undefined token makes the declaration invalid: the text
    // inherits its color, the background goes transparent.
    const read = (f: string) => fs.readFileSync(path.join(SRC, 'assets/styles', f), 'utf8')
    const defs = (css: string) => new Set([...css.matchAll(/(--[\w-]+)\s*:/g)].map((m) => m[1]))
    const main = read('main.css')
    const root = defs(main.slice(main.indexOf(':root {'), main.indexOf('\n}', main.indexOf(':root {'))))
    const themes = Object.fromEntries(
      ['light', 'dark', 'hyper'].map((t) => [t, defs(read(`themes/${t}.css`))]),
    ) as Record<string, Set<string>>
    const everyTheme = (t: string) => Object.values(themes).every((d) => d.has(t))
    const undefinedUses: string[] = []
    for (const [file, text] of sources()) {
      // Inside a theme file, its own tokens count.
      const own = /themes[\\/](\w+)\.css$/.exec(file)?.[1]
      for (const m of text.matchAll(/var\((--color-[\w-]+)\s*[,)]/g)) {
        const t = m[1]
        if (root.has(t) || everyTheme(t) || (own && themes[own]?.has(t))) continue
        undefinedUses.push(`${file}: ${t}`)
      }
    }
    expect(undefinedUses).toEqual([])
  })
})
