#!/usr/bin/env python3
"""Extract SRD test fixtures from the Mimir production database.

Produces:
  - frontend/__tests__/fixtures/*.ts  (TypeScript exports for Vitest)
  - crates/mimir-core/tests/fixtures/*.json (JSON for Rust integration tests)

SRD content is identified by `"srd":true` or `"basicRules":true` in the data blob.
All SRD content is published under the OGL and safe to commit.
"""

import json
import os
import sqlite3
import sys
from pathlib import Path
from typing import Any

# Paths
DB_PATH = os.path.expanduser(
    "~/Library/Application Support/com.mimir.app/data/mimir.db"
)
ROOT = Path(__file__).resolve().parent.parent
FE_FIXTURES = ROOT / "crates" / "mimir" / "frontend" / "__tests__" / "fixtures"
RS_FIXTURES = ROOT / "crates" / "mimir-core" / "tests" / "fixtures"

SRD_WHERE = """(data LIKE '%"srd":true%' OR data LIKE '%"basicRules":true%')"""


def connect():
    if not os.path.exists(DB_PATH):
        print(f"ERROR: Database not found at {DB_PATH}", file=sys.stderr)
        sys.exit(1)
    return sqlite3.connect(DB_PATH)


def entity_to_json(row: dict) -> dict:
    """Mimic the Rust entity_to_json: merge id/name/source into parsed data blob."""
    data = json.loads(row["data"])
    data["id"] = row["id"]
    data["name"] = row["name"]
    data["source"] = row["source"]
    if row.get("fluff"):
        try:
            data["fluff"] = json.loads(row["fluff"])
        except (json.JSONDecodeError, TypeError):
            pass
    return data


def dict_factory(cursor, row):
    return {col[0]: row[idx] for idx, col in enumerate(cursor.description)}


# ─── Extraction functions ────────────────────────────────────────────────

def extract_classes(conn) -> list[dict]:
    cur = conn.execute(
        f"SELECT id, name, source, data, fluff FROM classes WHERE {SRD_WHERE} ORDER BY name"
    )
    return [entity_to_json(r) for r in cur.fetchall()]


def extract_subclasses(conn) -> list[dict]:
    cur = conn.execute(
        f"SELECT id, name, class_name, source, data, fluff FROM subclasses WHERE {SRD_WHERE} ORDER BY class_name, name"
    )
    rows = cur.fetchall()
    result = []
    for r in rows:
        d = entity_to_json(r)
        d["className"] = r["class_name"]
        result.append(d)
    return result


def extract_class_features(conn) -> list[dict]:
    cur = conn.execute(
        f"SELECT id, name, source, class_name, class_source, level, data FROM class_features WHERE {SRD_WHERE} ORDER BY class_name, level, name"
    )
    rows = cur.fetchall()
    result = []
    for r in rows:
        data = json.loads(r["data"])
        data["id"] = r["id"]
        data["name"] = r["name"]
        data["source"] = r["source"]
        data["className"] = r["class_name"]
        data["classSource"] = r["class_source"]
        data["level"] = r["level"]
        result.append(data)
    return result


def extract_subclass_features(conn) -> list[dict]:
    cur = conn.execute(
        f"SELECT id, name, source, class_name, class_source, subclass_name, subclass_source, level, data "
        f"FROM subclass_features WHERE {SRD_WHERE} ORDER BY subclass_name, level, name"
    )
    rows = cur.fetchall()
    result = []
    for r in rows:
        data = json.loads(r["data"])
        data["id"] = r["id"]
        data["name"] = r["name"]
        data["source"] = r["source"]
        data["className"] = r["class_name"]
        data["classSource"] = r["class_source"]
        data["subclassName"] = r["subclass_name"]
        data["subclassSource"] = r["subclass_source"]
        data["level"] = r["level"]
        result.append(data)
    return result


def extract_backgrounds(conn) -> list[dict]:
    cur = conn.execute(
        f"SELECT id, name, source, data, fluff FROM backgrounds WHERE {SRD_WHERE} ORDER BY name"
    )
    return [entity_to_json(r) for r in cur.fetchall()]


