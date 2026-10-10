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

  it('uses no legacy colors; reader content follows the theme (MIMIR-T-0697)', () => {
    // The --legacy-* colors of MIMIR-T-0686 are mapped onto themed tokens.
    expect(matches(/--legacy-/)).toEqual([])
  })

  it('keeps theme differences in the theme files, not in components (MIMIR-T-0685)', () => {
    // A component that needs a different color per theme uses a token, for
    // example --color-primary-tint, that each theme file defines.
    // Only the theme files may select on a theme class (components and the
    // shared stylesheets alike).
    const hits = matches(/\.theme-(light|dark|hyper)\b/, (f) => f.startsWith(path.join('assets', 'styles', 'themes')))
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

  it('reveals hover-only controls on keyboard focus too (MIMIR-T-0691)', () => {
    // A control that appears only when the pointer is over its row is out of
    // reach for the keyboard. Pair `.row:hover .ctl` with `.row:focus-within .ctl`.
    // Not controls: the player display's overlays, the header's icon tint.
    const exempt = new Set([path.join('components', 'PlayerDisplayWindow.vue'), path.join('app', 'AppHeader.vue')])
    const hits: string[] = []
    for (const [file, text] of sources()) {
      if (!file.endsWith('.vue') || exempt.has(file)) continue
      for (const m of text.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
        const selectors = m[1].split(',').map((x) => x.trim())
        const reveals = /opacity:\s*1\b|display:\s*(flex|block|inline-flex|inline-block)/.test(m[2])
        if (reveals && selectors.some((x) => /:hover\s+\S/.test(x)) && !selectors.some((x) => x.includes(':focus-within'))) {
          hits.push(`${file}: ${selectors.join(', ')}`)
        }
      }
    }
    expect(hits, 'add the :focus-within selector').toEqual([])
  })

  it('has no emoji or symbol entities used as icons in templates (MIMIR-T-0682)', () => {
    // &#9650; (▲), &#10003; (✓), &#128196; (📄) and the like: use the Lucide icon.
    const hits = matches(/&#(9[6-9]\d\d|1\d{4,5});/, (f) => !f.endsWith('.vue'))
    expect(hits, 'import the icon from @lucide/vue').toEqual([])
  })

  it('loads no fonts from the network (MIMIR-T-0694)', () => {
    // The desktop app can be offline; fonts are bundled (@fontsource).
    expect(matches(/fonts\.(googleapis|gstatic)\.com/)).toEqual([])
  })

  it('draws focus halos with --color-focus-ring (MIMIR-T-0696)', () => {
    // A halo is box-shadow: 0 0 0 Npx <light or translucent primary>; each theme tunes the token.
    const halo = /box-shadow\s*:\s*0 0 0 \d+px (var\(--color-primary-(50|100|200|tint)\)|color-mix\(in srgb, var\(--color-primary)/
    expect(matches(halo, (f) => f.startsWith(path.join('assets', 'styles', 'themes')))).toEqual([])
  })

  it('colors reader content with theme tokens, not hex (MIMIR-T-0697)', () => {
    // The formatters write inline styles; a hex color there ignores the theme.
    const hits = matches(/(color|border[\w-]*)\s*:[^;'"`]*#[0-9a-fA-F]{3,8}\b/, (f) => !f.startsWith(path.join('features', 'sources', 'formatters')))
    expect(hits, 'use a --color-dnd-* or text token').toEqual([])
  })
})
