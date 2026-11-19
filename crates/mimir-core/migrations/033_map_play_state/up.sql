-- Play state of a map that the player display follows (MIMIR-T-0728).

-- The DM shows the whole map to the players: no fog, no sight limits.
ALTER TABLE maps ADD COLUMN reveal_all INTEGER NOT NULL DEFAULT 0;
-- Walls and closed doors block sight (on the DM map and the display).
ALTER TABLE maps ADD COLUMN line_of_sight INTEGER NOT NULL DEFAULT 1;

-- The DM marked the token dead.
ALTER TABLE token_placements ADD COLUMN dead INTEGER NOT NULL DEFAULT 0;

-- A door of the map file that the DM opened or closed. No row: the
-- door is as the file has it. `portal_index` is the door's index in
-- the file's portals.
CREATE TABLE map_door_states (
    map_id TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    portal_index INTEGER NOT NULL,
    closed INTEGER NOT NULL,
    PRIMARY KEY (map_id, portal_index)
);
