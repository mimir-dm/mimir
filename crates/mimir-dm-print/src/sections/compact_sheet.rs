//! Compact character sheet section (2-page WotC-style layout)
//!
//! Page 1: Combat & Stats - ability scores, saves, skills, attacks, spell slots
//! Page 2: Character Details - portrait, personality, backstory, features

use mimir_dm_core::models::catalog::Spell;
use mimir_dm_core::models::character::data::{AbilityScores, CharacterData};

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// Compact 2-page character sheet section
pub struct CompactSheetSection {
    character: CharacterData,
    spell_details: Vec<Spell>,
}

impl CompactSheetSection {
    pub fn new(character: CharacterData) -> Self {
        Self {
            character,
            spell_details: Vec::new(),
        }
    }

    pub fn with_spells(mut self, spells: Vec<Spell>) -> Self {
        self.spell_details = spells;
        self
    }

    fn build_class_string(&self) -> String {
        if self.character.classes.is_empty() {
            return "No Class".to_string();
        }

        self.character
            .classes
            .iter()
            .map(|c| {
                if let Some(ref sub) = c.subclass {
                    format!("{} ({}) {}", c.class_name, sub, c.level)
                } else {
                    format!("{} {}", c.class_name, c.level)
                }
            })
            .collect::<Vec<_>>()
            .join(" / ")
    }

    /// Calculate total spell slots per level based on class levels
    fn get_spell_slots(&self) -> [u8; 9] {
        // Simplified slot calculation - full caster progression
        // A proper implementation would check class spell slot tables
        let caster_level = self.character.level;
        match caster_level {
            0 => [0, 0, 0, 0, 0, 0, 0, 0, 0],
            1 => [2, 0, 0, 0, 0, 0, 0, 0, 0],
            2 => [3, 0, 0, 0, 0, 0, 0, 0, 0],
            3 => [4, 2, 0, 0, 0, 0, 0, 0, 0],
            4 => [4, 3, 0, 0, 0, 0, 0, 0, 0],
            5 => [4, 3, 2, 0, 0, 0, 0, 0, 0],
            6 => [4, 3, 3, 0, 0, 0, 0, 0, 0],
            7 => [4, 3, 3, 1, 0, 0, 0, 0, 0],
            8 => [4, 3, 3, 2, 0, 0, 0, 0, 0],
            9 => [4, 3, 3, 3, 1, 0, 0, 0, 0],
            10 => [4, 3, 3, 3, 2, 0, 0, 0, 0],
            11 => [4, 3, 3, 3, 2, 1, 0, 0, 0],
            12 => [4, 3, 3, 3, 2, 1, 0, 0, 0],
            13 => [4, 3, 3, 3, 2, 1, 1, 0, 0],
            14 => [4, 3, 3, 3, 2, 1, 1, 0, 0],
            15 => [4, 3, 3, 3, 2, 1, 1, 1, 0],
            16 => [4, 3, 3, 3, 2, 1, 1, 1, 0],
            17 => [4, 3, 3, 3, 2, 1, 1, 1, 1],
            18 => [4, 3, 3, 3, 3, 1, 1, 1, 1],
            19 => [4, 3, 3, 3, 3, 2, 1, 1, 1],
            _ => [4, 3, 3, 3, 3, 2, 2, 1, 1],
        }
    }

    fn has_spellcasting(&self) -> bool {
        !self.character.spells.cantrips.is_empty()
            || !self.character.spells.prepared_spells.is_empty()
            || !self.character.spells.known_spells.is_empty()
    }