def extract_races(conn) -> list[dict]:
    cur = conn.execute(
        f"SELECT id, name, source, data, fluff FROM races WHERE {SRD_WHERE} ORDER BY name"
    )
    return [entity_to_json(r) for r in cur.fetchall()]


def extract_items(conn) -> list[dict]:
    """Extract a representative sample of SRD items: weapons, armor, and a few magic items."""
    # Target items by name
    target_items = [
        # Weapons
        "Longsword", "Shortsword", "Dagger", "Longbow", "Shortbow",
        "Greataxe", "Handaxe", "Light Crossbow", "Heavy Crossbow",
        "Mace", "Quarterstaff", "Rapier", "Scimitar", "Warhammer",
        "Javelin", "Spear", "Greatsword", "Battleaxe",
        # Armor
        "Padded Armor", "Leather Armor", "Studded Leather Armor",
        "Hide Armor", "Chain Shirt", "Scale Mail", "Breastplate",
        "Half Plate Armor", "Ring Mail", "Chain Mail", "Splint Armor",
        "Plate Armor", "Shield",
        # Adventuring Gear
        "Backpack", "Rope, Hempen (50 feet)", "Torch", "Rations (1 day)",
        "Potion of Healing", "Potion of Greater Healing",
        # Magic Items
        "+1 Longsword", "+1 Shield", "Bag of Holding", "Cloak of Protection",
        "Ring of Protection", "Boots of Elvenkind", "Cloak of Elvenkind",
        "Deck of Many Things", "Staff of the Magi", "Vorpal Sword",
    ]
    placeholders = ",".join(["?" for _ in target_items])
    cur = conn.execute(
        f"SELECT id, name, source, item_type, rarity, data, fluff FROM items "
        f"WHERE name IN ({placeholders}) AND {SRD_WHERE} ORDER BY name",
        target_items,
    )
    rows = cur.fetchall()
    result = []
    for r in rows:
        d = entity_to_json(r)
        if r["item_type"]:
            d["_itemType"] = r["item_type"]
        if r["rarity"]:
            d["_rarity"] = r["rarity"]
        result.append(d)

    # If we didn't get enough, also grab any SRD items with specific types
    if len(result) < 20:
        existing_names = {r["name"] for r in result}
        cur2 = conn.execute(
            f"SELECT id, name, source, item_type, rarity, data, fluff FROM items "
            f"WHERE {SRD_WHERE} AND name NOT IN ({','.join(['?' for _ in existing_names])}) "
            f"ORDER BY name LIMIT 10",
            list(existing_names),
        )
        for r in cur2.fetchall():
            d = entity_to_json(r)
            if r["item_type"]:
                d["_itemType"] = r["item_type"]
            if r["rarity"]:
                d["_rarity"] = r["rarity"]
            result.append(d)

    return result


def extract_spells(conn) -> list[dict]:
    """Extract a representative sample of SRD spells across levels and schools."""
    # Target spells by name — representative cantrips through 9th level
    target_spells = [
        # Cantrips (level 0)
        "Fire Bolt", "Light", "Mage Hand", "Sacred Flame", "Eldritch Blast",
        "Minor Illusion", "Prestidigitation", "Guidance",
        # Level 1
        "Magic Missile", "Shield", "Cure Wounds", "Healing Word",
        "Thunderwave", "Detect Magic", "Mage Armor", "Bless",
        # Level 2
        "Misty Step", "Hold Person", "Spiritual Weapon", "Scorching Ray",
        "Lesser Restoration",
        # Level 3
        "Fireball", "Lightning Bolt", "Counterspell", "Dispel Magic",
        "Revivify", "Spirit Guardians",
        # Level 4
        "Greater Invisibility", "Banishment", "Dimension Door", "Polymorph",
        # Level 5
        "Cone of Cold", "Wall of Force", "Raise Dead", "Greater Restoration",
        # Level 6
        "Chain Lightning", "Heal", "Disintegrate",
        # Level 7
        "Teleport", "Finger of Death",
        # Level 8
        "Power Word Stun",
        # Level 9
        "Power Word Kill", "Wish",
    ]
    placeholders = ",".join(["?" for _ in target_spells])
    cur = conn.execute(
        f"SELECT id, name, source, level, school, ritual, concentration, data, fluff FROM spells "
        f"WHERE name IN ({placeholders}) AND {SRD_WHERE} ORDER BY level, name",
        target_spells,
    )
    rows = cur.fetchall()
    result = []
    for r in rows:
        d = entity_to_json(r)
        d["_level"] = r["level"]
        d["_school"] = r["school"]
        d["_ritual"] = bool(r["ritual"])
        d["_concentration"] = bool(r["concentration"])
        result.append(d)
    return result


