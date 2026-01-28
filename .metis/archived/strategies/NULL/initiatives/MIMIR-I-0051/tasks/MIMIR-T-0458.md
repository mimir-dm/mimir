---
id: tauri-commands-for-campaign-archive
level: task
title: "Tauri Commands for Campaign Archive"
short_code: "MIMIR-T-0458"
created_at: 2026-01-28T04:02:48.525778+00:00
updated_at: 2026-01-28T13:49:07.865785+00:00
parent: MIMIR-I-0051
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0051
---

# Tauri Commands for Campaign Archive

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0051]]

## Objective

Add Tauri commands to expose campaign archive functionality to the frontend.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `export_campaign` command - exports campaign to file path
- [ ] `preview_archive` command - returns archive info without importing
- [ ] `import_campaign` command - imports archive with optional new name
- [ ] Commands return appropriate progress/status for long operations
- [ ] Register commands in main.rs

## Command Signatures

```rust
#[tauri::command]
pub fn export_campaign(
    state: State<'_, AppState>,
    campaign_id: String,
    output_path: String,
) -> ApiResponse<ExportResult>;

#[tauri::command]
pub fn preview_archive(
    state: State<'_, AppState>,
    archive_path: String,
) -> ApiResponse<ArchivePreview>;

#[tauri::command]
pub fn import_campaign(
    state: State<'_, AppState>,
    archive_path: String,
    new_name: Option<String>,
) -> ApiResponse<ImportResult>;
```

## Dependencies

- MIMIR-T-0456 (Export Service)
- MIMIR-T-0457 (Import Service)

## Status Updates

### Completed
- Created `archive.rs` in `crates/mimir/src/commands/`
- Added `export_campaign` command - exports campaign to specified directory, returns path and size
- Added `preview_archive` command - static function to preview archive without import
- Added `import_campaign` command - imports archive with optional name override
- Added module to `mod.rs` and registered commands in `main.rs`
- All commands use proper `Path` types and delegate to `ArchiveService`
- Added `ImportResult` to mimir-core exports