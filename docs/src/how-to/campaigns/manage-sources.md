# Manage Campaign Sources

Configure which D&D source books are available in your campaign.

## Import Source Data

Before you can use catalog content (monsters, spells, items), source data must be imported into Mimir.

Mimir imports `.tar.gz` archives of JSON data in the format established by the [5etools](https://5e.tools/) community project. Pre-packaged archives are available from the [Mimir Resources](https://github.com/mimir-dm/resources/releases) releases page; download both the data archive and the matching image archive.

1. Click the **Settings** gear icon in the header bar
2. Under **Admin Tools**, click **Import Books**
3. In the Manage Catalog Sources modal, click **Import 5etools Data**
4. Select the data archive (`.tar.gz`)
5. Wait for the import to complete — this may take a few minutes for large archives
6. (Optional) Click **Import Images** and select the image archive to add token art and book illustrations

After import, the source books appear in the Manage Catalog Sources list.

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
1. Click the **Settings** gear icon, then click **Import Books** under Admin Tools
2. Check the box next to each source book you want to remove
3. Click **Delete Selected**, then confirm

Removing a source deletes all catalog entries from that book. Campaign data (characters, homebrew, documents) is not affected.

## Tips

- Configure sources early when creating a campaign — it controls what appears in all catalog searches
- Characters can optionally have their own source restrictions, independent of the campaign (useful when players own different books)
- Imported data is stored locally in your SQLite database — no internet connection is needed after import

## See Also

- [Create a Campaign](./create-campaign.md) — Initial campaign setup
- [The Catalog System](../../explanation/catalog-system.md) — How source data works
- [Troubleshooting: Missing catalog data](../../reference/troubleshooting.md#missing-catalog-data)