def extract_monsters(conn) -> list[dict]:
    """Extract a representative sample of SRD monsters across CRs."""
    target_monsters = [
        # Low CR
        "Goblin", "Kobold", "Skeleton", "Zombie", "Wolf", "Rat",
        # Medium CR
        "Ogre", "Owlbear", "Minotaur", "Basilisk", "Wight",
        # High CR
        "Hill Giant", "Young Red Dragon", "Vampire",
        # Very High CR
        "Adult Red Dragon", "Lich", "Ancient Red Dragon",
    ]
    placeholders = ",".join(["?" for _ in target_monsters])
    cur = conn.execute(
        f"SELECT id, name, source, cr, creature_type, size, data, fluff FROM monsters "
        f"WHERE name IN ({placeholders}) AND {SRD_WHERE} ORDER BY name",
        target_monsters,
    )
    rows = cur.fetchall()
    result = []
    for r in rows:
        d = entity_to_json(r)
        d["_cr"] = r["cr"]
        d["_creatureType"] = r["creature_type"]
        d["_size"] = r["size"]
        result.append(d)
    return result


# ─── Test character fixtures ─────────────────────────────────────────────

def build_test_characters() -> list[dict]:
    """Build synthetic test character data matching CharacterResponse format."""
    return [
        {
            "id": "test-fighter-champion-5",
            "name": "Test Fighter",
            "campaign_id": "test-campaign-1",
            "is_npc": False,
            "level": 5,
            "race": "Human",
            "classes": [
                {
                    "class_name": "Fighter",
                    "subclass_name": "Champion",
                    "level": 5,
                    "hit_dice_used": 0,
                }
            ],
            "ability_scores": {
                "str": 16, "dex": 14, "con": 14,
                "int": 10, "wis": 12, "cha": 8,
            },
            "max_hp": 44,
            "current_hp": 44,
            "temp_hp": 0,
            "armor_class": 18,
            "speed": 30,
            "background": "Soldier",
            "alignment": "Lawful Good",
            "proficiencies": [
                {"name": "Athletics", "type": "skill"},
                {"name": "Intimidation", "type": "skill"},
                {"name": "All Armor", "type": "armor"},
                {"name": "Shields", "type": "armor"},
                {"name": "Simple Weapons", "type": "weapon"},
                {"name": "Martial Weapons", "type": "weapon"},
                {"name": "Strength", "type": "saving_throw"},
                {"name": "Constitution", "type": "saving_throw"},
            ],
            "features": [],
            "spells": [],
            "notes": "Test fixture: Level 5 Champion Fighter with SRD equipment",
        },
        {
            "id": "test-rogue-thief-3",
            "name": "Test Rogue",
            "campaign_id": "test-campaign-1",
            "is_npc": False,
            "level": 3,
            "race": "Halfling",
            "classes": [
                {
                    "class_name": "Rogue",
                    "subclass_name": "Thief",
                    "level": 3,
                    "hit_dice_used": 0,
                }
            ],
            "ability_scores": {
                "str": 8, "dex": 16, "con": 12,
                "int": 14, "wis": 10, "cha": 13,
            },
            "max_hp": 21,
            "current_hp": 21,
            "temp_hp": 0,
            "armor_class": 14,
            "speed": 25,
            "background": "Criminal",
            "alignment": "Chaotic Neutral",
            "proficiencies": [
                {"name": "Stealth", "type": "skill"},
                {"name": "Thieves' Tools", "type": "tool"},
                {"name": "Sleight of Hand", "type": "skill"},
                {"name": "Acrobatics", "type": "skill"},
                {"name": "Deception", "type": "skill"},
                {"name": "Light Armor", "type": "armor"},
                {"name": "Simple Weapons", "type": "weapon"},
                {"name": "Dexterity", "type": "saving_throw"},
                {"name": "Intelligence", "type": "saving_throw"},
            ],
            "features": [],
            "spells": [],
            "notes": "Test fixture: Level 3 Thief Rogue",
        },
        {
            "id": "test-wizard-evocation-5",
            "name": "Test Wizard",
            "campaign_id": "test-campaign-1",
            "is_npc": False,
            "level": 5,
            "race": "High Elf",
            "classes": [
                {
                    "class_name": "Wizard",
                    "subclass_name": "School of Evocation",
                    "level": 5,
                    "hit_dice_used": 0,
                }
            ],
            "ability_scores": {
                "str": 8, "dex": 14, "con": 12,
                "int": 17, "wis": 13, "cha": 10,
            },
            "max_hp": 27,
            "current_hp": 27,
            "temp_hp": 0,
            "armor_class": 12,
            "speed": 30,
            "background": "Sage",
            "alignment": "Neutral Good",
            "proficiencies": [
                {"name": "Arcana", "type": "skill"},
                {"name": "Investigation", "type": "skill"},
                {"name": "History", "type": "skill"},
                {"name": "Intelligence", "type": "saving_throw"},
                {"name": "Wisdom", "type": "saving_throw"},
            ],
            "features": [],
            "spells": [],
            "notes": "Test fixture: Level 5 Evocation Wizard",
        },
        {
            "id": "test-multiclass-fighter3-rogue2",
            "name": "Test Multiclass",
            "campaign_id": "test-campaign-1",
            "is_npc": False,
            "level": 5,
            "race": "Half-Elf",
            "classes": [
                {
                    "class_name": "Fighter",
                    "subclass_name": "Champion",
                    "level": 3,
                    "hit_dice_used": 0,
                },
                {
                    "class_name": "Rogue",
                    "subclass_name": None,
                    "level": 2,
                    "hit_dice_used": 0,
                },
            ],
            "ability_scores": {
                "str": 14, "dex": 16, "con": 12,
                "int": 10, "wis": 10, "cha": 14,
            },
            "max_hp": 33,
            "current_hp": 33,
            "temp_hp": 0,
            "armor_class": 16,
            "speed": 30,
            "background": "Folk Hero",
            "alignment": "Neutral Good",
            "proficiencies": [
                {"name": "Athletics", "type": "skill"},
                {"name": "Perception", "type": "skill"},
                {"name": "Stealth", "type": "skill"},
                {"name": "All Armor", "type": "armor"},
                {"name": "Shields", "type": "armor"},
                {"name": "Simple Weapons", "type": "weapon"},
                {"name": "Martial Weapons", "type": "weapon"},
                {"name": "Strength", "type": "saving_throw"},
                {"name": "Constitution", "type": "saving_throw"},
            ],
            "features": [],
            "spells": [],
            "notes": "Test fixture: Fighter 3 (Champion) / Rogue 2 multiclass",
        },
    ]


