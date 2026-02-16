-- Initialize sort_order for campaign-level documents (grouped by campaign)
UPDATE documents SET sort_order = (
    SELECT COUNT(*)
    FROM documents d2
    WHERE d2.campaign_id = documents.campaign_id
      AND d2.module_id IS NULL
      AND d2.rowid <= documents.rowid
)
WHERE module_id IS NULL;

-- Initialize sort_order for module documents (grouped by module)
UPDATE documents SET sort_order = (
    SELECT COUNT(*)
    FROM documents d2
    WHERE d2.module_id = documents.module_id
      AND d2.module_id IS NOT NULL
      AND d2.rowid <= documents.rowid
)
WHERE module_id IS NOT NULL;
