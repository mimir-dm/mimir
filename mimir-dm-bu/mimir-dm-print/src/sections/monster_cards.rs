//! Monster cards section
//!
//! Generates half-page (4" x 5.5") monster cards for combat reference.
//! Shows full stat blocks including attacks, resistances, and abilities.

use serde_json::Value;

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// Monster cards section - generates half-page monster reference cards (2x2 layout)
pub struct MonsterCardSection {
    /// Monster data (JSON array)
    monsters: Vec<Value>,
    /// Show cut lines between cards
    show_cut_lines: bool,
}

impl MonsterCardSection {
    /// Create a new monster cards section
    pub fn new(monsters: Vec<Value>) -> Self {
        Self {
            monsters,
            show_cut_lines: true,
        }
    }

    /// Create from a JSON value (expects array)
    pub fn from_json(monsters: Value) -> Self {
        let monster_vec = monsters
            .as_array()
            .map(|arr| arr.clone())
            .unwrap_or_default();
        Self::new(monster_vec)
    }

    /// Set whether to show cut lines
    pub fn with_cut_lines(mut self, show: bool) -> Self {
        self.show_cut_lines = show;
        self
    }

    /// Render a single monster card (half-page format)
    fn render_card(monster: &Value) -> String {
        let name = monster
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let source = monster
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Size
        let size = monster
            .get("size")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or("M");
        let size_name = match size.to_uppercase().as_str() {
            "T" => "Tiny",
            "S" => "Small",
            "M" => "Medium",
            "L" => "Large",
            "H" => "Huge",
            "G" => "Gargantuan",
            _ => "Medium",
        };

        // Creature type
        let creature_type = extract_creature_type(monster);
        let alignment = extract_alignment(monster);

        // AC with source
        let ac_display = extract_ac_full(monster);

        // HP with formula
        let hp_display = extract_hp_full(monster);

        // HP tracker (numeric value for boxes)
        let hp_average = extract_hp_average(monster);
        let hp_tracker = render_hp_tracker(hp_average);

        // Speed (all types)
        let speed_display = extract_speed_full(monster);

        // Passive perception
        let passive = monster
            .get("passive")
            .and_then(|v| v.as_i64())
            .unwrap_or(10);

        // Ability scores
        let str_score = monster.get("str").and_then(|v| v.as_i64()).unwrap_or(10);
        let dex_score = monster.get("dex").and_then(|v| v.as_i64()).unwrap_or(10);
        let con_score = monster.get("con").and_then(|v| v.as_i64()).unwrap_or(10);
        let int_score = monster.get("int").and_then(|v| v.as_i64()).unwrap_or(10);
        let wis_score = monster.get("wis").and_then(|v| v.as_i64()).unwrap_or(10);
        let cha_score = monster.get("cha").and_then(|v| v.as_i64()).unwrap_or(10);

        // CR
        let cr = extract_cr(monster);

        // Save proficiencies
        let saves = extract_saves(monster);

        // Senses
        let senses = extract_senses(monster);

        // Languages
        let languages = extract_languages(monster);

        // Damage vulnerabilities, resistances, immunities
        let vulnerabilities = extract_damage_list(monster, "damage_vulnerabilities", "damageVulnerabilities");
        let resistances = extract_damage_list(monster, "damage_resistances", "damageResistances");
        let damage_immunities = extract_damage_list(monster, "damage_immunities", "damageImmunities");
        let condition_immunities = extract_condition_immunities(monster);

        // Plan card layout (single or foldable)
        let layout = plan_card_layout(monster);

        // Build front sections content
        let front_sections_content = render_sections(&layout.front_sections);

        format!(
            r##"box(
  width: 3.875in,
  height: 5.125in,
  stroke: (
    top: 3pt + colors.accent,
    bottom: 3pt + colors.accent,
    left: 0.5pt + colors.border,
    right: 0.5pt + colors.border,
  ),
  radius: 3pt,
  clip: true,
  inset: 0pt,
)[
  // HP Tracker
  #block(
    width: 100%,
    inset: (x: 6pt, y: 4pt),
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    {hp_tracker}
  ]

  // Header
  #block(
    width: 100%,
    fill: colors.background-alt,
    inset: (x: 6pt, y: 4pt),
  )[
    #grid(
      columns: (1fr, auto),
      [
        #text(size: 10pt, weight: "bold")[{name}]
        #linebreak()
        #text(size: 7pt, style: "italic")[{size_name} {creature_type}, {alignment}]
      ],
      align(right + horizon)[
        #text(size: 12pt, weight: "bold", fill: colors.accent)[CR {cr}]
      ]
    )
  ]

  // Core stats
  #block(
    width: 100%,
    inset: 6pt,
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: 7pt)
    #grid(
      columns: (1fr, 1fr, 1fr),
      row-gutter: 2pt,
      [*AC* {ac_display}],
      [*HP* {hp_display}],
      [*PP* {passive}],
    )
    #v(2pt)
    #text(size: 6pt)[*Speed* {speed_display}]
  ]

  // Ability scores
  #block(
    width: 100%,
    inset: (x: 6pt, y: 4pt),
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: 6pt)
    #grid(
      columns: (1fr,) * 6,
      align(center)[*STR*\ {str_score} ({str_mod})],
      align(center)[*DEX*\ {dex_score} ({dex_mod})],
      align(center)[*CON*\ {con_score} ({con_mod})],
      align(center)[*INT*\ {int_score} ({int_mod})],
      align(center)[*WIS*\ {wis_score} ({wis_mod})],
      align(center)[*CHA*\ {cha_score} ({cha_mod})],
    )
  ]

  // Saves, Senses, Languages
  #block(
    width: 100%,
    inset: (x: 6pt, y: 3pt),
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: 6pt)
    {saves_line}
    {senses_line}
    {languages_line}
  ]

  // Resistances/Immunities
  {resistances_block}

  // Front sections (traits, actions, etc.)
  {front_sections_content}

  // Footer
  #place(
    bottom + left,
    block(
      width: 100%,
      fill: colors.background-alt,
      inset: (x: 6pt, y: 2pt),
    )[
      #text(size: 5pt, fill: colors.text-secondary)[{source}{fold_indicator}]
    ]
  )
]"##,
            name = escape_typst(name),
            size_name = size_name,
            creature_type = escape_typst(&creature_type),
            alignment = escape_typst(&alignment),
            cr = escape_typst(&cr),
            ac_display = escape_typst(&ac_display),
            hp_display = escape_typst(&hp_display),
            hp_tracker = hp_tracker,
            passive = passive,
            speed_display = escape_typst(&speed_display),
            str_score = str_score,
            str_mod = modifier(str_score),
            dex_score = dex_score,
            dex_mod = modifier(dex_score),
            con_score = con_score,
            con_mod = modifier(con_score),
            int_score = int_score,
            int_mod = modifier(int_score),
            wis_score = wis_score,
            wis_mod = modifier(wis_score),
            cha_score = cha_score,
            cha_mod = modifier(cha_score),
            saves_line = if saves.is_empty() {
                String::new()
            } else {
                format!("*Saves* {}\n", escape_typst(&saves))
            },
            senses_line = if senses.is_empty() {
                String::new()
            } else {
                format!("*Senses* {}\n", escape_typst(&senses))
            },
            languages_line = if languages.is_empty() {
                String::new()
            } else {
                format!("*Languages* {}", escape_typst(&languages))
            },
            resistances_block = render_resistances_block(&vulnerabilities, &resistances, &damage_immunities, &condition_immunities),
            front_sections_content = front_sections_content,
            source = escape_typst(source),
            fold_indicator = if layout.is_foldable { " ▶ continued" } else { "" },
        )
    }

    /// Render the back card for foldable monsters
    fn render_back_card(monster: &Value) -> Option<String> {
        let layout = plan_card_layout(monster);

        if !layout.is_foldable {
            return None;
        }

        let name = monster
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let source = monster
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let back_sections_content = render_sections(&layout.back_sections);

        Some(format!(
            r##"box(
  width: 3.875in,
  height: 5.125in,
  stroke: (
    top: 3pt + colors.accent,
    bottom: 3pt + colors.accent,
    left: 0.5pt + colors.border,
    right: 0.5pt + colors.border,
  ),
  radius: 3pt,
  clip: true,
  inset: 0pt,
)[
  // Header - continuation
  #block(
    width: 100%,
    fill: colors.background-alt,
    inset: (x: 6pt, y: 4pt),
  )[
    #text(size: 10pt, weight: "bold")[{name}]
    #h(1fr)
    #text(size: 7pt, style: "italic", fill: colors.text-secondary)[(continued)]
  ]

  // Back sections (legendary, lair, etc.)
  {back_sections_content}

  // Footer
  #place(
    bottom + left,
    block(
      width: 100%,
      fill: colors.background-alt,
      inset: (x: 6pt, y: 2pt),
    )[
      #text(size: 5pt, fill: colors.text-secondary)[◀ fold \| {source}]
    ]
  )
]"##,
            name = escape_typst(name),
            back_sections_content = back_sections_content,
            source = escape_typst(source),
        ))
    }

    /// Check if a monster needs a foldable card
    fn needs_foldable(monster: &Value) -> bool {
        plan_card_layout(monster).is_foldable
    }
}

