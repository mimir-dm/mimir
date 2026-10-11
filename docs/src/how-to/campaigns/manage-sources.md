# Manage Source Books

This guide shows you how to import the catalog data and choose which source books a campaign uses.

## Import Source Data

Mimir has no game content until you import it. It imports `.tar.gz` archives of JSON data in the format of the [5etools](https://5e.tools/) community project. Archives are on the [Mimir Resources](https://github.com/mimir-dm/resources/releases) releases page. Get the data archive and the image archive of the same release.

1. Click **Settings** in the side navigation.
2. Click the **Books** tab.
3. In **Import 5etools data**, choose the data archive.
4. Click **Import**.
5. Wait for the message "Imported … sources". A full import takes a few minutes. You can use Mimir in the meantime.
6. Optionally, in **Import 5etools images**, choose the image archive and click **Import**. The images add pictures to the books, adventures, monsters and items.

The data archive also holds the published adventures. Mimir imports each adventure's text into the reader, and its own monsters, items and spells into the catalog.

## Import a Homebrew File

A homebrew file is a `.json` file in the 5etools homebrew format. It can hold an adventure, a book, monsters, items and spells.

1. Open **Settings**, then **Books**.
2. In **Import a homebrew file**, choose the file.
3. Click **Import**.
4. Read the message, for example "Imported HBX: 1 adventure, 3 entries." The adventure is in the reader, and the entries are in the catalog under the source of the file.

Mimir refuses a file whose source is already in the catalog. To import a new version of the file, remove the source first. See [Remove a Source Book](#remove-a-source-book).

The books appear in the **Sources** table. **Readable** shows whether you can read the book text in the reference.

## Choose the Books of a Campaign

1. Open the campaign.
2. Open **More** in the dashboard header.
3. Click **Source books…**.
4. Select the books that the campaign uses. **All** and **None** select every book or none; **Core only** selects the core rule books (PHB, MM, DMG, and their 2024 editions) that the catalog has.
5. Click **Save**.

The monster search of the campaign's modules (**Add a monster**, **Monster from the catalog…** on the DM map) now keeps to these books. With no book selected, it uses every book. The reference page always shows every book.

## Choose the Books of a Character

A player can own other books than the campaign uses.

1. Open the character sheet.
2. Click **Source books…**.
3. Select the books.
4. Click **Save**.

## Remove a Source Book

1. Click **Settings**, then the **Books** tab.
2. In the **Sources** table, click **×** (**Remove …**) in the row of the book.
3. Click **Remove** to confirm.

Mimir deletes the catalog entries and the book text of that source. Characters and campaigns keep what they hold. Import the archive again to bring the book back.

## Tips

- Choose the books when you make a campaign. They control what the searches show in it.
- The data stays on your server. Mimir does not need the internet after the import.

## See Also

- [Create a Campaign](./create-campaign.md)
- [The Catalog System](../../explanation/catalog-system.md)
- [Troubleshooting: No catalog results](../../reference/troubleshooting.md#no-catalog-results)
