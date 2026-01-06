# Running Your First Session

This tutorial walks you through using Play Mode to run a game session. You'll learn how to manage maps, control fog of war, move tokens, and keep session notes.

**Time to complete:** 10-15 minutes

**What you'll learn:**
- Enter and exit Play Mode
- Navigate the Play Mode interface
- Control fog of war and line of sight
- Move tokens and manage encounters
- Use session notes

## Prerequisites

- A module with at least one map and some tokens ([Tutorial 2](./02-first-module.md))
- Understanding of the Campaign Dashboard

## Step 1: Enter Play Mode

There are two ways to start a play session:

### From the Module Dashboard
1. Open your campaign and go to the **Modules** tab
2. Select your module from the sidebar
3. Click the **Play** button in the module header

### From the Maps List
1. In the module dashboard, find your map in the Maps section
2. Click the map, then use the Play action

Either method takes you to Play Mode.

## Step 2: Tour the Play Mode Interface

Play Mode has a different layout optimized for running the game:

<!-- Screenshot: play-mode.png -->

### Header Bar

- **Back to Prep** - Return to the module dashboard
- **Module Name** - Shows which module you're playing
- **PLAY MODE** badge - Visual confirmation you're in play mode
- **Player Display** - Open/close the player display window
- **Blackout** - Hide everything from players temporarily
- **End Session** - Exit play mode

### Left Sidebar

A collapsible panel containing:

**Monsters** - All monsters in the module
- Shows quantity (e.g., "3× Goblin")
- Click to view stat block
- Grouped by encounter tag

**Maps** - All maps in the module
- Click to switch the active map
- Active map shows a play icon
- Maps from other modules in the campaign are also available

### Main Area

The tactical map display with:
- Current map image
- Grid overlay
- Token positions
- Fog of war (if using UVTT maps with walls)

### Bottom Panel

**Session Notes** - Collapsible notes area
- Auto-saves as you type
- Persists between sessions
- Great for tracking HP, initiative, events

## Step 3: Understanding the Map Controls

The map toolbar (top of main area) provides essential controls:

### Zoom Controls
- **−/+** buttons - Zoom in and out
- **Percentage** - Current zoom level
- **Reset** button - Fit map to view

### Token Management
- **Add Token** - Quick-add a monster token to the map
- **Add PCs** - Place all campaign PCs on the map at once

### Reveal Map Toggle
A danger-styled toggle that reveals the entire map to players. Use with caution - this bypasses fog of war completely.

### Line of Sight Controls (UVTT maps only)
When using UVTT maps with wall data:

- **LOS Toggle** - Switch between Fog and Token modes:
  - **Fog** - Map is hidden outside PC vision areas
  - **Token** - Map visible, but enemy tokens hidden outside vision

- **Debug** button - Visualize line of sight calculations
- **Ambient Light** - Set base lighting level:
  - **Bright** - Full daylight
  - **Dim** - Twilight or torchlight
  - **Dark** - Complete darkness (darkvision only)

### Viewport Sync
- **Sync** - Auto-sync your view to the player display
- **Push View** - Manually push current view to players

## Step 4: Working with Tokens

### Moving Tokens

1. Click and hold a token
2. Drag to the new position
3. Release to place

Tokens snap to grid squares by default.

### Token Visibility

Tokens have visibility states that affect what players see:
- **Visible** - Players see the token
- **Hidden** - Only DM sees (for surprise encounters)

Right-click a token to toggle visibility.

### Interacting with Monster Tokens

Click a monster in the sidebar or on the map to open the Monster Stats Panel:
- Full stat block
- Actions and abilities
- Quick reference during combat

The panel slides in from the right without blocking the map.

## Step 5: Controlling Fog of War

Fog of war automatically calculates what players can see based on:
- PC token positions
- Vision radius (based on darkvision)
- Light sources (torches, lanterns)
- Wall obstructions (UVTT maps only)

### How Vision Works

1. **PC tokens reveal fog** - Only PCs with "visible to players" reveal areas
2. **NPCs don't reveal** - NPC tokens don't create vision
3. **Light sources extend vision** - Placed light sources affect visibility

### Lighting Controls

The ambient light dropdown affects base visibility:
- **Bright** - Everything visible within range
- **Dim** - Disadvantage on Perception, colors muted
- **Dark** - Only darkvision and light sources work

Toggle light sources on/off by right-clicking them on the map or using the player display controls.

## Step 6: Using the Player Display

The Player Display shows a separate view for your players (on a TV, projector, or shared screen).

### Opening Player Display

1. Click **Player Display** in the header
2. A new window opens (drag to your player-facing screen)
3. The button changes to "Display Open"

### What Players See

- Current map with fog of war applied
- Visible tokens only
- No monster names or stats
- Revealed areas based on PC vision

### Blackout Mode

Click the **Blackout** button (eye icon) to:
- Hide everything from players
- Show a black screen
- Useful for dramatic reveals or bathroom breaks

Click again to restore the view.

### Syncing Views

By default, auto-sync is enabled:
- When you pan/zoom, players see the same view
- Keeps everyone focused on the action

Disable sync for independent viewing, then use **Push View** to manually update the player display.

## Step 7: Taking Session Notes

The collapsible notes panel at the bottom is perfect for:
- Initiative order
- HP tracking
- Important events
- NPC dialogue
- Player decisions

### Using Notes

1. Click **Session Notes** bar to expand
2. Type your notes
3. Notes auto-save as you type
4. Status shows "Saving..." then "Saved"

Notes persist between sessions - they're saved to your campaign folder.

## Step 8: Ending the Session

When you're done:

1. Click **End Session** in the header
2. You return to the module dashboard
3. Session notes are saved
4. Token positions are preserved

Your module is ready for the next session right where you left off.

## Pro Tips

### Combat Efficiency
- Open monster stats before combat starts
- Use session notes for initiative tracking
- Pre-position tokens for surprise encounters

### Fog of War
- Place light sources at choke points
- Use darkness to create tension
- Toggle lights for dramatic effect

### Player Display
- Use Blackout for reveals
- Sync view for guided exploration
- Push View for "cinematic" shots

---

## Quick Reference

| Action | How To |
|--------|--------|
| Enter Play Mode | Module dashboard → Play button |
| Exit Play Mode | End Session button |
| Switch maps | Click map in sidebar |
| Move token | Drag and drop |
| View monster stats | Click monster in sidebar |
| Toggle fog mode | LOS toggle (Fog/Token) |
| Set ambient light | Light dropdown |
| Open player display | Player Display button |
| Blackout display | Eye icon (when display open) |
| Take notes | Session Notes panel |

---

*Next tutorial: [Player Display Setup](./04-player-display.md)*
