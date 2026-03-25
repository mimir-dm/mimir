# Manage Campaign Sources

Configure which D&D source books are available in your campaign.

## Import Source Data

Before you can use catalog content (monsters, spells, items), source data must be imported into Mimir.

1. Click the **Settings** gear icon in the header bar
2. In the **Catalog** section, click **Import**
3. Select a compatible data archive (`.zip` format)
4. Wait for the import to complete — this may take a few minutes for large archives

The import format follows the schema established by the [5etools](https://5e.tools/) community project. Mimir reads structured JSON archives in this format to populate the catalog.

After import, source books appear in the catalog management list where you can enable or disable individual books.

## Configure Sources for a Campaign

Each campaign can restrict which source books are available, letting you run a "PHB only" game or include supplements selectively.

1. Open your campaign dashboard
2. Click the **Sources** button in the header
3. Toggle source books on or off
4. Close the modal

Changes take effect immediately — catalog searches within this campaign now filter by the enabled sources.

## Disable or Remove a Source

To disable a source book for a specific campaign:
1. Open Campaign Sources (dashboard → Sources button)
2. Toggle the book off

To remove a source book entirely from Mimir:
1. Open **Settings** → Catalog section
2. Find the source book in the list
3. Click **Remove** to delete it from the database

Removing a source deletes all catalog entries from that book. Campaign data (characters, homebrew, documents) is not affected.

## Tips

- Configure sources early when creating a campaign — it controls what appears in all catalog searches
- Characters can optionally have their own source restrictions, independent of the campaign (useful when players own different books)
- Imported data is stored locally in your SQLite database — no internet connection is needed after import

## See Also

- [Create a Campaign](./create-campaign.md) — Initial campaign setup
- [The Catalog System](../../explanation/catalog-system.md) — How source data works
- [Troubleshooting: Missing catalog data](../../reference/troubleshooting.md#missing-catalog-data)
