#!/usr/bin/env node
/**
 * Compare two screenshot runs of the UI harness, image by image.
 *
 *   node playwright/diff.mjs <before-dir> <after-dir> [--out <dir>] [--fail]
 *
 * For each PNG in either run it prints the number of changed pixels (pixelmatch,
 * anti-aliasing ignored) and writes a diff image for each changed one to
 * --out (default: <after-dir>/diff). Images of different sizes are compared
 * over the common area and reported with both sizes. --fail exits 1 when an
 * image changed, is new or is missing.
 */
import fs from 'node:fs'
import path from 'node:path'
import { PNG } from 'pngjs'
import pixelmatch from 'pixelmatch'

const args = process.argv.slice(2)
const flag = (name) => {
  const i = args.indexOf(name)
  if (i === -1) return undefined
  const [, value] = args.splice(i, 2)
  return value
}
const fail = args.includes('--fail')
if (fail) args.splice(args.indexOf('--fail'), 1)
const outArg = flag('--out')
const [beforeDir, afterDir] = args
if (!beforeDir || !afterDir) {
  console.error('usage: node playwright/diff.mjs <before-dir> <after-dir> [--out <dir>] [--fail]')
  process.exit(2)
}
const outDir = outArg ?? path.join(afterDir, 'diff')

const pngs = (dir) => new Set(fs.readdirSync(dir).filter((f) => f.endsWith('.png')))
const before = pngs(beforeDir)
const after = pngs(afterDir)
const names = [...new Set([...before, ...after])].sort()

/** Copy the top-left w×h area of an image. */
function crop(img, w, h) {
  if (img.width === w && img.height === h) return img.data
  const out = Buffer.alloc(w * h * 4)
  for (let y = 0; y < h; y++) {
    img.data.copy(out, y * w * 4, y * img.width * 4, y * img.width * 4 + w * 4)
  }
  return out
}

const rows = []
for (const name of names) {
  if (!before.has(name)) {
    rows.push({ name, status: 'new', changed: '' })
    continue
  }
  if (!after.has(name)) {
    rows.push({ name, status: 'missing', changed: '' })
    continue
  }
  const a = PNG.sync.read(fs.readFileSync(path.join(beforeDir, name)))
  const b = PNG.sync.read(fs.readFileSync(path.join(afterDir, name)))
  const w = Math.min(a.width, b.width)
  const h = Math.min(a.height, b.height)
  const diff = new PNG({ width: w, height: h })
  const changed = pixelmatch(crop(a, w, h), crop(b, w, h), diff.data, w, h, { threshold: 0.1 })
  const resized = a.width !== b.width || a.height !== b.height
  if (changed === 0 && !resized) {
    rows.push({ name, status: 'same', changed: 0 })
    continue
  }
  fs.mkdirSync(outDir, { recursive: true })
  fs.writeFileSync(path.join(outDir, name), PNG.sync.write(diff))
  const size = resized ? ` (${a.width}x${a.height} -> ${b.width}x${b.height})` : ''
  rows.push({ name, status: `changed${size}`, changed, pct: ((changed / (w * h)) * 100).toFixed(3) })
}

const width = Math.max(...rows.map((r) => r.name.length), 10)
for (const r of rows) {
  const detail = r.status.startsWith('changed') ? `${r.changed} px (${r.pct}%) ${r.status}` : r.status
  console.log(`${r.name.padEnd(width)}  ${detail}`)
}
const counts = rows.reduce((c, r) => ((c[r.status.split(' ')[0]] = (c[r.status.split(' ')[0]] ?? 0) + 1), c), {})
console.log(`\n${names.length} images: ${Object.entries(counts).map(([k, v]) => `${v} ${k}`).join(', ')}`)
if (rows.some((r) => r.status !== 'same')) console.log(`diff images: ${outDir}`)
process.exit(fail && rows.some((r) => r.status !== 'same') ? 1 : 0)
