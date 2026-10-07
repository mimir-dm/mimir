/**
 * UI hygiene guards (COLLIERY-I-0465, UI Cohesion Pass). Each check keeps a
 * kind of drift from coming back after the sweep that removed it.
 */
import { describe, it, expect } from 'vitest'
import fs from 'node:fs'
import path from 'node:path'

const SRC = path.resolve(__dirname, '../../src')

/** Every .vue and .ts file under src (tests excluded), as [relative path, text]. */
function sources(): [string, string][] {
  const out: [string, string][] = []
  const walk = (dir: string) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name)
      if (entry.isDirectory()) {
        if (entry.name !== '__tests__') walk(full)
      } else if (/\.(vue|ts)$/.test(entry.name) && !entry.name.endsWith('.d.ts')) {
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
})