    fn render_page1(&self) -> String {
        let c = &self.character;
        let abilities = &c.abilities;

        let race_str = if let Some(ref sub) = c.subrace {
            format!("{} {}", sub, c.race)
        } else {
            c.race.clone()
        };
        let class_str = self.build_class_string();
        let prof_bonus = c.proficiency_bonus();

        // Ability modifiers
        let str_mod = AbilityScores::modifier(abilities.strength);
        let dex_mod = AbilityScores::modifier(abilities.dexterity);
        let con_mod = AbilityScores::modifier(abilities.constitution);
        let int_mod = AbilityScores::modifier(abilities.intelligence);
        let wis_mod = AbilityScores::modifier(abilities.wisdom);
        let cha_mod = AbilityScores::modifier(abilities.charisma);

        // Calculate AC (simplified - base 10 + DEX)
        let ac = 10 + dex_mod;

        let mut typst = String::new();

        // Page setup for letter size
        typst.push_str(r#"#set page(paper: "us-letter", margin: (x: 0.5in, y: 0.5in))
"#);

        // ==== HEADER ====
        typst.push_str(&format!(
            r#"#grid(
  columns: (1fr, auto, auto),
  row-gutter: 4pt,
  column-gutter: 12pt,
  [#text(size: 18pt, weight: "bold")[{}]], [#text(size: 10pt)[Class & Level]], [#text(size: 10pt)[Background]],
  [], [{}], [{}],
)

#v(8pt)
#grid(
  columns: (1fr, auto, auto, auto),
  column-gutter: 12pt,
  [#text(size: 9pt, fill: luma(100))[Player: \_\_\_\_\_\_\_\_\_\_\_\_\_]],
  [#text(size: 9pt)[Race: {}]],
  [#text(size: 9pt)[Alignment: {}]],
  [#text(size: 9pt)[XP: {}]],
)

#line(length: 100%, stroke: 1pt + colors.border)
#v(8pt)

"#,
            escape_typst(&c.character_name),
            escape_typst(&class_str),
            escape_typst(&c.background),
            escape_typst(&race_str),
            c.alignment.as_deref().unwrap_or("—"),
            c.experience_points
        ));

        // ==== MAIN CONTENT: 3-column layout ====
        typst.push_str(r#"#grid(
  columns: (70pt, 1fr, 1fr),
  column-gutter: 8pt,

  // LEFT COLUMN - Ability Scores
  {
"#);

        // Ability scores column
        for (name, score, modifier) in [
            ("STR", abilities.strength, str_mod),
            ("DEX", abilities.dexterity, dex_mod),
            ("CON", abilities.constitution, con_mod),
            ("INT", abilities.intelligence, int_mod),
            ("WIS", abilities.wisdom, wis_mod),
            ("CHA", abilities.charisma, cha_mod),
        ] {
            let sign = if modifier >= 0 { "+" } else { "" };
            typst.push_str(&format!(
                r#"    box(
      width: 100%,
      stroke: 1pt + colors.border,
      radius: 3pt,
      inset: 4pt,
      {{
        align(center)[
          #text(size: 8pt, weight: "bold")[{}]
          #v(2pt)
          #text(size: 14pt)[{}]
          #v(1pt)
          #box(
            fill: colors.accent,
            radius: 2pt,
            inset: (x: 6pt, y: 2pt),
            text(fill: white, weight: "bold", size: 9pt)[{}{}]
          )
        ]
      }}
    )
    v(4pt)
"#,
                name, score, sign, modifier
            ));
        }

        typst.push_str("  },\n\n  // MIDDLE COLUMN\n  {\n");

        // Middle column - Inspiration, Prof Bonus, Saves, Skills
        typst.push_str(&format!(
            r#"    // Inspiration & Prof
    grid(
      columns: (1fr, 1fr),
      column-gutter: 8pt,
      box(stroke: 1pt + colors.border, radius: 4pt, inset: 6pt)[
        #text(size: 8pt)[INSPIRATION]
        #h(1fr)
        #box(width: 16pt, height: 16pt, stroke: 1pt + luma(150))
      ],
      box(stroke: 1pt + colors.border, radius: 4pt, inset: 6pt)[
        #text(size: 8pt)[PROFICIENCY]
        #h(1fr)
        #text(size: 14pt, weight: "bold")[+{}]
      ],
    )
    v(8pt)

    // Saving Throws
    box(
      width: 100%,
      stroke: 1pt + colors.border,
      radius: 4pt,
      inset: 8pt,
    )[
      #text(size: 9pt, weight: "bold")[SAVING THROWS]
      #v(4pt)
"#,
            prof_bonus
        ));

        // Saving throws
        let prof_saves = &c.proficiencies.saves;
        for (abbrev, full, score) in [
            ("STR", "Strength", abilities.strength),
            ("DEX", "Dexterity", abilities.dexterity),
            ("CON", "Constitution", abilities.constitution),
            ("INT", "Intelligence", abilities.intelligence),
            ("WIS", "Wisdom", abilities.wisdom),
            ("CHA", "Charisma", abilities.charisma),
        ] {
            let modifier = AbilityScores::modifier(score);
            let is_prof = prof_saves.iter().any(|s| s == full);
            let total = modifier + if is_prof { prof_bonus } else { 0 };
            let sign = if total >= 0 { "+" } else { "" };
            let circle = if is_prof { "●" } else { "○" };

            typst.push_str(&format!(
                "      #text(size: 8pt)[{} {} {}{}]#linebreak()\n",
                circle, abbrev, sign, total
            ));
        }

        typst.push_str("    ]\n    v(8pt)\n\n    // Skills\n    box(\n      width: 100%,\n      stroke: 1pt + colors.border,\n      radius: 4pt,\n      inset: 8pt,\n    )[\n      #text(size: 9pt, weight: \"bold\")[SKILLS]\n      #v(4pt)\n");

        // Skills
        let prof_skills = &c.proficiencies.skills;
        let skills = [
            ("Acrobatics", "Dexterity", dex_mod),
            ("Animal Handling", "Wisdom", wis_mod),
            ("Arcana", "Intelligence", int_mod),
            ("Athletics", "Strength", str_mod),
            ("Deception", "Charisma", cha_mod),
            ("History", "Intelligence", int_mod),
            ("Insight", "Wisdom", wis_mod),
            ("Intimidation", "Charisma", cha_mod),
            ("Investigation", "Intelligence", int_mod),
            ("Medicine", "Wisdom", wis_mod),
            ("Nature", "Intelligence", int_mod),
            ("Perception", "Wisdom", wis_mod),
            ("Performance", "Charisma", cha_mod),
            ("Persuasion", "Charisma", cha_mod),
            ("Religion", "Intelligence", int_mod),
            ("Sleight of Hand", "Dexterity", dex_mod),
            ("Stealth", "Dexterity", dex_mod),
            ("Survival", "Wisdom", wis_mod),
        ];

        for (skill, _ability, base_mod) in skills {
            let is_prof = prof_skills.iter().any(|s| s == skill);
            let total = base_mod + if is_prof { prof_bonus } else { 0 };
            let sign = if total >= 0 { "+" } else { "" };
            let circle = if is_prof { "●" } else { "○" };

            typst.push_str(&format!(
                "      #text(size: 7pt)[{} {}{}#h(1fr){}]#linebreak()\n",
                circle, sign, total, skill
            ));
        }

        // Passive Perception
        let passive_perception = 10 + wis_mod + if prof_skills.iter().any(|s| s == "Perception") { prof_bonus } else { 0 };

        typst.push_str(&format!(
            r#"    ]
    v(8pt)

    // Passive Perception
    box(
      width: 100%,
      stroke: 1pt + colors.border,
      radius: 4pt,
      inset: 8pt,
    )[
      #text(size: 8pt)[PASSIVE PERCEPTION]
      #h(1fr)
      #text(size: 14pt, weight: "bold")[{}]
    ]
  }},

  // RIGHT COLUMN
  {{
"#,
            passive_perception
        ));

        // Right column - Combat stats
        typst.push_str(&format!(
            r#"    // AC, Initiative, Speed
    grid(
      columns: (1fr, 1fr, 1fr),
      column-gutter: 4pt,
      box(stroke: 1pt + colors.border, radius: 4pt, inset: 6pt, align(center)[
        #text(size: 8pt)[AC]
        #v(2pt)
        #text(size: 16pt, weight: "bold")[{}]
      ]),
      box(stroke: 1pt + colors.border, radius: 4pt, inset: 6pt, align(center)[
        #text(size: 8pt)[INIT]
        #v(2pt)
        #text(size: 16pt, weight: "bold")[{}{}]
      ]),
      box(stroke: 1pt + colors.border, radius: 4pt, inset: 6pt, align(center)[
        #text(size: 8pt)[SPEED]
        #v(2pt)
        #text(size: 16pt, weight: "bold")[{}]
      ]),
    )
    v(8pt)

    // HP Box
    box(
      width: 100%,
      stroke: 1pt + colors.border,
      radius: 4pt,
      inset: 8pt,
    )[
      #text(size: 9pt, weight: "bold")[HIT POINTS]
      #v(4pt)
      #grid(
        columns: (1fr, 1fr),
        column-gutter: 8pt,
        [Current: #text(size: 14pt, weight: "bold")[{} / {}]],
        [Temp: ______],
      )
    ]
    v(8pt)

    // Hit Dice & Death Saves
    grid(
      columns: (1fr, 1fr),
      column-gutter: 8pt,
      box(stroke: 1pt + colors.border, radius: 4pt, inset: 6pt)[
        #text(size: 8pt)[HIT DICE]
        #v(2pt)
        #text(size: 10pt)[{}]
      ],
      box(stroke: 1pt + colors.border, radius: 4pt, inset: 6pt)[
        #text(size: 8pt)[DEATH SAVES]
        #v(2pt)
        #text(size: 8pt)[S: ○○○  F: ○○○]
      ],
    )
    v(8pt)

    // Attacks
    box(
      width: 100%,
      stroke: 1pt + colors.border,
      radius: 4pt,
      inset: 8pt,
    )[
      #text(size: 9pt, weight: "bold")[ATTACKS & SPELLCASTING]
      #v(4pt)
      #table(
        columns: (1fr, auto, 1fr),
        stroke: 0.5pt + luma(200),
        inset: 4pt,
        [NAME], [ATK], [DAMAGE],
"#,
            ac,
            if dex_mod >= 0 { "+" } else { "" },
            dex_mod,
            c.speed,
            c.current_hp,
            c.max_hp,
            self.character.classes.iter().map(|cl| format!("{}d{}", cl.level, cl.hit_dice_type.trim_start_matches('d'))).collect::<Vec<_>>().join(" + ")
        ));

        // Add equipped weapon as attack
        if let Some(ref weapon) = c.equipped.main_hand {
            let atk_bonus = str_mod + prof_bonus; // simplified
            typst.push_str(&format!(
                "        [{}], [+{}], [1d8+{}],\n",
                escape_typst(weapon),
                atk_bonus,
                str_mod
            ));
        }

        typst.push_str("        [], [], [],\n        [], [], [],\n      )\n    ]\n");

        // Spell slots (if caster)
        if self.has_spellcasting() {
            let slots = self.get_spell_slots();
            typst.push_str(r#"    v(8pt)

    // Spell Slots
    box(
      width: 100%,
      stroke: 1pt + colors.border,
      radius: 4pt,
      inset: 8pt,
    )[
      #text(size: 9pt, weight: "bold")[SPELL SLOTS]
      #v(4pt)
      #grid(
        columns: (auto, 1fr, auto, 1fr),
        column-gutter: 4pt,
        row-gutter: 2pt,
"#);

            for (level, count) in slots.iter().enumerate() {
                if *count > 0 {
                    let circles = "○".repeat(*count as usize);
                    typst.push_str(&format!(
                        "        [#text(size: 8pt)[{}{}]], [#text(size: 8pt)[{}]],\n",
                        level + 1,
                        if level == 0 { "st" } else if level == 1 { "nd" } else if level == 2 { "rd" } else { "th" },
                        circles
                    ));
                }
            }

            typst.push_str("      )\n    ]\n");
        }

        // Equipment summary
        typst.push_str(r#"    v(8pt)

    // Equipment
    box(
      width: 100%,
      stroke: 1pt + colors.border,
      radius: 4pt,
      inset: 8pt,
    )[
      #text(size: 9pt, weight: "bold")[EQUIPMENT]
      #v(4pt)
      #text(size: 8pt)[
"#);

        // Currency
        let currency = &c.currency;
        let mut parts = Vec::new();
        if currency.copper > 0 { parts.push(format!("{} cp", currency.copper)); }
        if currency.silver > 0 { parts.push(format!("{} sp", currency.silver)); }
        if currency.electrum > 0 { parts.push(format!("{} ep", currency.electrum)); }
        if currency.gold > 0 { parts.push(format!("{} gp", currency.gold)); }
        if currency.platinum > 0 { parts.push(format!("{} pp", currency.platinum)); }

        if !parts.is_empty() {
            typst.push_str(&format!("        {}#linebreak()\n", parts.join(" ")));
        }

        // Equipped items
        if let Some(ref armor) = c.equipped.armor {
            typst.push_str(&format!("        {}#linebreak()\n", escape_typst(armor)));
        }
        if let Some(ref shield) = c.equipped.shield {
            typst.push_str(&format!("        {}#linebreak()\n", escape_typst(shield)));
        }
        if let Some(ref main_hand) = c.equipped.main_hand {
            typst.push_str(&format!("        {}#linebreak()\n", escape_typst(main_hand)));
        }

        typst.push_str("      ]\n    ]\n  }\n)\n\n");

        // ==== YOUR TURN FOOTER ====
        typst.push_str(r#"#v(1fr)

// Your Turn Reference
#box(
  width: 100%,
  stroke: 1pt + colors.border,
  radius: 4pt,
  inset: 8pt,
)[
  #text(size: 9pt, weight: "bold", fill: colors.accent)[YOUR TURN]
  #v(4pt)
  #grid(
    columns: (1fr, 1fr, 1fr, 1fr, 1fr),
    column-gutter: 4pt,
    box(stroke: 0.5pt + luma(200), radius: 2pt, inset: 4pt)[
      #text(size: 7pt, weight: "bold")[MOVEMENT]
      #v(2pt)
      #text(size: 6pt)[Up to speed]
      #linebreak()
      #text(size: 6pt)[Can split around actions]
    ],
    box(stroke: 0.5pt + luma(200), radius: 2pt, inset: 4pt)[
      #text(size: 7pt, weight: "bold")[ACTION]
      #v(2pt)
      #text(size: 6pt)[Attack, Cast Spell, Dash, Disengage, Dodge, Help, Hide, Ready, Search, Use]
    ],
    box(stroke: 0.5pt + luma(200), radius: 2pt, inset: 4pt)[
      #text(size: 7pt, weight: "bold")[BONUS ACTION]
      #v(2pt)
      #text(size: 6pt)[Class/feature specific (if available)]
    ],
    box(stroke: 0.5pt + luma(200), radius: 2pt, inset: 4pt)[
      #text(size: 7pt, weight: "bold")[FREE INTERACT]
      #v(2pt)
      #text(size: 6pt)[One object (draw weapon, open door)]
    ],
    box(stroke: 0.5pt + luma(200), radius: 2pt, inset: 4pt)[
      #text(size: 7pt, weight: "bold")[REACTION]
      #v(2pt)
      #text(size: 6pt)[Opportunity Attack, Ready trigger (1/round)]
    ],
  )
]
"#);

        typst
    }

    fn render_page2(&self) -> String {
        let c = &self.character;
        let mut typst = String::new();

        typst.push_str("#pagebreak()\n\n");

        // ==== HEADER ====
        typst.push_str(&format!(
            r#"#grid(
  columns: (1fr, auto),
  column-gutter: 12pt,
  [#text(size: 18pt, weight: "bold")[{}]],
  [#text(size: 9pt)[Age: \_\_\_\_\_  Height: \_\_\_\_\_  Weight: \_\_\_\_\_]],
)
#v(4pt)
#text(size: 9pt)[Eyes: \_\_\_\_\_\_\_  Skin: \_\_\_\_\_\_\_  Hair: \_\_\_\_\_\_\_]
#line(length: 100%, stroke: 1pt + colors.border)
#v(8pt)

"#,
            escape_typst(&c.character_name)
        ));

        // ==== MAIN CONTENT: 2-column layout ====
        typst.push_str(r#"#grid(
  columns: (1fr, 1fr),
  column-gutter: 16pt,

  // LEFT COLUMN - Portrait & Backstory
  {
    // Portrait placeholder
    box(
      width: 100%,
      height: 200pt,
      stroke: 1pt + colors.border,
      radius: 4pt,
      inset: 8pt,
    )[
      #align(center + horizon)[
        #text(size: 10pt, fill: luma(150))[Character Portrait]
      ]
    ]
    v(12pt)

    // Backstory
    box(
      width: 100%,
      stroke: 1pt + colors.border,
      radius: 4pt,
      inset: 8pt,
    )[
      #text(size: 9pt, weight: "bold")[CHARACTER BACKSTORY]
      #v(4pt)
"#);

        // Backstory is a placeholder - character data doesn't include backstory
        typst.push_str("      #v(80pt)\n");

        typst.push_str(r#"    ]
  },

  // RIGHT COLUMN - Personality
  {
"#);

        // Personality traits
        typst.push_str(r#"    box(
      width: 100%,
      stroke: 1pt + colors.border,
      radius: 4pt,
      inset: 8pt,
    )[
      #text(size: 9pt, weight: "bold")[PERSONALITY TRAITS]
      #v(4pt)
"#);

        if let Some(ref traits) = c.personality.traits {
            typst.push_str(&format!(
                "      #text(size: 8pt)[{}]\n",
                escape_typst(traits)
            ));
        } else {
            typst.push_str("      #v(40pt)\n");
        }

        typst.push_str(r#"    ]
    v(8pt)

    box(
      width: 100%,
      stroke: 1pt + colors.border,
      radius: 4pt,
      inset: 8pt,
    )[
      #text(size: 9pt, weight: "bold")[IDEALS]
      #v(4pt)
"#);

        if let Some(ref ideals) = c.personality.ideals {
            typst.push_str(&format!(
                "      #text(size: 8pt)[{}]\n",
                escape_typst(ideals)
            ));
        } else {
            typst.push_str("      #v(40pt)\n");
        }

        typst.push_str(r#"    ]
    v(8pt)

    box(
      width: 100%,
      stroke: 1pt + colors.border,
      radius: 4pt,
      inset: 8pt,
    )[
      #text(size: 9pt, weight: "bold")[BONDS]
      #v(4pt)
"#);

        if let Some(ref bonds) = c.personality.bonds {
            typst.push_str(&format!(
                "      #text(size: 8pt)[{}]\n",
                escape_typst(bonds)
            ));
        } else {
            typst.push_str("      #v(40pt)\n");
        }

        typst.push_str(r#"    ]
    v(8pt)

    box(
      width: 100%,
      stroke: 1pt + colors.border,
      radius: 4pt,
      inset: 8pt,
    )[
      #text(size: 9pt, weight: "bold")[FLAWS]
      #v(4pt)
"#);

        if let Some(ref flaws) = c.personality.flaws {
            typst.push_str(&format!(
                "      #text(size: 8pt)[{}]\n",
                escape_typst(flaws)
            ));
        } else {
            typst.push_str("      #v(40pt)\n");
        }

        typst.push_str("    ]\n  }\n)\n\n#v(12pt)\n\n");

        // ==== BOTTOM ROW: Features & Allies ====
        typst.push_str(r#"#grid(
  columns: (1fr, 1fr),
  column-gutter: 16pt,

  // Allies & Organizations
  box(
    width: 100%,
    stroke: 1pt + colors.border,
    radius: 4pt,
    inset: 8pt,
  )[
    #text(size: 9pt, weight: "bold")[ALLIES & ORGANIZATIONS]
    #v(4pt)
    #v(60pt)
  ],

  // Features & Traits
  box(
    width: 100%,
    stroke: 1pt + colors.border,
    radius: 4pt,
    inset: 8pt,
  )[
    #text(size: 9pt, weight: "bold")[FEATURES & TRAITS]
    #v(4pt)
"#);

        for feature in &c.class_features {
            typst.push_str(&format!(
                "    #text(size: 8pt)[- {}]#linebreak()\n",
                escape_typst(&feature.name)
            ));
        }

        if c.class_features.is_empty() {
            typst.push_str("    #v(60pt)\n");
        }

        typst.push_str(r#"  ],
)

#v(12pt)

// Additional Treasure
#box(
  width: 100%,
  stroke: 1pt + colors.border,
  radius: 4pt,
  inset: 8pt,
)[
  #text(size: 9pt, weight: "bold")[ADDITIONAL TREASURE]
  #v(4pt)
  #v(40pt)
]

#v(1fr)
#align(center)[
  #text(size: 8pt, fill: luma(150))[Generated by Mimir]
]
"#);

        typst
    }
}

impl Renderable for CompactSheetSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        let mut typst = String::new();

        // Page 1: Combat & Stats
        typst.push_str(&self.render_page1());

        // Page 2: Character Details
        typst.push_str(&self.render_page2());

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        Some(format!("{} - Compact Sheet", self.character.character_name))
    }

    fn page_break_before(&self) -> bool {
        true
    }
}

/// Escape special Typst characters
fn escape_typst(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use mimir_dm_core::models::character::data::{
        AbilityScores, ClassLevel, Currency, EquippedItems, Personality, Proficiencies,
        SpellData as CharacterSpellData,
    };

    fn sample_character() -> CharacterData {
        CharacterData {
            character_name: "Aria Silverleaf".to_string(),
            player_id: Some(1),
            level: 5,
            experience_points: 6500,
            version: 1,
            snapshot_reason: None,
            created_at: "2025-01-01".to_string(),
            race: "Elf".to_string(),
            subrace: Some("High".to_string()),
            classes: vec![ClassLevel {
                class_name: "Wizard".to_string(),
                level: 5,
                subclass: Some("Evocation".to_string()),
                hit_dice_type: "d6".to_string(),
                hit_dice_remaining: 5,
            }],
            background: "Sage".to_string(),
            alignment: Some("Neutral Good".to_string()),
            abilities: AbilityScores {
                strength: 8,
                dexterity: 14,
                constitution: 12,
                intelligence: 18,
                wisdom: 13,
                charisma: 10,
            },
            max_hp: 27,
            current_hp: 27,
            speed: 30,
            proficiencies: Proficiencies {
                skills: vec!["Arcana".to_string(), "History".to_string(), "Investigation".to_string()],
                saves: vec!["Intelligence".to_string(), "Wisdom".to_string()],
                armor: vec![],
                weapons: vec!["Daggers".to_string(), "Darts".to_string(), "Quarterstaffs".to_string()],
                tools: vec![],
                languages: vec!["Common".to_string(), "Elvish".to_string(), "Draconic".to_string()],
            },
            class_features: vec![],
            feats: vec![],
            spells: CharacterSpellData::default(),
            inventory: vec![],
            currency: Currency { copper: 0, silver: 50, electrum: 0, gold: 125, platinum: 0 },
            equipped: EquippedItems {
                armor: None,
                shield: None,
                main_hand: Some("Quarterstaff".to_string()),
                off_hand: None,
            },
            personality: Personality {
                traits: Some("I use polysyllabic words that convey the impression of great erudition.".to_string()),
                ideals: Some("Knowledge. The path to power is through knowledge.".to_string()),
                bonds: Some("I have an ancient text that holds terrible secrets.".to_string()),
                flaws: Some("I overlook obvious solutions in favor of complicated ones.".to_string()),
            },
            npc_role: None,
            npc_location: None,
            npc_faction: None,
            npc_notes: None,
            legendary_actions: vec![],
            legendary_action_count: None,
        }
    }

    #[test]
    fn test_compact_sheet_basic() {
        let character = sample_character();
        let section = CompactSheetSection::new(character);
        assert_eq!(
            section.toc_title(),
            Some("Aria Silverleaf - Compact Sheet".to_string())
        );
    }

    #[test]
    fn test_compact_sheet_generates_typst() {
        let character = sample_character();
        let section = CompactSheetSection::new(character);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        // Check for character name
        assert!(typst.contains("Aria Silverleaf"));
        // Check for class
        assert!(typst.contains("Wizard"));
        // Check for page setup
        assert!(typst.contains("us-letter"));
        // Check for Your Turn section
        assert!(typst.contains("YOUR TURN"));
        // Check for both pages
        assert!(typst.contains("pagebreak"));
    }
}
