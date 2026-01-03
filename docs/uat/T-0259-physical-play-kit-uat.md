# UAT Script: Physical Play Kit PDF Generation (T-0259)

## Prerequisites

1. Running Mimir application (`npm run tauri dev`)
2. A campaign with at least one module containing maps
3. At least one UVTT map imported to a module (with grid data)
4. Some tokens placed on module maps (for cutout testing)
5. At least one campaign-level map (regional/world map)

---

## Export Dialog Structure

The Campaign Export dialog has the following structure:

**Reference Document**
- [ ] Campaign Documents
- [ ] Module Content
  - *Module Maps* (appears when Module Content checked)
    - [ ] Map Previews
    - [ ] Tiled Maps
      - [ ] Token Cutouts (enabled when Tiled Maps checked)
- [ ] NPCs

**Campaign Maps**
- [ ] Map Previews
- [ ] Tiled Maps

---

## Test Case 1: Campaign Tiled Maps Only

**Objective:** Verify campaign (regional) tiled maps generate correctly

### Steps
1. Navigate to the Campaign view
2. Click **Export** to open the Campaign Export dialog
3. **Uncheck all** Reference Document options
4. Under **Campaign Maps**:
   - Uncheck "Map Previews"
   - **Check** "Tiled Maps"
5. Click **Export PDF**
6. Verify PDF contains:
   - [ ] "Physical Play Kit" section header page
   - [ ] Assembly Guide page for each campaign map showing:
     - [ ] Map name
     - [ ] Grid of tile labels (A1, A2, B1, B2, etc.)
     - [ ] Total tile count
     - [ ] Assembly instructions
   - [ ] Individual tile pages with:
     - [ ] Tile label in header (e.g., "Map Name - Tile A1")
     - [ ] Neighbor indicators (↑B1, ←A2, etc.)
     - [ ] Actual map tile image at 9x6 grid squares
     - [ ] All content on single page (no page breaks within tile)

