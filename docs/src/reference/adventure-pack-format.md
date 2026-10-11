# Adventure Pack Format

A Mimir adventure pack holds an adventure to be played, for another Mimir. It is a zip file with the extension `.mimir-adventure.zip` (ADR MIMIR-A-0011).

## Which File to Use

| You want to | Use |
|-------------|-----|
| Back up a campaign, or move it to another computer as it was played | A [campaign archive](./campaign-archive-format.md) (`.mimir-campaign.tar.gz`): it holds everything, the player characters, the fog and the doors too |
| Give an adventure, ready to play, to another Mimir user | An adventure pack (`.mimir-adventure.zip`) |
| Give an adventure to a 5etools user | An [adventure file](./adventure-file-format.md) (`.json`) |

## Contents

| Path | Holds |
|------|-------|
| `mimir.json` | `format` (`mimir-adventure`), `version` (`1.0`), `created_at`, `mimir_version`, `adventure_source`, `adventure_name`, and `data` |
| `adventure.json` | The campaign as a 5etools homebrew adventure file, with a copy of every catalog entry it names. See [Adventure File Format](./adventure-file-format.md). |
| `assets/<asset id>/<filename>` | The map files and images |

`data` has the shape of the campaign archive's `data.json`: the campaign, its books, modules, documents, NPC characters, maps (walls, grid, lighting, lights, traps, points of interest), token placements, module monsters and NPCs, assets, and the campaign's homebrew.

A pack leaves out what belongs to the table: the player characters, the fog, the doors' state, dead tokens, triggered traps, and revealed maps. Player links and player notes are not in a campaign's data either.

## Import

Mimir checks `format` and the major version of `version`. It refuses a pack of another major version, with a message that names both versions.

When the pack's adventure source is not in the catalog, Mimir imports `adventure.json` first: the adventure goes to the reader, and the copies of the catalog entries go to the catalog under the adventure's source. A module monster whose catalog entry this Mimir does not have then uses the pack's copy.

The import makes a new campaign, with new ids, in one transaction.

## See Also

- [Export a Campaign as an Adventure](../how-to/campaigns/export-adventure.md)
- [Campaign Archive Format](./campaign-archive-format.md)