def build_homebrew_fixtures() -> list[dict]:
    """Build synthetic homebrew item fixtures in 5etools format."""
    return [
        {
            "id": "hb-item-1",
            "name": "Blade of Testing",
            "source": "HB",
            "type": "M",
            "rarity": "rare",
            "reqAttune": True,
            "weight": 3,
            "weaponCategory": "martial",
            "dmg1": "1d8",
            "dmgType": "S",
            "bonusWeapon": "+1",
            "entries": [
                "You gain a +1 bonus to attack and damage rolls made with this magic weapon.",
                "When you hit with this weapon, the target takes an extra 1d6 radiant damage.",
            ],
            "srd": False,
            "_isHomebrew": True,
        },
        {
            "id": "hb-item-2",
            "name": "Amulet of Test Protection",
            "source": "HB",
            "type": "WN",
            "rarity": "uncommon",
            "reqAttune": True,
            "entries": [
                "While wearing this amulet, you gain a +1 bonus to AC and saving throws.",
            ],
            "srd": False,
            "_isHomebrew": True,
        },
        {
            "id": "hb-item-3",
            "name": "Potion of Test Healing",
            "source": "HB",
            "type": "P",
            "rarity": "common",
            "reqAttune": False,
            "entries": [
                "When you drink this potion, you regain 3d8 + 3 hit points.",
            ],
            "srd": False,
            "_isHomebrew": True,
        },
    ]


