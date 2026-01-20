//! Table catalog extraction types
//!
//! Types for deserializing 5etools table JSON data (random tables, etc.).

use super::types::SrdValue;
use serde::{Deserialize, Serialize};

/// Table cell content - can be a simple string or complex object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TableCell {
    Text(String),
    Number(i64),
    Object(serde_json::Value),
}

/// A D&D 5e table from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    /// Table caption/description
    #[serde(default)]
    pub caption: Option<String>,

    /// Column labels
    #[serde(default)]
    pub col_labels: Option<Vec<String>>,

    /// Column styles
    #[serde(default)]
    pub col_styles: Option<Vec<String>>,

    /// Table rows
    #[serde(default)]
    pub rows: Vec<Vec<TableCell>>,

    /// Introductory entries
    #[serde(default)]
    pub intro: Vec<serde_json::Value>,

    /// Concluding entries
    #[serde(default)]
    pub outro: Vec<serde_json::Value>,

    /// Footnotes
    #[serde(default)]
    pub footnotes: Vec<serde_json::Value>,

    /// Table include reference
    #[serde(default)]
    pub table_include: Option<serde_json::Value>,

    // Flags
    #[serde(default)]
    pub basic_rules: Option<bool>,
    #[serde(default)]
    pub has_fluff: Option<bool>,
    #[serde(default)]
    pub has_fluff_images: Option<bool>,

    // SRD status
    #[serde(default)]
    pub srd: Option<SrdValue>,
}

/// Container for table data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    #[serde(default)]
    pub table: Vec<Table>,
}

/// Table fluff data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableFluff {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub entries: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub images: Option<Vec<serde_json::Value>>,
}

/// Container for table fluff data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableFluffData {
    #[serde(default)]
    pub table_fluff: Vec<TableFluff>,
}

/// Categorize a table by its name.
pub fn categorize_table(name: &str) -> &'static str {
    let name_lower = name.to_lowercase();

    if name_lower.contains("madness") || name_lower.contains("insanity") {
        "Madness"
    } else if name_lower.contains("treasure")
        || name_lower.contains("loot")
        || name_lower.contains("hoard")
    {
        "Treasure"
    } else if name_lower.contains("encounter") || name_lower.contains("random") {
        "Encounters"
    } else if name_lower.contains("trinket") {
        "Trinkets"
    } else if name_lower.contains("wild magic") || name_lower.contains("surge") {
        "Wild Magic"
    } else if name_lower.contains("damage") || name_lower.contains("critical") {
        "Combat"
    } else if name_lower.contains("npc")
        || name_lower.contains("name")
        || name_lower.contains("personality")
    {
        "NPCs"
    } else if name_lower.contains("quest")
        || name_lower.contains("adventure")
        || name_lower.contains("plot")
    {
        "Adventures"
    } else if name_lower.contains("magic item") || name_lower.contains("artifact") {
        "Magic Items"
    } else {
        "Miscellaneous"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_table() {
        let json = json!({
            "name": "Wild Magic Surge",
            "source": "PHB",
            "caption": "Wild Magic Surge",
            "colLabels": ["d100", "Effect"],
            "rows": [
                ["01-02", "Roll on this table at the start of each of your turns."],
                ["03-04", "For the next minute, you can see any invisible creature."]
            ]
        });
        let table: Table = serde_json::from_value(json).unwrap();
        assert_eq!(table.name, "Wild Magic Surge");
        assert_eq!(table.col_labels.unwrap().len(), 2);
        assert_eq!(table.rows.len(), 2);
    }

    #[test]
    fn test_table_with_intro() {
        let json = json!({
            "name": "Trinkets",
            "source": "PHB",
            "intro": ["When you make your character, you can roll once on the Trinkets table."],
            "colLabels": ["d100", "Trinket"],
            "rows": [["01", "A mummified goblin hand"]]
        });
        let table: Table = serde_json::from_value(json).unwrap();
        assert_eq!(table.intro.len(), 1);
    }

    #[test]
    fn test_table_data() {
        let json = json!({
            "table": [
                {"name": "Wild Magic Surge", "source": "PHB"},
                {"name": "Trinkets", "source": "PHB"}
            ]
        });
        let data: TableData = serde_json::from_value(json).unwrap();
        assert_eq!(data.table.len(), 2);
    }

    #[test]
    fn test_categorize_table() {
        assert_eq!(categorize_table("Wild Magic Surge"), "Wild Magic");
        assert_eq!(categorize_table("Trinkets"), "Trinkets");
        assert_eq!(categorize_table("Random Encounters"), "Encounters");
        assert_eq!(categorize_table("Individual Treasure"), "Treasure");
        assert_eq!(categorize_table("Some Other Table"), "Miscellaneous");
    }

    #[test]
    fn test_table_cell_types() {
        let json = json!({
            "name": "Test Table",
            "source": "TEST",
            "rows": [
                ["text", 42, {"type": "complex"}]
            ]
        });
        let table: Table = serde_json::from_value(json).unwrap();
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.rows[0].len(), 3);
    }
}
