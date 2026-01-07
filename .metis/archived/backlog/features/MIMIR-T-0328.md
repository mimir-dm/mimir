---
id: campaign-image-upload-support
level: task
title: "Campaign Image Upload Support"
short_code: "MIMIR-T-0328"
created_at: 2026-01-07T09:32:29.975074+00:00
updated_at: 2026-01-07T09:32:29.975074+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Campaign Image Upload Support

## Objective

Allow users to upload and manage images within a campaign for use in puzzles, diagrams, handouts, and other visual content. Images should be stored in the campaign folder and easily referenceable in documents.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [ ] P2 - Medium (nice to have)

### Business Justification
- **User Value**: DMs frequently need to share visual content like puzzle diagrams, dungeon sketches, NPC portraits, or handout images with players
- **Business Value**: Makes Mimir more complete as a campaign management tool
- **Effort Estimate**: M

## Acceptance Criteria

## Acceptance Criteria

- [ ] Add "Images" or "Resources" section in campaign view
- [ ] Support drag-and-drop or file picker to upload images (PNG, JPG, WEBP, GIF)
- [ ] Store images in campaign folder under `resources/images/` or similar
- [ ] Display image gallery with thumbnails
- [ ] Copy markdown image reference to clipboard for easy embedding in documents
- [ ] Support image deletion
- [ ] Images included in campaign export/import

## Implementation Notes

### Technical Approach
1. Create `resources/images/` folder structure in campaign directory
2. Add `CampaignImageService` for CRUD operations
3. Add Tauri commands for upload, list, delete
4. Create `ImageGallery.vue` component with thumbnail grid
5. Generate unique filenames to avoid collisions
6. Consider optional image optimization/resizing on upload

### Storage Options
- Option A: Store in campaign folder (simpler, portable)
- Option B: Store in app data with references (centralized)
- Recommend Option A for better export/portability

### Dependencies
- Campaign folder structure already exists
- May want to integrate with document editor for easy insertion

### Risk Considerations
- Large images could bloat campaign size
- Consider max file size limit or optimization

## Status Updates

*To be added during implementation*