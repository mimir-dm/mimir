---
id: implement-physical-play-kit-pdf
level: task
title: "Implement Physical Play Kit PDF generation"
short_code: "MIMIR-T-0259"
created_at: 2025-12-29T16:21:10.348854+00:00
updated_at: 2025-12-29T19:06:10.295009+00:00
parent: MIMIR-I-0027
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0027
---

# Implement Physical Play Kit PDF generation

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0027]]

## Objective

Implement the Physical Play Kit PDF generation that produces tiled maps at true scale plus token cutout sheets for all maps in a module or campaign.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

### Backend Implementation
- [x] Add `ModuleExportOptions` struct with granular flags:
  - `include_documents`, `include_monsters`, `include_npcs`, `include_map_previews`
  - `include_tiled_maps`, `include_token_cutouts`
- [x] Add `CampaignExportOptions` struct with granular flags:
  - `include_campaign_docs`, `include_module_content`, `include_npcs`, `include_map_previews`
  - `include_tiled_maps`, `include_token_cutouts`
- [x] Update `export_module_documents` to accept options parameter
- [x] Update `export_campaign_documents` to accept options parameter

### Play Kit Generation
- [x] Iterate all maps in module/campaign scope
- [x] Reuse existing map tiling logic from `print_map` (extracted to `generate_tiled_map_data()`)
- [x] Generate per-map: Assembly Guide → Tiles → Token Cutouts (if enabled)
- [x] Combine all maps into single PDF with section breaks

### PDF Structure (when both Reference + Play Kit selected)
```
1. Title Page
2. Table of Contents  
3. --- REFERENCE SECTION ---
   - Documents
   - Monsters (stat blocks)
   - NPCs (if selected)
   - Map Previews (1 page each)
4. --- PHYSICAL PLAY KIT ---
   - Map 1: Assembly Guide + Tiles + Cutouts
   - Map 2: Assembly Guide + Tiles + Cutouts
   - ...
```

### UX
- [x] Loading state during generation (PdfPreviewModal with setLoading)
- [ ] Progress indication for large exports (optional, nice-to-have - deferred)

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes **[CONDITIONAL: Technical Task]**

{Keep for technical tasks, delete for non-technical. Technical details, approach, or important considerations}

### Technical Approach
{How this will be implemented}

### Dependencies
{Other tasks or systems this depends on}

### Risk Considerations
{Technical risks and mitigation strategies}

## Status Updates **[REQUIRED]**

### 2025-12-29: Implementation Complete

**Backend Changes:**
- Added `ModuleExportOptions` and `CampaignExportOptions` structs with granular flags
- Created `TiledMapData` struct for tiled map output
- Extracted tiling logic into reusable `generate_tiled_map_data()` function
- Updated `export_campaign_documents` to generate tiled maps when `include_tiled_maps` is true
- Updated `export_module_documents` to support maps and tiled maps (was missing before)

**Print Service Changes:**
- Added `render_campaign_combined_with_all_extended()` method accepting tiled_maps data
- Updated `build_campaign_combined_data_with_all_extended()` to include tiled_maps in JSON output

**Template Changes:**
- Physical Play Kit section added to `campaign/combined.typ`
- Assembly guide page with grid visualization
- Individual tile pages with neighbor indicators
- Token cutouts integration via shared `cutouts.typ`

**Files Modified:**
- `crates/mimir-dm/src/commands/print/mod.rs`
- `crates/mimir-dm-print/src/campaign.rs`
- `crates/mimir-dm-print/templates/campaign/combined.typ`