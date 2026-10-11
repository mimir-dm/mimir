# mimir-web editor bundle

The rich editor of the web app (MIMIR-T-0719): TipTap 3 without a framework.
`src/index.js` sets `window.MimirEditor`. The Leptos app drives it through
wasm-bindgen (`../src/editor.rs`). The content is markdown.

## Build

Trunk runs `build.sh` before each build (`../Trunk.toml`). The script runs
`npm ci` when `node_modules` is missing or older than the lockfile, then
esbuild. The output is `dist/mimir-editor.js`. `index.html` loads it before
the app. Node must be on PATH.

## Markdown

TipTap does not write the same bytes that it reads. It pads tables, escapes
brackets and puts blank lines around blocks. The bundle corrects these items:

- `&amp;` becomes `&` again, outside code.
- Task lists (`- [ ]`) stay.
- An HTML comment on its own lines stays as it is.
- There are no blank lines at the end.

`../web-e2e/specs/editor.spec.ts` sends the fixture documents through the
editor. The structure and the words must not change, and a second save must
give the same markdown.
