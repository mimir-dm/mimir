# UI Reference

The screens of the Mimir web app and their addresses.

## Screens

| Screen | Address | Who |
|--------|---------|-----|
| [Sign-in page](./home-screen.md#sign-in-page) | any address, before sign-in | Everyone |
| [Campaigns (home)](./home-screen.md) | `/` | DM |
| [Campaign Dashboard](./campaign-dashboard.md) | `/campaigns/<id>`, `/campaigns/<id>/modules`, `/npcs`, `/pcs`, `/homebrew` | DM |
| [Module Page](./module-page.md) | `/modules/<id>` | DM |
| [DM Map](./dm-map.md) | `/maps/<id>` | DM |
| [Player Display](./player-display.md) | `/display/<campaign>` | DM (shown to players) |
| [Player Page](./player-page.md) | `/play/<link>`, then `/player` | A player |
| [Characters and Character Sheet](./characters.md) | `/characters`, `/characters/<id>` | DM |
| [Reference Page](./reference-page.md) | `/reference` | DM |
| [Settings](./settings.md) | `/settings` | DM |

Most screens keep their tab or selection in the address (`?section=`, `?doc=`, `?kind=`). A link or the back button opens the same view.

Mimir works on a desktop browser and on a tablet. The map pages take mouse, touch and pen.