# ─── Output functions ────────────────────────────────────────────────────

def write_json(path: Path, data: Any):
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)
    print(f"  Written: {path.relative_to(ROOT)} ({len(data)} entries)")


def write_ts_fixture(path: Path, var_name: str, data: list[dict]):
    """Write a TypeScript fixture file exporting the data array and named exports."""
    path.parent.mkdir(parents=True, exist_ok=True)
    json_str = json.dumps(data, indent=2, ensure_ascii=False)
    content = f"""// Auto-generated SRD test fixtures — do not edit manually
// Generated by scripts/extract-srd-fixtures.py
// SRD content is published under the OGL

/* eslint-disable @typescript-eslint/no-explicit-any */
export const {var_name}: Record<string, any>[] = {json_str} as const

// Named exports for convenience
"""
    # Add named exports for each entry by name (deduplicated)
    seen_names: set[str] = set()
    for i, item in enumerate(data):
        name = item.get("name", f"item_{i}")
        # Build a disambiguation prefix from className/subclassName if present
        prefix = ""
        if item.get("className"):
            prefix = item["className"].replace(" ", "")
        elif item.get("subclassName"):
            prefix = item["subclassName"].replace(" ", "")

        safe_name = (
            name.replace(" ", "")
            .replace("'", "")
            .replace("-", "")
            .replace(",", "")
            .replace("(", "")
            .replace(")", "")
            .replace("/", "")
            .replace("+", "Plus")
            .replace(".", "")
            .replace(":", "")
        )
        # camelCase the safe name
        safe_name = safe_name[0].lower() + safe_name[1:] if safe_name else f"item{i}"

        # Add prefix if there's a duplicate
        if safe_name in seen_names and prefix:
            safe_name = prefix[0].lower() + prefix[1:] + safe_name[0].upper() + safe_name[1:]

        # If still duplicate, append index
        if safe_name in seen_names:
            safe_name = f"{safe_name}_{i}"

        seen_names.add(safe_name)
        content += f"export const {safe_name} = {var_name}[{i}]\n"

    with open(path, "w") as f:
        f.write(content)
    print(f"  Written: {path.relative_to(ROOT)} ({len(data)} entries)")


def write_ts_index(path: Path, modules: list[str]):
    """Write an index.ts that re-exports all fixture modules."""
    path.parent.mkdir(parents=True, exist_ok=True)
    content = "// Auto-generated fixture index — do not edit manually\n"
    for mod in modules:
        content += f"export * from './{mod}'\n"
    with open(path, "w") as f:
        f.write(content)
    print(f"  Written: {path.relative_to(ROOT)}")


# ─── Rust fixture format ─────────────────────────────────────────────────

def build_rust_fixture(rows: list[dict], table_columns: list[str]) -> list[dict]:
    """Build Rust fixtures that include both table columns and the raw data blob."""
    result = []
    for row in rows:
        entry = {}
        for col in table_columns:
            if col in row:
                entry[col] = row[col]
            # Map camelCase back to snake_case for table columns
            camel = col.replace("_", " ").title().replace(" ", "")
            camel = camel[0].lower() + camel[1:]
            if camel in row and col not in entry:
                entry[col] = row[camel]
        # The data blob is the row minus metadata we added
        data_blob = {k: v for k, v in row.items()
                     if k not in ("id", "_itemType", "_rarity", "_cr", "_creatureType",
                                  "_size", "_level", "_school", "_ritual", "_concentration")}
        entry["data"] = json.dumps(data_blob, ensure_ascii=False)
        result.append(entry)
    return result