impl Renderable for MonsterCardSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        if self.monsters.is_empty() {
            return Ok(String::new());
        }

        let mut typst = String::new();

        // Set page margins for half-page cards (centered with gutters for cutting)
        typst.push_str("#set page(paper: \"us-letter\", margin: 0.25in)\n");

        // Build list of card units (single cards or front+back pairs that must stay together)
        // Each unit is a Vec of 1-2 cards
        let mut card_units: Vec<Vec<String>> = Vec::new();
        for monster in &self.monsters {
            let front = Self::render_card(monster);
            if let Some(back) = Self::render_back_card(monster) {
                // Foldable: front and back must stay together
                card_units.push(vec![front, back]);
            } else {
                // Single card
                card_units.push(vec![front]);
            }
        }

        // Pack units into pages (4 cards per page), keeping pairs together
        let mut pages: Vec<Vec<String>> = Vec::new();
        let mut current_page: Vec<String> = Vec::new();

        for unit in card_units {
            let unit_size = unit.len();
            let space_left = 4 - current_page.len();

            // If unit won't fit on current page, start a new page
            if unit_size > space_left && !current_page.is_empty() {
                pages.push(current_page);
                current_page = Vec::new();
            }

            // Add all cards from this unit to current page
            for card in unit {
                current_page.push(card);
            }

            // If page is full, start a new one
            if current_page.len() >= 4 {
                pages.push(current_page);
                current_page = Vec::new();
            }
        }

        // Don't forget the last page
        if !current_page.is_empty() {
            pages.push(current_page);
        }

        for (page_num, page_cards) in pages.iter().enumerate() {

            if page_num > 0 {
                typst.push_str("\n#pagebreak()\n");
            }

            // Card grid (2x2) - cards sized to fit with gutters for cutting
            typst.push_str("#grid(\n");
            typst.push_str("    columns: (3.875in,) * 2,\n");
            typst.push_str("    rows: (5.125in,) * 2,\n");
            typst.push_str("    column-gutter: 0.25in,\n");
            typst.push_str("    row-gutter: 0.25in,\n\n");

            // Render each card
            for (i, card) in page_cards.iter().enumerate() {
                typst.push_str("    ");
                typst.push_str(card);
                if i < page_cards.len() - 1 || page_cards.len() < 4 {
                    typst.push(',');
                }
                typst.push('\n');
            }

            // Fill remaining slots with empty boxes
            for _ in page_cards.len()..4 {
                typst.push_str("    box(width: 3.875in, height: 5.125in),\n");
            }

            typst.push_str(")\n");

            // Cut lines indicator
            if self.show_cut_lines && !page_cards.is_empty() {
                typst.push_str("#place(\n  bottom + center,\n  dy: 0.1in,\n");
                typst.push_str(
                    "  text(size: 6pt, fill: colors.text-secondary)[Cut along card borders - fold adjacent cards for extended stat blocks]\n)\n",
                );
            }
        }

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        if self.monsters.is_empty() {
            None
        } else {
            Some("Monster Cards".to_string())
        }
    }
}

