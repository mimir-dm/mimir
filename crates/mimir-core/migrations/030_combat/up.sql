-- Combat tracker (COLLIERY-I-0468, MIMIR-T-0676).
-- One active combat per module; entries are the creatures in initiative order.

CREATE TABLE combat_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    module_id TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    -- Current round, starting at 1
    round INTEGER NOT NULL DEFAULT 1,
    -- Index into the entries in turn order of whose turn it is
    turn_index INTEGER NOT NULL DEFAULT 0,
    -- 'active' or 'ended'
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'ended')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- At most one active combat per module.
CREATE UNIQUE INDEX idx_combat_sessions_one_active
    ON combat_sessions(module_id) WHERE status = 'active';

CREATE TABLE combat_entries (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES combat_sessions(id) ON DELETE CASCADE,
    -- 'module_monster', 'module_npc', 'character' or 'custom'
    source_kind TEXT NOT NULL CHECK (source_kind IN ('module_monster', 'module_npc', 'character', 'custom')),
    -- id in the source table (NULL for custom entries)
    source_id TEXT,
    -- Map token this entry is (optional)
    token_id TEXT,
    display_name TEXT NOT NULL,
    -- NULL until rolled
    initiative INTEGER,
    -- Dexterity modifier for initiative ties (NULL when unknown)
    dex_modifier INTEGER,
    -- HP is optional (PCs have no HP column)
    max_hp INTEGER,
    current_hp INTEGER,
    temp_hp INTEGER NOT NULL DEFAULT 0,
    is_concentrating INTEGER NOT NULL DEFAULT 0,
    -- JSON array of {"name": "...", "expires_round": n|null}
    conditions TEXT NOT NULL DEFAULT '[]',
    -- JSON array of the last 3 HP changes: {"amount": n, "kind": "damage"|"heal", "round": n}
    damage_log TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_combat_entries_session ON combat_entries(session_id);