# ─── Main ────────────────────────────────────────────────────────────────

def main():
    print("Extracting SRD test fixtures from production database...")
    print(f"  DB: {DB_PATH}")
    print()

    conn = connect()
    conn.row_factory = dict_factory

    # Extract all categories
    print("Extracting classes...")
    classes = extract_classes(conn)

    print("Extracting subclasses...")
    subclasses = extract_subclasses(conn)

    print("Extracting class features...")
    class_features = extract_class_features(conn)

    print("Extracting subclass features...")
    subclass_features = extract_subclass_features(conn)

    print("Extracting backgrounds...")
    backgrounds = extract_backgrounds(conn)

    print("Extracting races...")
    races = extract_races(conn)

    print("Extracting items...")
    items = extract_items(conn)

    print("Extracting spells...")
    spells = extract_spells(conn)

    print("Extracting monsters...")
    monsters = extract_monsters(conn)

    print("Building test characters...")
    characters = build_test_characters()

    print("Building homebrew fixtures...")
    homebrew = build_homebrew_fixtures()

    conn.close()

    # ─── Write Frontend fixtures ──────────────────────────────────────
    print("\nWriting frontend fixtures:")
    ts_modules = []

    for var_name, filename, data in [
        ("srdClasses", "classes", classes),
        ("srdSubclasses", "subclasses", subclasses),
        ("srdClassFeatures", "classFeatures", class_features),
        ("srdSubclassFeatures", "subclassFeatures", subclass_features),
        ("srdBackgrounds", "backgrounds", backgrounds),
        ("srdRaces", "races", races),
        ("srdItems", "items", items),
        ("srdSpells", "spells", spells),
        ("srdMonsters", "monsters", monsters),
        ("testCharacters", "characters", characters),
        ("homebrewItems", "homebrew", homebrew),
    ]:
        write_ts_fixture(FE_FIXTURES / f"{filename}.ts", var_name, data)
        ts_modules.append(filename)

    write_ts_index(FE_FIXTURES / "index.ts", ts_modules)

    # ─── Write Rust fixtures ──────────────────────────────────────────
    print("\nWriting Rust fixtures:")
    write_json(RS_FIXTURES / "srd_classes.json", classes)
    write_json(RS_FIXTURES / "srd_subclasses.json", subclasses)
    write_json(RS_FIXTURES / "srd_class_features.json", class_features)
    write_json(RS_FIXTURES / "srd_subclass_features.json", subclass_features)
    write_json(RS_FIXTURES / "srd_backgrounds.json", backgrounds)
    write_json(RS_FIXTURES / "srd_races.json", races)
    write_json(RS_FIXTURES / "srd_items.json", items)
    write_json(RS_FIXTURES / "srd_spells.json", spells)
    write_json(RS_FIXTURES / "srd_monsters.json", monsters)
    write_json(RS_FIXTURES / "test_characters.json", characters)
    write_json(RS_FIXTURES / "homebrew_items.json", homebrew)

    print("\nDone! Summary:")
    print(f"  Classes: {len(classes)}")
    print(f"  Subclasses: {len(subclasses)}")
    print(f"  Class features: {len(class_features)}")
    print(f"  Subclass features: {len(subclass_features)}")
    print(f"  Backgrounds: {len(backgrounds)}")
    print(f"  Races: {len(races)}")
    print(f"  Items: {len(items)}")
    print(f"  Spells: {len(spells)}")
    print(f"  Monsters: {len(monsters)}")
    print(f"  Test characters: {len(characters)}")
    print(f"  Homebrew items: {len(homebrew)}")


if __name__ == "__main__":
    main()
