# Level Up Workflow

This example shows how to level up a character, including multiclassing and feat selection.

## Step 1: Check Current State

Always read the character first:

```
get_character(character_id: "character-id")
```

Note the current level, class(es), and ability scores.

## Step 2: Simple Level Up

Level up in the same class using average HP:

```
level_up_character(
  character_id: "character-id",
  class_name: "Warlock",
  hp_method: "average"
)
```

## Step 3: Level Up with ASI

At ASI levels (4, 8, 12, 16, 19), choose ability score increases:

```
level_up_character(
  character_id: "character-id",
  class_name: "Warlock",
  hp_method: "average",
  asi_type: "asi",
  asi_ability1: "Constitution",
  asi_increase1: 1,
  asi_ability2: "Charisma",
  asi_increase2: 1
)
```

Use `asi_ability1`/`asi_increase1` for the first increase and optionally `asi_ability2`/`asi_increase2` for a second. Total increases must equal 2.

## Step 4: Level Up with Feat

Take a feat instead of an ASI:

```
search_catalog(category: "feat", name: "War Caster")

level_up_character(
  character_id: "character-id",
  class_name: "Warlock",
  hp_method: "average",
  asi_type: "feat",
  feat_name: "War Caster"
)
```

## Step 5: Multiclass Level Up

Level up in a different class:

```
level_up_character(
  character_id: "character-id",
  class_name: "Sorcerer",
  hp_method: "average"
)
```

## Step 6: Subclass Selection

When reaching a subclass level, include the subclass:

```
level_up_character(
  character_id: "character-id",
  class_name: "Warlock",
  hp_method: "average",
  subclass_name: "The Fiend"
)
```

## Step 7: Verify

```
get_character(character_id: "character-id")
```

Check the updated level, class breakdown, HP, and any new features.