**Expected Result:** PDF generates with only campaign tiled maps, no token cutouts (campaign maps don't have tokens)

---

## Test Case 2: Module Tiled Maps with Token Cutouts

**Objective:** Verify module maps with token cutouts generate correctly

### Steps
1. Open Campaign Export dialog
2. **Check** "Module Content"
3. In the nested **Module Maps** section:
   - Check "Tiled Maps"
   - Check "Token Cutouts"
4. **Uncheck** all other options (Campaign Documents, NPCs, Campaign Maps)
5. Click **Export PDF**
6. Verify PDF contains:
   - [ ] Physical Play Kit section
   - [ ] Tiled maps for module dungeon/encounter maps
   - [ ] After each map's tiles, a "Token Cutouts" page with:
     - [ ] Tokens grouped by size (Tiny, Small/Medium, Large, Huge, Gargantuan)
     - [ ] Circular standee designs with front/back
     - [ ] Dashed fold lines
     - [ ] Assembly instructions

**Expected Result:** PDF includes module tiled maps with token cutout sheets

---

## Test Case 3: Full Campaign Export

**Objective:** Verify combined export with all options produces correct structure

### Steps
1. Open Campaign Export dialog
2. **Check all options:**
   - [x] Campaign Documents
   - [x] Module Content
     - [x] Map Previews (module)
     - [x] Tiled Maps (module)
       - [x] Token Cutouts
   - [x] NPCs
   - [x] Campaign Map Previews
   - [x] Campaign Tiled Maps
3. Click **Export PDF**
4. Verify PDF structure:
   - [ ] Title page with campaign name
   - [ ] Campaign documents section
   - [ ] Module sections with documents and monsters
   - [ ] NPC sheets (if NPCs exist)
   - [ ] Map Previews (both campaign and module maps)
   - [ ] **Then** Physical Play Kit section:
     - [ ] Section header
     - [ ] Campaign maps: Assembly → Tiles (no cutouts)
     - [ ] Module maps: Assembly → Tiles → Cutouts

**Expected Result:** Reference content appears first, Play Kit appears at the end with both map types

---

## Test Case 4: Module Maps Only Appear When Module Content Checked

**Objective:** Verify Module Maps options visibility

### Steps
1. Open Campaign Export dialog
2. **Uncheck** "Module Content"
3. Verify:
   - [ ] Module Maps section is hidden
   - [ ] No Map Previews, Tiled Maps, or Token Cutouts options for modules visible
4. **Check** "Module Content"
5. Verify:
   - [ ] Module Maps section appears with left border indicator
   - [ ] Map Previews option visible
   - [ ] Tiled Maps option visible
   - [ ] Token Cutouts visible but disabled

**Expected Result:** Module Maps options only appear when Module Content is selected

---

## Test Case 5: Token Cutouts Requires Tiled Maps

**Objective:** Verify Token Cutouts is dependent on Tiled Maps

### Steps
1. Open Campaign Export dialog
2. Check "Module Content"
3. Verify Token Cutouts checkbox is disabled (grayed out)
4. Check "Tiled Maps" under Module Maps
5. Verify Token Cutouts checkbox becomes enabled
6. Check "Token Cutouts"
7. Uncheck "Tiled Maps"
8. Verify:
   - [ ] Token Cutouts automatically unchecked
   - [ ] Token Cutouts disabled again

**Expected Result:** Token Cutouts can only be selected when Tiled Maps is checked

---

## Test Case 6: Export with No Maps

**Objective:** Verify graceful handling when no maps exist

### Steps
1. Select a campaign with no maps (or a module with no maps)
2. Open Export dialog
3. Check "Campaign Tiled Maps" option
4. Click **Export PDF**
5. Verify:
   - [ ] PDF generates successfully
   - [ ] Physical Play Kit section is omitted (no empty section)
   - [ ] Other selected content still appears

**Expected Result:** No errors, Play Kit section simply absent when no maps

---

## Test Case 7: Map Without Grid Data

**Objective:** Verify maps without grid info are skipped for tiling

### Steps
1. Import a regular image (not UVTT) as a map
2. Export with "Tiled Maps" checked
3. Check application logs
4. Verify:
   - [ ] Log message: "Skipping map 'X' for tiling - no grid size"
   - [ ] Other maps with grid data are still tiled
   - [ ] PDF generates without error

**Expected Result:** Maps without grid_size_px are gracefully skipped

---

## Test Case 8: Large Map Tiling

**Objective:** Verify large maps produce correct number of tiles

### Steps
1. Use a large map (e.g., 50x50+ grid squares)
2. Export with "Tiled Maps" checked
3. Verify:
   - [ ] Assembly guide shows correct grid (e.g., 6x9 for 50x50 map)
   - [ ] All tiles are present and labeled correctly
   - [ ] Tiles at edges may be partial (smaller than 9x6)
   - [ ] Neighbor indicators are correct at boundaries
   - [ ] Mostly blank tiles are skipped (< 5% content)

**Expected Result:** Large maps correctly decomposed into multiple tiles

---

## Test Case 9: Validation - No Selection Warning

**Objective:** Verify export requires at least one option

### Steps
1. Open Export dialog
2. **Uncheck all** options
3. Verify:
   - [ ] Warning message appears: "Select at least one option to export"
   - [ ] Export button is disabled

**Expected Result:** Cannot export with nothing selected

---

## Test Case 10: Loading State

**Objective:** Verify proper loading feedback during generation

### Steps
1. Open Export dialog with tiled maps on a campaign with multiple large maps
2. Click **Export PDF**
3. Verify:
   - [ ] Button shows "Generating..." with spinner
   - [ ] Dialog cannot be closed during generation
   - [ ] Preview modal appears with loading state
   - [ ] PDF eventually loads in preview

**Expected Result:** Clear loading feedback throughout generation

---

## Test Case 11: Download and Print Verification

**Objective:** Verify physical output matches expected scale

### Steps
1. Export a map with "Tiled Maps" checked
2. Download the PDF
3. Print one tile page at 100% scale (no scaling)
4. Measure the grid squares with a ruler
5. Verify:
   - [ ] Each grid square measures approximately 1 inch
   - [ ] 9 squares fit horizontally on letter paper
   - [ ] 6 squares fit vertically on letter paper

**Expected Result:** Tiles print at true 1"=5ft scale

---

## Bug Reporting Template

If any test fails, document:

```
**Test Case #:**
**Steps to Reproduce:**
**Expected Result:**
**Actual Result:**
**Screenshots/Logs:**
**Severity:** (Blocker/Critical/Major/Minor)
```

---

## Sign-Off

| Test Case | Pass/Fail | Tester | Date | Notes |
|-----------|-----------|--------|------|-------|
| TC-1 | | | | |
| TC-2 | | | | |
| TC-3 | | | | |
| TC-4 | | | | |
| TC-5 | | | | |
| TC-6 | | | | |
| TC-7 | | | | |
| TC-8 | | | | |
| TC-9 | | | | |
| TC-10 | | | | |
| TC-11 | | | | |

**Overall Status:** ____________

**Approved By:** ____________ **Date:** ____________
