# Make a Campaign from an Adventure

Mimir can turn an adventure in the reader into a campaign to prepare and play. The adventure comes from the 5etools data or from a homebrew file. See [Manage Source Books](./manage-sources.md).

## What Mimir Makes

| Part of the adventure | Becomes |
|-----------------------|---------|
| A numbered chapter or part | A module |
| A top-level section of a chapter | A document of its module. A keyed area ("1. Cave Mouth", "K1. Front Courtyard") joins the section before it. |
| A map image with a square grid | A map of its module, with the grid of the image and no walls |
| A creature that the chapter names | A monster of its module, from the catalog |
| The introduction, the appendices, the credits | Campaign documents |

A map image with a hex grid or no grid stays an image in its document. A link to a place in another document goes to that document.

The campaign's books are the adventure's source and the sources of the creatures and items it names.

## Make the Campaign

1. Open **Reference**, then the **Books** tab.
2. In **Book**, choose the adventure in the **Adventures** group.
3. Click **Make a campaign…**.
4. Optionally, change the **Campaign name**.
5. For each part, choose what it becomes: **Module**, **Campaign document** or **Leave out**.
6. Optionally, for a module, tick **One document** to keep the whole chapter in one document.
7. Click **Make the campaign**.
8. Read the message, for example "Made 4 modules, 30 documents, 5 maps and 40 monsters." It also names what was not found: a map image that is not in the imported images, or a creature that is not in the catalog.
9. Click **Open the campaign**.

To get the map images, import the 5etools image archive before you make the campaign.

## With Claude

The MCP tools `preview_adventure_campaign` and `create_campaign_from_adventure` do the same. See [MCP Server Reference](../../reference/mcp-server.md).

## See Also

- [Manage Documents](./manage-documents.md): the read-aloud boxes, sidebars and stat blocks of adventure text
- [Upload a Map](../maps/upload-map.md): replace an image map with a UVTT map that has walls
