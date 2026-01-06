# LLM Tools System

Mimir's AI assistant uses a tool system that enables interaction with campaign data through natural language. When you ask questions like "What are Thorin's current hit points?" or request actions like "Cast fireball at 4th level," the assistant translates these into tool calls that query and modify the underlying database.

## Architecture

Tools are implemented as Rust structs that implement the `ToolTrait` interface. Each tool provides a name, description, JSON schema for its parameters, and an async execute method that performs the actual work. The ToolRegistry collects all available tools at session start, generates tool definitions for the LLM provider, and routes execution requests to the appropriate implementation.

```rust
pub trait ToolTrait {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Value;
    async fn execute(&self, args: Value) -> Result<String>;
}
```

The registry provides tool definitions in the format expected by LLM providers, allowing the model to understand what tools are available and how to call them. When the model responds with a tool call, the registry looks up the tool by name and invokes its execute method with the provided arguments.

## ReAct Pattern

The assistant uses a Reasoning and Acting pattern for complex tasks. Before taking action, it analyzes what needs to be done through explicit THOUGHT steps. It then executes the appropriate ACTION by calling tools. After receiving results, it makes OBSERVATION notes about what succeeded or failed and what to do next. This cycle repeats until the task is complete.

The system prompt instructs the model to use `<thought>` blocks for visible reasoning on multi-step workflows, debugging scenarios, and complex character operations. Simple lookups and single-step operations skip the explicit reasoning to keep responses concise.

## Available Tools

### Character Tools

The `get_character` tool retrieves complete character data including stats, inventory, and spells for a given character ID. The `list_campaign_characters` tool returns all characters in a campaign, with `list_pcs` and `list_npcs` variants for filtering by character type. The `list_players` tool returns registered players for associating with characters. The `get_character_stats` tool provides a focused view of ability scores, saves, and skills. The `check_spell_slots` tool returns current spell slot availability.

For modifications, `create_character` handles full PC creation while `create_npc` provides a streamlined path for non-player characters. The `update_character` tool modifies character attributes, and `update_character_hp` specifically handles damage and healing. The `level_up` tool advances a character and recalculates derived statistics. The `cast_spell` tool expends a spell slot of the appropriate level. The `take_rest` tool handles short and long rest resource recovery.

### Inventory Tools

The `add_inventory_item` tool adds items to a character's inventory by name or catalog reference. The `remove_inventory_item` tool removes items by inventory entry ID. The `update_equipped` tool changes which items are currently equipped. The `update_currency` tool modifies gold, silver, copper, and other currency amounts.

### Catalog Search Tools

The `search_monsters` tool queries the monster catalog with filters for name, challenge rating, creature type, size, and alignment. The `search_spells` tool searches spells by name, level, school, and class list. The `search_items` tool finds equipment and magic items by name, type, and rarity.

### Module Tools

The `create_module` tool creates new adventure modules within a campaign. The `list_modules` tool returns all modules for a campaign. The `get_module` tool retrieves module details including status and session count. The `update_module_status` tool tracks module progress through planning, active, and completed states.

### File Tools

When a campaign is active, file tools become available for managing campaign documents. The `read_file` tool retrieves file contents from the campaign directory. The `write_file` tool creates or overwrites files. The `edit_file` tool modifies specific line ranges within existing files. The `list_files` tool returns the campaign directory structure.

### Utility Tools

The `todo_write` tool manages a task list for tracking progress on multi-step operations. The assistant uses this to show users what steps it plans to take and mark items complete as it works through complex requests.

## Tool Execution Flow

When the assistant decides to use a tool, it includes a tool_calls array in its response specifying the tool name and arguments. The ChatProcessor extracts these calls and routes them through the ToolRegistry. Each tool executes asynchronously and returns a string result. The processor collects all results and sends them back to the model in a follow-up message, allowing the model to interpret the results and decide whether to make additional tool calls or provide a final response.

The system limits tool execution to 20 iterations per request to prevent infinite loops. Most requests complete in one to three iterations depending on complexity.

## Adding Custom Tools

Creating a new tool requires implementing the ToolTrait interface on a struct. The name method returns the tool identifier used in LLM responses. The description method provides text that helps the model understand when to use the tool. The parameters method returns a JSON schema defining the expected arguments. The execute method performs the actual work, returning a string result or error.

Register the new tool in the `register_all_tools` function in `tools/mod.rs`. The tool automatically becomes available in subsequent chat sessions.
