# Adventures Page

The adventures page (`/adventures`) shows the adventures that you imported: the published adventures of the 5etools data, and the adventures of homebrew files. See [Manage Source Books](../../how-to/campaigns/manage-sources.md).

## The List

Each adventure has a card:

| Element | Function |
|---------|----------|
| Cover | The cover of the adventure, when the 5etools images are imported |
| Name | Open the adventure |
| Levels line | The level range and the storyline, for example "Levels 1–5 · Starter Set" |
| **Homebrew** | The adventure comes from a homebrew file |
| **Read** | Open the adventure |
| **Make a campaign…** | Make a campaign from the adventure. See [Make a Campaign from an Adventure](../../how-to/campaigns/campaign-from-adventure.md). |

### Search and Filters

| Control | Function |
|---------|----------|
| **Search** | Show the adventures whose name or storyline has the text |
| **Level** | Show the adventures for a character level. **Any level** turns the filter off. |
| **Group by** | **Storyline**: a group for each storyline, by name. In a group, the adventures are in the order of their first level. The adventures without a storyline are in **Other adventures**, at the end. **None (by name)**: one list, by name. |

Without adventures, the page says "No adventures yet. Import the 5etools data or a homebrew file (Settings)."

## An Adventure

The page of an adventure (`/adventures/<id>`) shows its text in the reader.

| Control | Function |
|---------|----------|
| **← Adventures** | Go back to the list |
| Levels line | The level range and the storyline |
| **Make a campaign…** | Make a campaign from the adventure |
| **Remove this adventure…** | Remove it, after a confirmation: its catalog entries and its text go, as **×** in Settings |
| **Contents** | The chapters and headings. Click one to go to it. |
| **← Previous**, **Next →** | Go to the previous or next chapter |

The text shows its tables, sidebars, read-aloud boxes and images. A map image says that it is a map, and its grid: for example "(map, square grid)". A cross-reference shows a short summary when you point at it. Click it to open the detail drawer.

A link to a place, for example "the gatehouse", goes to the chapter and the heading of that place.

The chapter and the heading are in the address (`?section=2&at=<heading>`). You can keep a link to a place in the adventure.

## See Also

- [Reference Page](./reference-page.md): the catalog and the books
- [Make a Campaign from an Adventure](../../how-to/campaigns/campaign-from-adventure.md)
