---
title: "UI Reference"
---

The screens of the Mimir web app and their addresses.

## Screens

| Screen | Address | Who |
|--------|---------|-----|
| [Sign-in page](home-screen/#sign-in-page) | any address, before sign-in | Everyone |
| [Campaigns (home)](home-screen/) | `/` | DM |
| [Campaign Dashboard](campaign-dashboard/) | `/campaigns/<id>`, `/campaigns/<id>/modules`, `/npcs`, `/pcs`, `/homebrew` | DM |
| [Module Page](module-page/) | `/modules/<id>` | DM |
| [DM Map](dm-map/) | `/maps/<id>` | DM |
| [Player Display](player-display/) | `/display/<campaign>` | DM (shown to players) |
| [Player Page](player-page/) | `/play/<link>`, then `/player` | A player |
| [Characters and Character Sheet](characters/) | `/characters`, `/characters/<id>` | DM |
| [Reference Page](reference-page/) | `/reference` | DM |
| [Settings](settings/) | `/settings` | DM |

Most screens keep their tab or selection in the address (`?section=`, `?doc=`, `?kind=`). A link or the back button opens the same view.

Mimir works on a desktop browser and on a tablet. The map pages take mouse, touch and pen.
