# NPC Roles

When creating NPCs with `create_npc` or assigning them to modules with `assign_npc_to_module`, use these roles:

## Available Roles

### quest_giver
Provides hooks and objectives to the party.
- Introduces the adventure premise
- Offers rewards for completion
- May provide resources or information

### ally
Actively helps the party achieve their goals.
- May join the party temporarily
- Provides assistance in combat or skill checks
- Has personal stake in party's success

### antagonist
Opposes the party's objectives.
- Primary villain or their agents
- Creates obstacles and complications
- Has competing goals or methods

### neutral
Could go either way based on party actions.
- Undecided loyalties
- Responds to party's approach
- Good for moral complexity

### merchant
Sells goods and services.
- Provides equipment and supplies
- May offer special or rare items
- Can be source of local information

### informant
Provides crucial information.
- Knows secrets about the adventure
- May require persuasion or payment
- Information may be partial or biased

## Multiple Roles

NPCs can serve different roles in different modules. For example:
- A merchant might be an informant in one module
- An ally might become an antagonist if betrayed
- A neutral NPC might become a quest_giver after trust is built

Use `assign_npc_to_module` to set the specific role an NPC plays in each module context.
