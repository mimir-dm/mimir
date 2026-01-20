---
id: token-image-import-and-storage
level: task
title: "Token image import and storage"
short_code: "MIMIR-T-0379"
created_at: 2026-01-20T02:44:02.601537+00:00
updated_at: 2026-01-20T02:44:02.601537+00:00
parent: MIMIR-I-0042
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0042
---

# Token image import and storage

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0042]]

## Objective

Implement token image import from 5etools image repository. Copy images to app data directory and store path references in monster table.

## Acceptance Criteria

- [ ] Create token directory structure: `{app_data_dir}/tokens/{source}/`
- [ ] Copy token images from 5etools img directory during import
- [ ] Update `token_image_path` column in monsters table
- [ ] Handle missing tokens gracefully (NULL path, log warning)
- [ ] Support common image formats (PNG, WEBP)
- [ ] Provide utility to resolve token path to absolute filesystem path

## Directory Structure

```
{app_data_dir}/
└── tokens/
    ├── MM/           # Monster Manual
    │   ├── aboleth.png
    │   ├── adult-red-dragon.png
    │   └── ...
    ├── VGM/          # Volo's Guide
    │   └── ...
    └── {source}/
        └── {monster-name-slug}.png
```

## Token Path Resolution

```rust
/// Get the absolute filesystem path for a token
pub fn resolve_token_path(
    app_data_dir: &Path,
    token_image_path: &str
) -> PathBuf {
    app_data_dir.join(token_image_path)
}

/// Generate the relative token path for storage
pub fn token_relative_path(source: &str, monster_name: &str) -> String {
    let slug = slugify(monster_name);
    format!("tokens/{}/{}.png", source, slug)
}
```

## Implementation Notes

### 5etools Token Location

Tokens in 5etools img repository:
- Location: `img/bestiary/tokens/{source}/{monster-name}.png`
- Some use WEBP format
- Name matches monster name with spaces/special chars

### Name Slugification

Convert monster names to filesystem-safe slugs:
- Lowercase
- Replace spaces with hyphens
- Remove special characters
- Example: "Adult Red Dragon" → "adult-red-dragon"

### Import Integration

Called from `CatalogImportService` during monster import:

```rust
fn import_monster(&mut self, monster: &Monster, source_img_dir: &Path) -> Result<i32> {
    // 1. Insert monster record
    let monster_id = self.insert_monster(monster)?;
    
    // 2. Copy token if exists
    let source_token = source_img_dir
        .join("bestiary/tokens")
        .join(&monster.source)
        .join(format!("{}.png", &monster.name));
    
    if source_token.exists() {
        let rel_path = token_relative_path(&monster.source, &monster.name);
        let dest_path = self.app_data_dir.join(&rel_path);
        
        fs::create_dir_all(dest_path.parent().unwrap())?;
        fs::copy(&source_token, &dest_path)?;
        
        self.update_monster_token(monster_id, &rel_path)?;
    }
    
    Ok(monster_id)
}
```

### Dependencies

- MIMIR-T-0371 (Monster table with token_image_path column)
- MIMIR-T-0378 (Import service integration)

## Status Updates **[REQUIRED]**

*To be added during implementation*