// === Helper Functions ===

fn extract_creature_type(monster: &Value) -> String {
    monster
        .get("type")
        .or_else(|| monster.get("creature_type"))
        .map(|v| {
            if let Some(s) = v.as_str() {
                s.to_string()
            } else if let Some(obj) = v.as_object() {
                obj.get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("creature")
                    .to_string()
            } else {
                "creature".to_string()
            }
        })
        .unwrap_or_else(|| "creature".to_string())
}

fn extract_alignment(monster: &Value) -> String {
    monster
        .get("alignment")
        .map(|align| {
            if let Some(s) = align.as_str() {
                s.to_string()
            } else if let Some(arr) = align.as_array() {
                arr.iter()
                    .filter_map(|a| {
                        a.as_str().map(|s| match s {
                            "L" => "lawful",
                            "N" | "NX" | "NY" => "neutral",
                            "C" => "chaotic",
                            "G" => "good",
                            "E" => "evil",
                            "U" => "unaligned",
                            "A" => "any",
                            _ => s,
                        })
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                "unaligned".to_string()
            }
        })
        .unwrap_or_else(|| "unaligned".to_string())
}

fn extract_ac_full(monster: &Value) -> String {
    monster
        .get("ac")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .map(|ac| {
            if let Some(n) = ac.as_i64() {
                n.to_string()
            } else if let Some(obj) = ac.as_object() {
                let ac_val = obj.get("ac").and_then(|v| v.as_i64()).unwrap_or(10);
                if let Some(from) = obj.get("from").and_then(|v| v.as_array()) {
                    let sources: Vec<_> = from
                        .iter()
                        .filter_map(|f| f.as_str())
                        .collect();
                    if !sources.is_empty() {
                        return format!("{} ({})", ac_val, sources.join(", "));
                    }
                }
                ac_val.to_string()
            } else {
                "10".to_string()
            }
        })
        .unwrap_or_else(|| "10".to_string())
}

fn extract_hp_full(monster: &Value) -> String {
    monster
        .get("hp")
        .map(|hp_val| {
            if let Some(n) = hp_val.as_i64() {
                n.to_string()
            } else if let Some(obj) = hp_val.as_object() {
                let avg = obj.get("average").and_then(|v| v.as_i64()).unwrap_or(1);
                if let Some(formula) = obj.get("formula").and_then(|v| v.as_str()) {
                    format!("{} ({})", avg, formula)
                } else {
                    avg.to_string()
                }
            } else {
                "1".to_string()
            }
        })
        .unwrap_or_else(|| "1".to_string())
}

/// Extract just the numeric HP value for tracker calculations
fn extract_hp_average(monster: &Value) -> i64 {
    monster
        .get("hp")
        .map(|hp_val| {
            if let Some(n) = hp_val.as_i64() {
                n
            } else if let Some(obj) = hp_val.as_object() {
                obj.get("average").and_then(|v| v.as_i64()).unwrap_or(1)
            } else {
                1
            }
        })
        .unwrap_or(1)
}

// === HP Tracker Functions ===

/// Render HP tracker based on HP value
fn render_hp_tracker(hp: i64) -> String {
    match hp {
        1..=20 => render_individual_boxes(hp),
        21..=100 => render_fives_and_ones(hp),
        _ => render_tens_and_ones(hp),
    }
}

/// Render individual 1-HP boxes (for 1-20 HP)
/// Every 10th box is styled differently
fn render_individual_boxes(hp: i64) -> String {
    let mut boxes = Vec::new();
    for i in 1..=hp {
        if i % 10 == 0 {
            // Styled 10th box - thicker border, slight fill
            boxes.push(r##"#box(width: 7pt, height: 7pt, stroke: 1pt + black, fill: rgb("#e5e5e5"))"##.to_string());
        } else {
            // Regular 1-HP box
            boxes.push(r##"#box(width: 6pt, height: 6pt, stroke: 0.5pt + black)"##.to_string());
        }
    }
    boxes.join("#h(1pt)")
}

/// Render 5-HP boxes plus 1-HP boxes (for 21-100 HP)
fn render_fives_and_ones(hp: i64) -> String {
    let fives = hp / 5;
    let ones = hp % 5;

    let mut parts = Vec::new();

    // 5-HP boxes with "5" label
    for i in 0..fives {
        // Every other 5-box (at 10, 20, 30...) gets extra styling
        if (i + 1) % 2 == 0 {
            parts.push(r##"#box(width: 10pt, height: 10pt, stroke: 1pt + black, fill: rgb("#d4d4d4"))[#align(center + horizon)[#text(size: 5pt, weight: "bold")[5]]]"##.to_string());
        } else {
            parts.push(r##"#box(width: 10pt, height: 10pt, stroke: 0.75pt + black, fill: rgb("#f5f5f5"))[#align(center + horizon)[#text(size: 5pt)[5]]]"##.to_string());
        }
    }

    // Add spacing before 1-HP boxes if we have both
    if fives > 0 && ones > 0 {
        parts.push("#h(3pt)".to_string());
    }

    // 1-HP boxes
    for _ in 0..ones {
        parts.push(r##"#box(width: 6pt, height: 6pt, stroke: 0.5pt + black)"##.to_string());
    }

    parts.join("#h(1pt)")
}

/// Render 10-HP boxes plus 1-HP boxes (for 101+ HP)
fn render_tens_and_ones(hp: i64) -> String {
    let tens = hp / 10;
    let ones = hp % 10;

    let mut parts = Vec::new();

    // 10-HP boxes with "10" label
    for i in 0..tens {
        // Every 5th ten-box (at 50, 100, 150...) gets extra styling
        if (i + 1) % 5 == 0 {
            parts.push(r##"#box(width: 12pt, height: 10pt, stroke: 1.5pt + black, fill: rgb("#c4c4c4"))[#align(center + horizon)[#text(size: 5pt, weight: "bold")[10]]]"##.to_string());
        } else {
            parts.push(r##"#box(width: 12pt, height: 10pt, stroke: 0.75pt + black, fill: rgb("#e5e5e5"))[#align(center + horizon)[#text(size: 5pt)[10]]]"##.to_string());
        }
    }

    // Add spacing before 1-HP boxes if we have both
    if tens > 0 && ones > 0 {
        parts.push("#h(3pt)".to_string());
    }

    // 1-HP boxes
    for _ in 0..ones {
        parts.push(r##"#box(width: 6pt, height: 6pt, stroke: 0.5pt + black)"##.to_string());
    }

    parts.join("#h(1pt)")
}

fn extract_speed_full(monster: &Value) -> String {
    monster
        .get("speed")
        .map(|s| {
            if let Some(obj) = s.as_object() {
                let mut parts = Vec::new();

                if let Some(walk) = obj.get("walk").and_then(|v| v.as_i64()) {
                    parts.push(format!("{} ft.", walk));
                }
                if let Some(fly) = obj.get("fly").and_then(|v| v.as_i64()) {
                    let hover = obj.get("hover").and_then(|v| v.as_bool()).unwrap_or(false)
                        || obj.get("canHover").and_then(|v| v.as_bool()).unwrap_or(false);
                    if hover {
                        parts.push(format!("fly {} ft. (hover)", fly));
                    } else {
                        parts.push(format!("fly {} ft.", fly));
                    }
                }
                if let Some(swim) = obj.get("swim").and_then(|v| v.as_i64()) {
                    parts.push(format!("swim {} ft.", swim));
                }
                if let Some(climb) = obj.get("climb").and_then(|v| v.as_i64()) {
                    parts.push(format!("climb {} ft.", climb));
                }
                if let Some(burrow) = obj.get("burrow").and_then(|v| v.as_i64()) {
                    parts.push(format!("burrow {} ft.", burrow));
                }

                if parts.is_empty() {
                    "30 ft.".to_string()
                } else {
                    parts.join(", ")
                }
            } else {
                "30 ft.".to_string()
            }
        })
        .unwrap_or_else(|| "30 ft.".to_string())
}

fn extract_cr(monster: &Value) -> String {
    monster
        .get("cr")
        .map(|cr_val| {
            if let Some(s) = cr_val.as_str() {
                s.to_string()
            } else if let Some(obj) = cr_val.as_object() {
                obj.get("cr")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0")
                    .to_string()
            } else if let Some(n) = cr_val.as_i64() {
                n.to_string()
            } else {
                "0".to_string()
            }
        })
        .unwrap_or_else(|| "0".to_string())
}

fn extract_saves(monster: &Value) -> String {
    let mut saves = Vec::new();

    if let Some(save_obj) = monster.get("save").and_then(|v| v.as_object()) {
        for (key, val) in save_obj {
            if let Some(bonus) = val.as_str() {
                let abbrev = match key.to_lowercase().as_str() {
                    "str" => "Str",
                    "dex" => "Dex",
                    "con" => "Con",
                    "int" => "Int",
                    "wis" => "Wis",
                    "cha" => "Cha",
                    _ => continue,
                };
                saves.push(format!("{} {}", abbrev, bonus));
            }
        }
    }

    saves.join(", ")
}

fn extract_senses(monster: &Value) -> String {
    let mut parts = Vec::new();

    if let Some(senses) = monster.get("senses").and_then(|v| v.as_array()) {
        for sense in senses {
            if let Some(s) = sense.as_str() {
                parts.push(s.to_string());
            }
        }
    }

    parts.join(", ")
}

fn extract_languages(monster: &Value) -> String {
    if let Some(langs) = monster.get("languages").and_then(|v| v.as_array()) {
        langs
            .iter()
            .filter_map(|l| l.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        String::new()
    }
}

fn extract_damage_list(monster: &Value, snake_key: &str, camel_key: &str) -> String {
    monster
        .get(snake_key)
        .or_else(|| monster.get(camel_key))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|d| d.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}

fn extract_condition_immunities(monster: &Value) -> String {
    monster
        .get("condition_immunities")
        .or_else(|| monster.get("conditionImmune"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| {
                    if let Some(s) = c.as_str() {
                        Some(s.to_string())
                    } else if let Some(obj) = c.as_object() {
                        obj.get("conditionImmune")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}

fn render_resistances_block(vuln: &str, resist: &str, dmg_immune: &str, cond_immune: &str) -> String {
    let mut lines = Vec::new();

    if !vuln.is_empty() {
        lines.push(format!("*Vuln* {}", escape_typst(vuln)));
    }
    if !resist.is_empty() {
        lines.push(format!("*Resist* {}", escape_typst(resist)));
    }
    if !dmg_immune.is_empty() {
        lines.push(format!("*Immune* {}", escape_typst(dmg_immune)));
    }
    if !cond_immune.is_empty() {
        lines.push(format!("*Cond Immune* {}", escape_typst(cond_immune)));
    }

    if lines.is_empty() {
        String::new()
    } else {
        format!(
            r#"#block(
    width: 100%,
    inset: (x: 6pt, y: 3pt),
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: 6pt)
    {}
  ]"#,
            lines.join("\n    ")
        )
    }
}

/// Section content with its rendered size
struct SectionContent {
    name: &'static str,
    content: String,
    char_count: usize,
}

/// Card layout - single or foldable double
struct CardLayout {
    front_sections: Vec<SectionContent>,
    back_sections: Vec<SectionContent>,
    is_foldable: bool,
}

/// Extract all abilities and determine card layout (single or foldable)
fn plan_card_layout(monster: &Value) -> CardLayout {
    // Single card budget for ability sections (after header, stats, abilities row)
    const SINGLE_CARD_BUDGET: usize = 1200;

    let mut sections = Vec::new();

    // Extract traits
    let traits_raw = extract_ability_list(monster, "trait");
    if !traits_raw.is_empty() {
        let content = format_abilities_full(&traits_raw);
        let char_count = content.len() + 20; // header overhead
        sections.push(SectionContent {
            name: "Traits",
            content,
            char_count,
        });
    }

    // Extract actions
    let actions_raw = extract_ability_list(monster, "action");
    if !actions_raw.is_empty() {
        let content = format_abilities_full(&actions_raw);
        let char_count = content.len() + 20;
        sections.push(SectionContent {
            name: "Actions",
            content,
            char_count,
        });
    }

    // Extract reactions
    let reactions_raw = extract_ability_list(monster, "reaction");
    if !reactions_raw.is_empty() {
        let content = format_abilities_full(&reactions_raw);
        let char_count = content.len() + 20;
        sections.push(SectionContent {
            name: "Reactions",
            content,
            char_count,
        });
    }

    // Extract legendary actions
    let legendary_raw = extract_ability_list(monster, "legendary");
    if !legendary_raw.is_empty() {
        let content = format_abilities_full(&legendary_raw);
        let char_count = content.len() + 20;
        sections.push(SectionContent {
            name: "Legendary",
            content,
            char_count,
        });
    }

    // Extract lair actions
    let lair_raw = extract_ability_list(monster, "legendaryGroup");
    if !lair_raw.is_empty() {
        let content = format_abilities_full(&lair_raw);
        let char_count = content.len() + 20;
        sections.push(SectionContent {
            name: "Lair",
            content,
            char_count,
        });
    }

    // Calculate total content size
    let total_size: usize = sections.iter().map(|s| s.char_count).sum();

    // If everything fits, single card
    if total_size <= SINGLE_CARD_BUDGET {
        return CardLayout {
            front_sections: sections,
            back_sections: Vec::new(),
            is_foldable: false,
        };
    }

    // Need foldable card - find break point at section boundary
    let mut front_sections = Vec::new();
    let mut back_sections = Vec::new();
    let mut front_used = 0;

    for section in sections {
        if front_used + section.char_count <= SINGLE_CARD_BUDGET {
            front_used += section.char_count;
            front_sections.push(section);
        } else {
            back_sections.push(section);
        }
    }

    let is_foldable = !back_sections.is_empty();
    CardLayout {
        front_sections,
        back_sections,
        is_foldable,
    }
}

/// Extract ability list from monster data
fn extract_ability_list(monster: &Value, key: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();

    if let Some(arr) = monster.get(key).and_then(|v| v.as_array()) {
        for item in arr {
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                let desc = extract_entries_text(item.get("entries"));
                let cleaned = strip_5etools_tags(&desc);
                result.push((name.to_string(), cleaned));
            }
        }
    }

    result
}

/// Format abilities without truncation
fn format_abilities_full(items: &[(String, String)]) -> String {
    if items.is_empty() {
        return String::new();
    }

    let mut result = Vec::new();
    for (i, (name, desc)) in items.iter().enumerate() {
        if i > 0 {
            result.push(String::from("#v(2pt)"));
        }
        result.push(format!("*{}.* {}", escape_typst(name), escape_typst(desc)));
    }

    result.join("\n")
}

/// Render sections into Typst blocks
fn render_sections(sections: &[SectionContent]) -> String {
    let mut result = Vec::new();

    for section in sections {
        let color = match section.name {
            "Traits" => r##"rgb("#7c3aed")"##,
            "Actions" => "colors.accent",
            "Reactions" => r##"rgb("#0891b2")"##,
            "Legendary" => r##"rgb("#ca8a04")"##,
            "Lair" => r##"rgb("#059669")"##,
            _ => "colors.accent",
        };

        let label = match section.name {
            "Legendary" => "Legendary Actions",
            "Lair" => "Lair Actions",
            other => other,
        };

        result.push(format!(
            r##"#block(
    width: 100%,
    inset: (x: 6pt, y: 3pt),
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: 6pt)
    #text(size: 7pt, weight: "bold", fill: {})[{}]
    #v(2pt)
    {}
  ]"##,
            color, label, section.content
        ));
    }

    result.join("\n\n")
}

/// Extract text from entries array, handling nested structures
fn extract_entries_text(entries: Option<&Value>) -> String {
    let Some(entries) = entries else {
        return String::new();
    };

    let Some(arr) = entries.as_array() else {
        return String::new();
    };

    let mut parts = Vec::new();
    for entry in arr {
        if let Some(text) = entry.as_str() {
            parts.push(text.to_string());
        } else if let Some(obj) = entry.as_object() {
            // Handle nested entries
            if let Some(nested) = obj.get("entries").and_then(|v| v.as_array()) {
                for ne in nested {
                    if let Some(text) = ne.as_str() {
                        parts.push(text.to_string());
                    }
                }
            }
            // Handle items in lists
            if let Some(items) = obj.get("items").and_then(|v| v.as_array()) {
                for item in items {
                    if let Some(text) = item.as_str() {
                        parts.push(format!("• {}", text));
                    }
                }
            }
        }
    }

    parts.join(" ")
}

/// Format abilities with budget constraint
fn format_abilities(items: &[(String, String)], budget: usize) -> String {
    if items.is_empty() || budget == 0 {
        return String::new();
    }

    let mut result = Vec::new();
    let mut used = 0;

    for (i, (name, desc)) in items.iter().enumerate() {
        let entry_overhead = 15; // formatting chars
        let name_len = name.len();

        // Calculate how much space we have for this entry's description
        let available = if budget == usize::MAX {
            usize::MAX
        } else {
            budget.saturating_sub(used).saturating_sub(name_len + entry_overhead)
        };

        if available < 20 {
            break; // Not enough space for meaningful content
        }

        let truncated_desc = if desc.len() <= available {
            desc.clone()
        } else {
            truncate_text(desc, available)
        };

        if i > 0 {
            result.push(String::from("#v(2pt)"));
        }
        result.push(format!("*{}.* {}", escape_typst(name), escape_typst(&truncated_desc)));

        used += name_len + truncated_desc.len() + entry_overhead;
    }

    result.join("\n")
}


/// Strip 5etools formatting tags and convert to plain text
fn strip_5etools_tags(text: &str) -> String {
    let mut result = text.to_string();

    // {@atk mw} -> Melee Weapon Attack:
    result = regex_replace(&result, r"\{@atk mw\}", "Melee Weapon Attack:");
    result = regex_replace(&result, r"\{@atk rw\}", "Ranged Weapon Attack:");
    result = regex_replace(&result, r"\{@atk ms\}", "Melee Spell Attack:");
    result = regex_replace(&result, r"\{@atk rs\}", "Ranged Spell Attack:");
    result = regex_replace(&result, r"\{@atk mw,rw\}", "Melee or Ranged Weapon Attack:");

    // {@hit N} -> +N
    result = regex_replace(&result, r"\{@hit (\d+)\}", "+$1");

    // {@damage XdY+Z} -> XdY+Z
    result = regex_replace(&result, r"\{@damage ([^}]+)\}", "$1");

    // {@dice XdY} -> XdY
    result = regex_replace(&result, r"\{@dice ([^}]+)\}", "$1");

    // {@dc N} -> DC N
    result = regex_replace(&result, r"\{@dc (\d+)\}", "DC $1");

    // {@condition X} -> X
    result = regex_replace(&result, r"\{@condition ([^|}]+)[^}]*\}", "$1");

    // {@creature X} -> X
    result = regex_replace(&result, r"\{@creature ([^|}]+)[^}]*\}", "$1");

    // Generic tag removal
    result = regex_replace(&result, r"\{@\w+ ([^|}]+)[^}]*\}", "$1");

    result
}

fn regex_replace(text: &str, pattern: &str, replacement: &str) -> String {
    if let Ok(re) = regex::Regex::new(pattern) {
        re.replace_all(text, replacement).to_string()
    } else {
        text.to_string()
    }
}

fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
}

fn modifier(score: i64) -> String {
    let m = (score - 10) / 2;
    if m >= 0 {
        format!("+{}", m)
    } else {
        format!("{}", m)
    }
}

fn escape_typst(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('"', "\\\"")
        .replace('_', "\\_")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('@', "\\@")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_monster_cards_empty() {
        let section = MonsterCardSection::new(vec![]);
        assert!(section.toc_title().is_none());
    }

    #[test]
    fn test_monster_cards_with_monsters() {
        let monsters = vec![json!({
            "name": "Goblin",
            "cr": "1/4",
            "size": ["S"],
            "type": "humanoid"
        })];
        let section = MonsterCardSection::new(monsters);
        assert_eq!(section.toc_title(), Some("Monster Cards".to_string()));
    }

    #[test]
    fn test_from_json() {
        let data = json!([
            {"name": "Goblin", "cr": "1/4"},
            {"name": "Orc", "cr": "1/2"}
        ]);
        let section = MonsterCardSection::from_json(data);
        assert_eq!(section.monsters.len(), 2);
    }

    #[test]
    fn test_modifier() {
        assert_eq!(modifier(10), "+0");
        assert_eq!(modifier(18), "+4");
        assert_eq!(modifier(8), "-1");
    }

    #[test]
    fn test_extract_alignment() {
        let monster = json!({"alignment": ["L", "G"]});
        assert_eq!(extract_alignment(&monster), "lawful good");
    }

    #[test]
    fn test_strip_5etools_tags() {
        assert_eq!(strip_5etools_tags("{@atk mw}"), "Melee Weapon Attack:");
        assert_eq!(strip_5etools_tags("{@hit 5}"), "+5");
        assert_eq!(strip_5etools_tags("{@damage 1d6+3}"), "1d6+3");
        assert_eq!(strip_5etools_tags("{@dc 13}"), "DC 13");
    }

    #[test]
    fn test_extract_hp_average() {
        // Simple number
        let monster = json!({"hp": 42});
        assert_eq!(extract_hp_average(&monster), 42);

        // Object with average
        let monster = json!({"hp": {"average": 59, "formula": "7d10+21"}});
        assert_eq!(extract_hp_average(&monster), 59);

        // Missing HP defaults to 1
        let monster = json!({"name": "Test"});
        assert_eq!(extract_hp_average(&monster), 1);
    }

    #[test]
    fn test_render_individual_boxes() {
        // 7 HP should produce 7 boxes
        let result = render_individual_boxes(7);
        assert!(result.contains("#box(width: 6pt"));
        assert_eq!(result.matches("#box").count(), 7);

        // 10 HP should have a styled 10th box
        let result = render_individual_boxes(10);
        assert!(result.contains("width: 7pt")); // styled box
        assert_eq!(result.matches("#box").count(), 10);

        // 20 HP should have two styled boxes (at 10 and 20)
        let result = render_individual_boxes(20);
        assert_eq!(result.matches("width: 7pt").count(), 2);
    }

    #[test]
    fn test_render_fives_and_ones() {
        // 25 HP = 5 five-boxes, 0 ones
        let result = render_fives_and_ones(25);
        assert!(result.contains("[5]"));
        assert_eq!(result.matches("[5]").count(), 5);

        // 59 HP = 11 five-boxes + 4 one-boxes
        let result = render_fives_and_ones(59);
        assert_eq!(result.matches("[5]").count(), 11);
        assert!(result.contains("width: 6pt")); // 1-HP boxes
    }

    #[test]
    fn test_render_tens_and_ones() {
        // 100 HP = 10 ten-boxes, 0 ones
        let result = render_tens_and_ones(100);
        assert!(result.contains("[10]"));
        assert_eq!(result.matches("[10]").count(), 10);

        // 125 HP = 12 ten-boxes + 5 one-boxes
        let result = render_tens_and_ones(125);
        assert_eq!(result.matches("[10]").count(), 12);
        assert!(result.contains("width: 6pt")); // 1-HP boxes
    }

    #[test]
    fn test_render_hp_tracker_tier_selection() {
        // Low HP (1-20) uses individual boxes
        let result = render_hp_tracker(7);
        assert!(!result.contains("[5]"));
        assert!(!result.contains("[10]"));

        // Medium HP (21-100) uses 5s and 1s
        let result = render_hp_tracker(59);
        assert!(result.contains("[5]"));
        assert!(!result.contains("[10]"));

        // High HP (101+) uses 10s and 1s
        let result = render_hp_tracker(256);
        assert!(result.contains("[10]"));
        assert!(!result.contains("[5]"));
    }

    #[test]
    fn test_render_card_with_hp_tracker() {
        use crate::builder::{RenderContext, Renderable};

        // Create monsters with different HP values to test all tiers
        let monsters = vec![
            json!({
                "name": "Goblin",
                "hp": {"average": 7, "formula": "2d6"},
                "ac": [{"ac": 15, "from": ["leather armor", "shield"]}],
                "cr": "1/4",
                "size": ["S"],
                "type": "humanoid",
                "alignment": ["N", "E"],
                "speed": {"walk": 30},
                "str": 8, "dex": 14, "con": 10, "int": 10, "wis": 8, "cha": 8,
                "passive": 9,
                "source": "MM"
            }),
            json!({
                "name": "Ogre",
                "hp": {"average": 59, "formula": "7d10+21"},
                "ac": [{"ac": 11, "from": ["hide armor"]}],
                "cr": "2",
                "size": ["L"],
                "type": "giant",
                "alignment": ["C", "E"],
                "speed": {"walk": 40},
                "str": 19, "dex": 8, "con": 16, "int": 5, "wis": 7, "cha": 7,
                "passive": 8,
                "source": "MM"
            }),
            json!({
                "name": "Adult Red Dragon",
                "hp": {"average": 256, "formula": "19d12+133"},
                "ac": [{"ac": 19, "from": ["natural armor"]}],
                "cr": "17",
                "size": ["H"],
                "type": "dragon",
                "alignment": ["C", "E"],
                "speed": {"walk": 40, "climb": 40, "fly": 80},
                "str": 27, "dex": 10, "con": 25, "int": 16, "wis": 13, "cha": 21,
                "passive": 23,
                "source": "MM"
            }),
            json!({
                "name": "Tarrasque",
                "hp": {"average": 676, "formula": "33d20+330"},
                "ac": [{"ac": 25, "from": ["natural armor"]}],
                "cr": "30",
                "size": ["G"],
                "type": "monstrosity",
                "alignment": ["U"],
                "speed": {"walk": 40},
                "str": 30, "dex": 11, "con": 30, "int": 3, "wis": 11, "cha": 11,
                "passive": 10,
                "source": "MM"
            }),
        ];

        let section = MonsterCardSection::new(monsters);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        // Verify HP trackers are present for each monster
        assert!(typst.contains("#box(width: 6pt")); // 1-HP boxes for Goblin
        assert!(typst.contains("[5]")); // 5-HP boxes for Ogre
        assert!(typst.contains("[10]")); // 10-HP boxes for Dragon/Tarrasque

        // Output the Typst for manual inspection if needed
        // println!("{}", typst);
    }
}
