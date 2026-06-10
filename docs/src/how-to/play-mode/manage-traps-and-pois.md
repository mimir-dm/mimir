# Manage Traps and POIs

Place trap markers and points of interest (POIs) on a map, control what players see, and trigger traps during play.

Traps and POIs are placed in the **Token Setup modal** and managed during play from the DM Map window's map viewer. Some operations are only available in Token Setup — each section below says where it works.

## Place a Trap

1. Open Token Setup: click the map card (or its **Place Tokens** button) in the Module Prep View or the dashboard Modules tab.
2. In the token palette, select **Trap** (⚠️).
3. Optionally use **Link Trap** to search the catalog and link a published trap — the marker takes the trap's name, and its full description can be pulled up in the trap details panel during play.
4. Set the name (if not linked) and choose whether the marker starts **Visible** to players (off by default — traps usually start hidden).
5. Click a grid square on the map to place the marker.

Drag the marker to reposition it (Token Setup only).

> **Note:** A trap record also stores a detection DC and trigger/effect text. The current UI does not provide a form for editing these fields directly; link a catalog trap to give the marker a name that resolves to full trap mechanics in the play-mode trap details panel.

## Hide or Reveal a Trap

Hidden traps appear gray and are never sent to the Player Display. Visible traps show a dashed green ring and appear on the Player Display.

- **Token Setup:** right-click the marker and choose **Show to Players** / **Hide from Players**, or use the **Vis/Hid** button next to the trap in the placed-content list.
- **Play Mode / DM Map window:** right-click the trap marker to toggle its visibility directly.

Reveal a trap when the party detects it; keep it hidden until then.

## Trigger and Reset a Trap

Available in **Token Setup only**: right-click the trap marker and choose **Trigger Trap**. The marker is shown in its triggered state and the placed-content list shows "(Triggered)". To re-arm it, right-click again and choose the reset option.

## Place a POI

1. Open Token Setup for the map.
2. In the token palette, select **Marker** (📍).
3. Set a name and color, and choose initial player visibility.
4. Click a grid square to place it.

New POIs use the pin icon. To change the icon or add a description, edit the POI (next section).

## Edit a POI (Icon, Color, Description)

Open the POI edit form from either view:

- **Token Setup:** click the **Edit** button next to the POI in the placed-content list, or right-click the marker and choose **Edit...**
- **Play Mode / DM Map window:** right-click the POI marker and choose **Edit...**

The form lets you set the name, a DM-notes description, one of eight icons (pin, star, skull, chest, door, secret, question, exclamation), and a color.

## Hide or Reveal a POI

- **Token Setup:** use the **Vis/Hid** button in the placed-content list, or right-click the marker.
- **Play Mode / DM Map window:** right-click the POI marker and choose **Show to Players** / **Hide from Players**.

Visible POIs are sent to the Player Display with their icon and color.

## Delete a Trap or POI

Right-click the marker and choose **Delete** (POI deletion is available in both views; trap deletion in Token Setup), or use the **×** button in the Token Setup placed-content list. Deletion asks for confirmation and is permanent.

## During Play

- Double-clicking a trap or POI in the DM Map window focuses its details in the campaign dashboard — for catalog-linked traps this shows the full trap mechanics.
- Marker visibility changes propagate to the Player Display immediately while it is open.

## See Also

- [Fog of War](./fog-of-war.md) — Controlling map visibility around your markers
- [Token Setup Modal](../../reference/ui/token-setup-modal.md) — Full reference for the placement UI
- [Play Mode](../../reference/ui/play-mode.md) — Session interface reference
