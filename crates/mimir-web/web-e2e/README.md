# mimir-web e2e

Playwright tests and screenshot captures for the web app.

Playwright starts `session.sh`. The script starts mimir-server on a new
scratch data directory with the UI fixture (the SRD catalog and "The Lost
Mine of Phandelver"). The server serves the built app (`../dist`) and asks
for a token (`e2e-dm-token`, a test value). The script removes the scratch
directory when the server stops. It does not use a real data directory.

## Commands

From the repository root:

```
angreal web e2e                                   # smoke suite (CI runs it)
angreal web screenshots                           # captures: light and dark
angreal web screenshots --themes light,dark,hyper
angreal web screenshots-diff --before <run>       # compare with the latest run
```

The captures go to `screenshots/runs/<timestamp>/`, one PNG for each screen,
size (desktop 1280×800, tablet 820×1100) and theme. `screenshots-diff` writes
diff images to `<after>/diff`.

## Specs

- `specs/smoke.spec.ts`: sign-in, the campaign list, the dashboard tabs, the
  module page, an unknown address.
- `specs/map.spec.ts`, `display.spec.ts`, `combat.spec.ts`: the DM map, the
  player display, combat.
- `specs/players.spec.ts`, `characters.spec.ts`, `party.spec.ts`: player
  links, character sheets, player notes and party sharing.
- `specs/authoring.spec.ts`: campaigns, documents (autosave), modules,
  module monsters, map uploads.
- `specs/characters-dm.spec.ts`: the character wizard (PC, NPC), the
  characters page (move, delete), NPC fields on the sheet.
- `specs/reference.spec.ts`: the catalog, cross-references, the book reader,
  the module inspector.
- `specs/homebrew.spec.ts`: homebrew items (form and JSON), monsters
  (copied, used by a module, deleted), spells.
- `specs/editor.spec.ts`: the rich editor. The fixture documents keep their
  structure and words through it; the toolbar; a numbered list continues.
- `specs/capture.spec.ts`: the screens to capture.
- `specs/session.ts`: the token, the fixture names, and `test`. Each test
  fails on a page error or a console error (except the browser's line for a
  401 or 404 answer).
