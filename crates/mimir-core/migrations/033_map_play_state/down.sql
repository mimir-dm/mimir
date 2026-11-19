DROP TABLE IF EXISTS map_door_states;
ALTER TABLE token_placements DROP COLUMN dead;
ALTER TABLE maps DROP COLUMN line_of_sight;
ALTER TABLE maps DROP COLUMN reveal_all;
