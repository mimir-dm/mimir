-- Add sort_order column to documents for user-controlled ordering
ALTER TABLE documents ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;

-- Assign sequential sort_order to existing campaign-level documents (grouped by campaign)
UPDATE documents SET sort_order = (
    SELECT COUNT(*)
    FROM documents d2
    WHERE d2.campaign_id = documents.campaign_id
      AND d2.module_id IS NULL
      AND documents.module_id IS NULL
      AND d2.rowid <= documents.rowid
)
WHERE module_id IS NULL;

-- Assign sequential sort_order to existing module documents (grouped by module)
UPDATE documents SET sort_order = (
    SELECT COUNT(*)
    FROM documents d2
    WHERE d2.module_id = documents.module_id
      AND d2.module_id IS NOT NULL
      AND documents.module_id IS NOT NULL
      AND d2.rowid <= documents.rowid
)
WHERE module_id IS NOT NULL;
