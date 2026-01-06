# 5etools Item Processing Pipeline

## Overview
5etools uses a sophisticated item processing pipeline that dynamically generates magic item variants from base items and generic variant templates. This document describes the complete processing flow.

## Data Sources

### 1. items-base.json
Contains base items (weapons, armor, gear) with properties like:
- Basic item stats (weight, value, damage, etc.)
- Item type (M for melee weapon, R for ranged, A for ammunition, etc.)
- Weapon/armor properties
- Flags like `weapon: true`, `sword: true`, etc.

### 2. items.json
Contains:
- **item**: Magic items and special items
- **itemGroup**: Groups of related items (e.g., "Arcane Focus")
- **itemProperty**: Definitions of item properties (e.g., "Finesse", "Heavy")
- **itemType**: Definitions of item types with their descriptions
- **itemEntry**: Additional entries for items
- **itemTypeAdditionalEntries**: Extra entries for specific item types
- **itemMastery**: Weapon mastery properties (from newer content)

### 3. magicvariants.json
Contains:
- **magicvariant**: Generic variant templates (e.g., "+1 Weapon", "+2 Armor")
- **linkedLootTables**: Mapping of items to loot tables

## Processing Steps

### Step 1: Load and Mark Base Items
```javascript
baseItemData.baseitem.forEach(it => it._isBaseItem = true);
```
- Load items from items-base.json
- Mark each with `_isBaseItem = true`
- Set `_category = "Basic"` for base items

### Step 2: Load and Process Item Groups
```javascript
itemData.itemGroup.forEach(it => it._isItemGroup = true);
```
- Load item groups from items.json
- Mark each with `_isItemGroup = true`

### Step 3: Process Magic Variants
For each generic variant in magicvariants.json:
1. Add inherited properties to self
2. For each base item, check if it matches the variant

### Step 4: Variant Matching Logic

#### Requirements Matching
```javascript
// Requirements are OR'd together - item must match ANY requirement object
genericVariant.requires.some(req => 
    isRequiresExcludesMatch(baseItem, req, "every")
)
```

Example requirement structures:
```json
// Simple requirement
{ "weapon": true }

// Multiple requirements (must match ALL properties in the object)
{ "type": "M", "sword": true }

// Array of requirements (item must match ANY)
[
    { "type": "A" },
    { "type": "AF|DMG" }
]
```

#### Exclusions Matching
```javascript
// If item matches ANY exclusion, it's excluded
isRequiresExcludesMatch(baseItem, genericVariant.excludes, "some")
```

### Step 5: Create Specific Variants
When a base item matches a generic variant:

1. **Copy base item** and update metadata:
   - Set `__prop = "item"`
   - Remove `_isBaseItem`
   - Set `_category = "Specific Variant"`
   - Store `_baseName`, `_baseSource`, etc.

2. **Apply inherited properties** from variant:
   - `namePrefix`: Prepend to item name
   - `nameSuffix`: Append to item name
   - `nameRemove`: Remove text from name
   - `source`, `page`, `rarity`, `tier`: Override base values
   - `bonusWeapon`, `bonusAc`: Combat bonuses
   - `entries`: Additional description entries

3. **Handle special properties**:
   - `propertyAdd`: Add new properties
   - `propertyRemove`: Remove existing properties
   - `weightExpression`, `valueExpression`: Calculate new values
   - `barding`: Special armor variant

4. **Process entry templates**:
   Replace template variables in entries:
   - `{=bonusWeapon}` → "+1", "+2", etc.
   - `{=dmgType}` → Full damage type name
   - `{=baseName}` → Original item name

### Step 6: Enhance Items
The `enhanceItem` function adds:
- Full entries from item types and properties
- Special handling for armor (stealth disadvantage, strength requirements)
- Spellcasting focus descriptions
- Weapon mastery properties
- Links to variant rules (e.g., tool proficiencies)

## Key Implementation Details

### Template Variable Replacement
```javascript
const injectableProps = {
    baseName: baseItem.name,
    dmgType: Parser.dmgTypeToFull(baseItem.dmgType),
    bonusAc: inherits.bonusAc,
    bonusWeapon: inherits.bonusWeapon,
    bonusWeaponAttack: inherits.bonusWeaponAttack,
    bonusWeaponDamage: inherits.bonusWeaponDamage,
    bonusWeaponCritDamage: inherits.bonusWeaponCritDamage,
    bonusSpellAttack: inherits.bonusSpellAttack,
    bonusSpellSaveDc: inherits.bonusSpellSaveDc,
    bonusSavingThrow: inherits.bonusSavingThrow
};
```

### Property Inheritance
Properties that are merged:
- Arrays are concatenated (e.g., `entries`, `property`)
- Objects are merged recursively

Properties that are NOT inherited:
- `namePrefix`, `nameSuffix`, `nameRemove` (applied then discarded)
- `propertyAdd`, `propertyRemove` (applied then discarded)

## Example: Creating "+1 Longsword"

1. **Base Item** (Longsword):
```json
{
    "name": "Longsword",
    "type": "M",
    "weapon": true,
    "sword": true,
    "dmgType": "S",
    "dmg1": "1d8"
}
```

2. **Generic Variant** (+1 Weapon):
```json
{
    "name": "+1 Weapon",
    "requires": [{"weapon": true}],
    "excludes": {"net": true},
    "inherits": {
        "namePrefix": "+1 ",
        "source": "DMG",
        "rarity": "uncommon",
        "bonusWeapon": "+1",
        "entries": ["You have a {=bonusWeapon} bonus to attack and damage rolls made with this magic weapon."]
    }
}
```

3. **Result** (+1 Longsword):
```json
{
    "name": "+1 Longsword",
    "type": "M",
    "weapon": true,
    "sword": true,
    "dmgType": "S",
    "dmg1": "1d8",
    "source": "DMG",
    "rarity": "uncommon",
    "bonusWeapon": "+1",
    "entries": ["You have a +1 bonus to attack and damage rolls made with this magic weapon."],
    "_baseName": "Longsword",
    "_baseSource": "PHB",
    "_variantName": "+1 Weapon",
    "_category": "Specific Variant"
}
```

## Implementation Considerations for Splitter

For the Mimir 5etools splitter, we need to:

1. **Pre-generate all variants** at build time rather than runtime
2. **Include both generic and specific variants** in the output
3. **Preserve metadata** for linking variants to their base items
4. **Handle template variable replacement** during generation
5. **Support itemGroup** processing from items.json
6. **Consider property and type definitions** for complete item data

## Files to Generate

For each book archive:
- `items/{source}.json` - All items including generated variants
- `items/expanded-weapons.json` - Separate file for weapon variants (optional)
- `items/fluff-{source}.json` - Fluff entries for items