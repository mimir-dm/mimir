# Manage Documents

Documents hold your campaign and module content: session notes, places, lore, encounter plans and anything else for your game.

## Where Documents Live

A document belongs to a campaign or to a module:

- **Campaign documents**: world, lore, factions, setting. They are in the **Campaign** tab of the dashboard.
- **Module documents**: session prep, encounters, places. They are in the **Documents** tab of the module page.

Both places show the same document list and editor.

## Create a Document

1. Open the **Campaign** tab, or the **Documents** tab of a module.
2. Click **New document**.
3. Type a **Title**.
4. Optionally, drop a markdown file on **Or start from a file**, or click it to choose one. The file can be 5 MB at most.
5. Click **Create**.

The new document opens.

## Add an Image

An image is a handout, a portrait or a picture of a place. It belongs to the campaign or to the module.

1. Open the **Campaign** tab, or the **Documents** tab of a module.
2. Click **New document**.
3. Drop an image (PNG, JPEG, WebP, GIF or SVG) on **Or start from a file**, or click it to choose one. The image can be 50 MB at most. A preview shows.
4. Change the **Title** if you want.
5. Click **Create**.

The image opens big. It is listed under **Images**, below the documents.

## View or Delete an Image

1. Click the image under **Images**. It opens big.
2. To delete it, click **Delete image** and confirm.

## Read a Document

Click a document in the list. Mimir shows it on the right. The address holds the document (`?doc=…`), so you can bookmark it or send the link to yourself.

## Edit a Document

1. Open the document.
2. Click **Edit**.
3. Change the **Title** or the text.
4. Click **Done** when you finish.

The editor saves one second after the last change. The status shows **Saving…** and then **Saved**. There is no save button.

The toolbar has:
- **Heading 1**, **Heading 2**, **Heading 3**
- **Bold**, **Italic**, **Strikethrough**
- **Bullet list**, **Numbered list**, **Quote**, **Horizontal rule**
- **Insert a read-aloud box**, **Insert a sidebar**
- **Insert a table**, **Add a row**, **Add a column**, **Delete the row**, **Delete the column**, **Delete the table**
- **Undo**, **Redo**

Documents are markdown. Type `1. ` at the start of a line to start a numbered list.

## Add Adventure Text

Documents can hold the blocks of a published adventure. The editor shows each one as a box, and the document view and the PDF show it as the reader does.

| In the markdown | Shows as |
|-----------------|----------|
| `> [!read-aloud]` and quoted lines after it | A box of text to read aloud to the players |
| `> [!sidebar] Title` and quoted lines after it | A sidebar with its title |
| A line that is only `{@creature Goblin\|MM}` | The full stat block of that creature from the catalog |
| A line that is only `{@creatureFluff Goblin\|MM}` | The creature's lore |
| `{@creature Goblin\|MM\|two goblins}` in a sentence | A link that opens the creature in the detail drawer |

A stat block line can also name an item, a spell, a background or another catalog entry: `{@item Bag of Holding|DMG}`. To keep a line that is only a tag as text, start it with a backslash: `\{@creature Goblin|MM}`.

In a player's notes, tags show as their words only.

## Export a Document as PDF

1. Open the document.
2. Click **PDF**.

The browser downloads the PDF. Read-aloud boxes and sidebars print as boxes, and stat block lines print the stat block from the catalog.

## Reorder Documents

The documents that came with the campaign or module keep their order. Your own documents are under **Your documents**. They have up and down arrows.

1. Click the up arrow (**Move … up**) or the down arrow (**Move … down**) of a document.

## Delete a Document

1. Open the document.
2. Click **×** (**Delete the document**).
3. Click **Delete** to confirm.

The document goes for good.

## Session Notes

Each module has a **Play Notes** document for the session: HP, initiative, events. Open it from the **Documents** tab of the module.

## See Also

- [Create a Campaign](./create-campaign.md)
- [Module Documents](../modules/module-documents.md)
- [Start a Session](../play-mode/start-session.md)
