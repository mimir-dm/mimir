# Running Your First Session

This tutorial walks you through running a game session with the DM Map window. You'll learn how to open the battle map, place your party, control fog of war, show the map to players, and keep session notes.

**Time to complete:** 10-15 minutes

**What you'll learn:**
- Open the DM Map window for a module
- Select the active map and place PC tokens
- Control fog of war
- Open the Player Display and move tokens during play
- Keep session notes
- End the session

## Prerequisites

- A module with at least one map and some tokens ([Tutorial 2](./02-first-module.md))
- At least one PC [created](../how-to/characters/create-pc.md) and [assigned to the campaign](../how-to/characters/assign-to-campaign.md) — fog of war reveals around PC tokens, so you need a PC to see anything
- For the fog-of-war step, your map must be a UVTT file (it carries the wall data fog needs)

## Step 1: Open the DM Map Window

1. Open your campaign and go to the **Modules** tab
2. Select your module from the sidebar
3. Click the **Play** button in the module header

A separate **DM Map** window opens showing your module's battle map. Your main window stays on the campaign dashboard — keep it nearby, you'll use it for monster stats and notes during play.

## Step 2: Select Your Map

At the top left of the DM Map window is a **Map** dropdown.

1. Open the dropdown — it lists your module's maps plus any campaign-level maps (suffixed "(Campaign)")
2. Select the map you prepared in Tutorial 2 (the first module map is usually selected automatically)

The map loads in the window with its grid and the monster tokens and light sources you placed in Tutorial 2.

## Step 3: Place Your PCs

PC tokens aren't placed during prep — you add them at the table.

1. Click **Add PCs** in the map toolbar

A token appears for each PC assigned to the campaign, placed in a small formation in the top-left corner of the map. Drag each PC token to where the party enters the map.

To move any token: click and hold, drag, release. Tokens snap to grid squares.

## Step 4: Turn On Fog of War

Fog starts **off**. To hide everything the party can't see:

1. Click **Fog** in the map toolbar

On your view, areas outside the party's vision turn semi-transparent gray; players see nothing there at all. While Fog is on, **LOS** (token line of sight) is forced on too, so enemies outside the party's sight stay hidden automatically.

> **Note:** The **Fog** and **LOS** buttons only appear for UVTT maps — fog needs the wall data UVTT files carry. On a plain image map, work with **Reveal Map** and per-token visibility instead (right-click a token → **Hide from Players**).

Try the ambient light dropdown too: set it to **Dark** and watch the visible area shrink to your placed torch and the PCs' darkvision. See [Control Fog of War](../how-to/play-mode/fog-of-war.md) for the full set of controls.

## Step 5: Open the Player Display

The Player Display is a second window for your players (a TV, projector, or shared screen).

1. Click **Display** in the DM Map window's top toolbar
2. A new window opens, and the button changes to **Display On**
3. Drag the new window to your player-facing screen

Players see the current map with fog applied, visible tokens only, and no names, stats, or controls. While the display is open, a **Blackout** button appears next to **Display On** — click it to black out the player screen while you reposition tokens or set up a reveal.

## Step 6: Play

Now run the encounter:

- **Move tokens** by dragging them. The Player Display updates immediately, and fog recalculates as PC tokens move — drag a PC down a corridor and watch the visible area follow.
- **Reveal hidden enemies** by right-clicking a token and choosing **Show to Players** (or select it and press **H**).
- **Look up monster stats** in your main window: the module dashboard's monster list opens a stat block panel when you click a monster. Double-clicking a monster token in the DM Map window selects that monster in the dashboard for you.

## Step 7: Keep Session Notes

Every module comes with a **Play Notes** document. In your main window:

1. On the module dashboard, find **Play Notes** in the Documents panel
2. Click it to open the editor

Type initiative order, HP, rulings, and player decisions as you go — the editor auto-saves ("Saving..." then "Saved"). Notes persist between sessions and are included when you export the campaign.

## Step 8: End the Session

When you're done:

1. Click **Display On** to close the Player Display
2. Close the DM Map window

Token positions are saved automatically, so the next time you click **Play**, the map is exactly where you left it.

---

*Next tutorial: [Player Display Setup](./04-player-display.md)*
