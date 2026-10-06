//! Table-driven tests for `CharacterService::level_up` (MIMIR-T-0666).
//!
//! Each case: a starting character (ability scores + existing classes), a
//! request, and the expected outcome. Cases with side effects carry a
//! `check` that inspects the database afterwards. The DB has the SRD
//! catalog loaded, so hit dice and multiclass proficiencies come from it.
//! These tests pin today's behaviour; they do not change it.

use super::super::test_support::create_test_campaign;
use super::*;
use crate::models::campaign::NewCharacter;
use crate::seed::srd::seed_srd_catalog;
use crate::test_utils::setup_test_db;

/// Ability scores in STR, DEX, CON, INT, WIS, CHA order.
type Scores = [i32; 6];
/// Existing classes: (class, level, subclass). The first is the starting class.
type Classes = &'static [(&'static str, i32, Option<&'static str>)];
type Check = fn(&mut SqliteConnection, &str, &LevelUpResult);

enum Expect {
    Ok {
        hp_gained: i32,
        new_total_level: i32,
        is_multiclass: bool,
        class_level: i32,
        check: Option<Check>,
    },
    Err(&'static str),
}

struct Case {
    name: &'static str,
    scores: Scores,
    classes: Classes,
    request: LevelUpRequest,
    expect: Expect,
}

// Starting characters.
const FIGHTER: Scores = [16, 12, 14, 10, 12, 8]; // CON +2
const WIZARD: Scores = [8, 14, 12, 16, 12, 10]; // CON +1
const FRAIL_WIZARD: Scores = [8, 14, 6, 16, 12, 10]; // CON -2
const STRONG_SMART: Scores = [16, 12, 14, 14, 10, 8]; // STR and INT 13+
const DEX_ROGUE: Scores = [8, 16, 12, 10, 12, 10]; // DEX 13+, STR below

fn req(class: &str, hp: HpGainMethod) -> LevelUpRequest {
    LevelUpRequest {
        class_name: class.to_string(),
        class_source: "PHB".to_string(),
        hit_points_method: hp,
        subclass: None,
        asi_or_feat: None,
        spell_changes: None,
        feature_choices: None,
    }
}

fn asi(a1: &str, i1: i32, a2: Option<&str>, i2: Option<i32>) -> Option<AsiOrFeat> {
    Some(AsiOrFeat::AbilityScoreImprovement {
        ability1: a1.to_string(),
        increase1: i1,
        ability2: a2.map(str::to_string),
        increase2: i2,
    })
}

fn spell(name: &str) -> SpellReference {
    SpellReference {
        name: name.to_string(),
        source: "PHB".to_string(),
    }
}

fn feature(name: &str) -> FeatureReference {
    FeatureReference {
        name: name.to_string(),
        source: "PHB".to_string(),
    }
}

fn no_features() -> FeatureChoices {
    FeatureChoices {
        fighting_style: None,
        metamagic: None,
        maneuvers: None,
        invocations: None,
        pact_boon: None,
        expertise_skills: None,
    }
}

fn setup(scores: Scores, classes: Classes) -> (SqliteConnection, String) {
    let mut conn = setup_test_db();
    seed_srd_catalog(&mut conn).expect("SRD catalog");
    let campaign_id = create_test_campaign(&mut conn);
    let id = Uuid::new_v4().to_string();
    let [s, d, c, i, w, ch] = scores;
    dal::insert_character(
        &mut conn,
        &NewCharacter::new_pc(&id, Some(&campaign_id), "Tester", "Player")
            .with_ability_scores(s, d, c, i, w, ch),
    )
    .expect("character");
    for (n, (class, level, subclass)) in classes.iter().enumerate() {
        let class_id = Uuid::new_v4().to_string();
        let base = if n == 0 {
            NewCharacterClass::starting(&class_id, &id, class, "PHB")
        } else {
            NewCharacterClass::multiclass(&class_id, &id, class, "PHB")
        };
        let mut row = base.with_level(*level);
        if let Some(sub) = subclass {
            row = row.with_subclass(sub, "PHB");
        }
        dal::insert_character_class(&mut conn, &row).expect("class");
    }
    (conn, id)
}

fn cases() -> Vec<Case> {
    use HpGainMethod::{Average, Manual, Roll};
    vec![
        // ── HP gain ─────────────────────────────────────────────────────
        Case {
            name: "average HP: d10 average 6 + CON 2",
            scores: FIGHTER,
            classes: &[("Fighter", 3, None)],
            request: req("Fighter", Average),
            expect: Expect::Ok {
                hp_gained: 8,
                new_total_level: 4,
                is_multiclass: false,
                class_level: 4,
                check: None,
            },
        },
        Case {
            name: "rolled HP: 7 + CON 2",
            scores: FIGHTER,
            classes: &[("Fighter", 3, None)],
            request: req("Fighter", Roll(7)),
            expect: Expect::Ok {
                hp_gained: 9,
                new_total_level: 4,
                is_multiclass: false,
                class_level: 4,
                check: None,
            },
        },
        Case {
            name: "rolled HP at the die's maximum is allowed",
            scores: FIGHTER,
            classes: &[("Fighter", 3, None)],
            request: req("Fighter", Roll(10)),
            expect: Expect::Ok {
                hp_gained: 12,
                new_total_level: 4,
                is_multiclass: false,
                class_level: 4,
                check: None,
            },
        },
        Case {
            name: "manual HP: 3 + CON 2",
            scores: FIGHTER,
            classes: &[("Fighter", 3, None)],
            request: req("Fighter", Manual(3)),
            expect: Expect::Ok {
                hp_gained: 5,
                new_total_level: 4,
                is_multiclass: false,
                class_level: 4,
                check: None,
            },
        },
        Case {
            name: "roll above the hit die is rejected",
            scores: FIGHTER,
            classes: &[("Fighter", 3, None)],
            request: req("Fighter", Roll(11)),
            expect: Expect::Err("HP roll 11 is invalid for hit die d10"),
        },
        Case {
            name: "roll of 0 is rejected",
            scores: FIGHTER,
            classes: &[("Fighter", 3, None)],
            request: req("Fighter", Roll(0)),
            expect: Expect::Err("HP roll 0 is invalid"),
        },
        Case {
            name: "negative CON never drops HP gain below 1",
            scores: FRAIL_WIZARD,
            classes: &[("Wizard", 2, None)],
            request: req("Wizard", Roll(1)),
            expect: Expect::Ok {
                hp_gained: 1,
                new_total_level: 3,
                is_multiclass: false,
                class_level: 3,
                check: None,
            },
        },
        Case {
            name: "wizard uses the catalog d6: average 4 + CON 1",
            scores: WIZARD,
            classes: &[("Wizard", 2, None)],
            request: req("Wizard", Average),
            expect: Expect::Ok {
                hp_gained: 5,
                new_total_level: 3,
                is_multiclass: false,
                class_level: 3,
                check: None,
            },
        },
        // ── First class and multiclass ─────────────────────────────────
        Case {
            name: "first class on a classless character is not a multiclass",
            scores: FIGHTER,
            classes: &[],
            request: req("Fighter", Average),
            expect: Expect::Ok {
                hp_gained: 8,
                new_total_level: 1,
                is_multiclass: false,
                class_level: 1,
                check: None,
            },
        },
        Case {
            name: "multiclass into Wizard with STR and INT 13+",
            scores: STRONG_SMART,
            classes: &[("Fighter", 3, None)],
            request: req("Wizard", Average),
            expect: Expect::Ok {
                hp_gained: 6,
                new_total_level: 4,
                is_multiclass: true,
                class_level: 1,
                check: None,
            },
        },
        Case {
            name: "leveling the second class keeps both and sums levels",
            scores: STRONG_SMART,
            classes: &[("Fighter", 3, None), ("Wizard", 1, None)],
            request: req("Wizard", Average),
            expect: Expect::Ok {
                hp_gained: 6,
                new_total_level: 5,
                is_multiclass: false,
                class_level: 2,
                check: None,
            },
        },
        Case {
            name: "multiclass refused when the new class's prerequisite fails",
            scores: FIGHTER,
            classes: &[("Fighter", 3, None)],
            request: req("Wizard", Average),
            expect: Expect::Err("Wizard requires intelligence 13 (character has 10)"),
        },
        Case {
            name: "multiclass refused when a current class's prerequisite fails",
            // INT 12 is below the Wizard requirement the character must still meet.
            scores: [8, 14, 12, 12, 10, 16],
            classes: &[("Wizard", 2, None)],
            request: req("Sorcerer", Average),
            expect: Expect::Err("Wizard requires intelligence 13"),
        },
        Case {
            name: "Fighter's STR-or-DEX prerequisite passes on DEX alone",
            scores: DEX_ROGUE,
            classes: &[("Rogue", 2, None)],
            request: req("Fighter", Average),
            expect: Expect::Ok {
                hp_gained: 7,
                new_total_level: 3,
                is_multiclass: true,
                class_level: 1,
                check: Some(|conn, id, _| {
                    let profs = dal::list_character_proficiencies(conn, id).unwrap();
                    let has = |t: &str, n: &str| {
                        profs
                            .iter()
                            .any(|p| p.proficiency_type == t && p.name.eq_ignore_ascii_case(n))
                    };
                    assert!(
                        has("armor", "medium"),
                        "Fighter multiclass grants medium armor: {profs:?}"
                    );
                    assert!(
                        has("weapon", "martial"),
                        "Fighter multiclass grants martial weapons: {profs:?}"
                    );
                }),
            },
        },
        Case {
            name: "Fighter's STR-or-DEX prerequisite fails when both are low",
            scores: [8, 10, 12, 16, 12, 10],
            classes: &[("Wizard", 2, None)],
            request: req("Fighter", Average),
            expect: Expect::Err("Fighter requires strength 13 or dexterity 13"),
        },
        Case {
            name: "Monk needs DEX and WIS: WIS 12 fails",
            scores: [8, 16, 12, 10, 12, 10],
            classes: &[("Rogue", 2, None)],
            request: req("Monk", Average),
            expect: Expect::Err("Monk requires wisdom 13 (character has 12)"),
        },
        // ── Subclass ───────────────────────────────────────────────────
        Case {
            name: "subclass chosen at Fighter 3 is recorded on the class",
            scores: FIGHTER,
            classes: &[("Fighter", 2, None)],
            request: LevelUpRequest {
                subclass: Some(SubclassChoice {
                    name: "Champion".into(),
                    source: "PHB".into(),
                }),
                ..req("Fighter", Average)
            },
            expect: Expect::Ok {
                hp_gained: 8,
                new_total_level: 3,
                is_multiclass: false,
                class_level: 3,
                check: Some(|_, _, r| {
                    assert_eq!(r.class.subclass_name.as_deref(), Some("Champion"));
                    assert_eq!(r.class.subclass_source.as_deref(), Some("PHB"));
                }),
            },
        },
        // ── ASI and feats ──────────────────────────────────────────────
        Case {
            name: "ASI +2 to one ability",
            scores: FIGHTER,
            classes: &[("Fighter", 3, None)],
            request: LevelUpRequest {
                asi_or_feat: asi("strength", 2, None, None),
                ..req("Fighter", Average)
            },
            expect: Expect::Ok {
                hp_gained: 8,
                new_total_level: 4,
                is_multiclass: false,
                class_level: 4,
                check: Some(|_, _, r| assert_eq!(r.character.strength, 18)),
            },
        },
        Case {
            name: "ASI +1/+1 accepts short ability names",
            scores: FIGHTER,
            classes: &[("Fighter", 3, None)],
            request: LevelUpRequest {
                asi_or_feat: asi("str", 1, Some("con"), Some(1)),
                ..req("Fighter", Average)
            },
            expect: Expect::Ok {
                hp_gained: 8,
                new_total_level: 4,
                is_multiclass: false,
                class_level: 4,
                check: Some(|_, _, r| {
                    assert_eq!(r.character.strength, 17);
                    assert_eq!(r.character.constitution, 15);
                }),
            },
        },
        Case {
            name: "ASI caps an ability at 20",
            scores: [19, 12, 14, 10, 12, 8],
            classes: &[("Fighter", 3, None)],
            request: LevelUpRequest {
                asi_or_feat: asi("strength", 2, None, None),
                ..req("Fighter", Average)
            },
            expect: Expect::Ok {
                hp_gained: 8,
                new_total_level: 4,
                is_multiclass: false,
                class_level: 4,
                check: Some(|_, _, r| assert_eq!(r.character.strength, 20)),
            },
        },
        Case {
            name: "ASI totalling other than 2 is rejected",
            scores: FIGHTER,
            classes: &[("Fighter", 3, None)],
            request: LevelUpRequest {
                asi_or_feat: asi("strength", 1, None, None),
                ..req("Fighter", Average)
            },
            expect: Expect::Err("ASI total increase must be exactly 2, got 1"),
        },
        Case {
            name: "feat instead of ASI is recorded with source type asi",
            scores: FIGHTER,
            classes: &[("Fighter", 3, None)],
            request: LevelUpRequest {
                asi_or_feat: Some(AsiOrFeat::Feat {
                    name: "Alert".into(),
                    source: "PHB".into(),
                }),
                ..req("Fighter", Average)
            },
            expect: Expect::Ok {
                hp_gained: 8,
                new_total_level: 4,
                is_multiclass: false,
                class_level: 4,
                check: Some(|conn, id, r| {
                    let feats = dal::list_character_feats(conn, id).unwrap();
                    assert_eq!(feats.len(), 1);
                    assert_eq!(feats[0].feat_name, "Alert");
                    assert_eq!(feats[0].source_type, "asi");
                    assert_eq!(
                        r.character.strength, 16,
                        "a feat leaves ability scores alone"
                    );
                }),
            },
        },
        // ── Spells ─────────────────────────────────────────────────────
        Case {
            name: "new spells and cantrips are recorded for the leveled class",
            scores: WIZARD,
            classes: &[("Wizard", 2, None)],
            request: LevelUpRequest {
                spell_changes: Some(SpellChanges {
                    new_spells: vec![spell("Scorching Ray"), spell("Misty Step")],
                    new_cantrips: vec![spell("Light")],
                    swap_out: None,
                    swap_in: None,
                }),
                ..req("Wizard", Average)
            },
            expect: Expect::Ok {
                hp_gained: 5,
                new_total_level: 3,
                is_multiclass: false,
                class_level: 3,
                check: Some(|conn, id, _| {
                    let spells = dal::list_character_spells(conn, id).unwrap();
                    let mut names: Vec<_> = spells.iter().map(|s| s.spell_name.as_str()).collect();
                    names.sort();
                    assert_eq!(names, ["Light", "Misty Step", "Scorching Ray"]);
                    assert!(spells.iter().all(|s| s.source_class == "Wizard"));
                }),
            },
        },
        Case {
            name: "swapping out a spell the class does not know is rejected",
            scores: WIZARD,
            classes: &[("Wizard", 2, None)],
            request: LevelUpRequest {
                spell_changes: Some(SpellChanges {
                    new_spells: vec![],
                    new_cantrips: vec![],
                    swap_out: Some(spell("Fireball")),
                    swap_in: Some(spell("Shield")),
                }),
                ..req("Wizard", Average)
            },
            expect: Expect::Err("Cannot swap out spell 'Fireball'"),
        },
        // ── Feature choices ────────────────────────────────────────────
        Case {
            name: "fighting style and metamagic are recorded as features",
            scores: [10, 14, 14, 10, 10, 16],
            classes: &[("Sorcerer", 2, None)],
            request: LevelUpRequest {
                feature_choices: Some(FeatureChoices {
                    fighting_style: Some(feature("Defense")),
                    metamagic: Some(vec![feature("Quickened Spell"), feature("Twinned Spell")]),
                    ..no_features()
                }),
                ..req("Sorcerer", Average)
            },
            expect: Expect::Ok {
                hp_gained: 6,
                new_total_level: 3,
                is_multiclass: false,
                class_level: 3,
                check: Some(|conn, id, _| {
                    assert_eq!(
                        dal::list_features_by_type(conn, id, "fighting_style")
                            .unwrap()
                            .len(),
                        1
                    );
                    let mm = dal::list_features_by_type(conn, id, "metamagic").unwrap();
                    let mut names: Vec<_> = mm.iter().map(|f| f.feature_name.as_str()).collect();
                    names.sort();
                    assert_eq!(names, ["Quickened Spell", "Twinned Spell"]);
                }),
            },
        },
        Case {
            name: "expertise adds a skill proficiency with expertise when missing",
            scores: DEX_ROGUE,
            classes: &[("Rogue", 5, None)],
            request: LevelUpRequest {
                feature_choices: Some(FeatureChoices {
                    expertise_skills: Some(vec!["Stealth".into()]),
                    ..no_features()
                }),
                ..req("Rogue", Average)
            },
            expect: Expect::Ok {
                hp_gained: 6,
                new_total_level: 6,
                is_multiclass: false,
                class_level: 6,
                check: Some(|conn, id, _| {
                    let profs = dal::list_character_proficiencies(conn, id).unwrap();
                    let stealth = profs
                        .iter()
                        .find(|p| p.proficiency_type == "skill" && p.name == "Stealth")
                        .expect("Stealth proficiency");
                    assert_eq!(stealth.expertise, 1);
                }),
            },
        },
    ]
}

#[test]
fn level_up_table() {
    let mut failures = Vec::new();
    for case in cases() {
        let (mut conn, id) = setup(case.scores, case.classes);
        let result = CharacterService::new(&mut conn).level_up(&id, case.request);
        match (&case.expect, &result) {
            (
                Expect::Ok {
                    hp_gained,
                    new_total_level,
                    is_multiclass,
                    class_level,
                    check,
                },
                Ok(r),
            ) => {
                let got = (
                    r.hp_gained,
                    r.new_total_level,
                    r.is_multiclass,
                    r.class.level,
                );
                let want = (*hp_gained, *new_total_level, *is_multiclass, *class_level);
                if got != want {
                    failures.push(format!(
                        "{}: (hp, total, multiclass, class level) = {got:?}, want {want:?}",
                        case.name
                    ));
                    continue;
                }
                if let Some(check) = check {
                    check(&mut conn, &id, r);
                }
            }
            (Expect::Err(fragment), Err(e)) => {
                if !e.to_string().contains(fragment) {
                    failures.push(format!("{}: error {e:?} lacks {fragment:?}", case.name));
                }
            }
            (Expect::Ok { .. }, Err(e)) => {
                failures.push(format!("{}: unexpected error {e}", case.name))
            }
            (Expect::Err(fragment), Ok(r)) => failures.push(format!(
                "{}: expected an error containing {fragment:?}, got level {}",
                case.name, r.class.level
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "level-up cases failed:\n{}",
        failures.join("\n")
    );
}

#[test]
fn hit_die_comes_from_the_catalog_entry() {
    // A class whose catalog hit die (d12) differs from the d8 name fallback.
    let (mut conn, id) = setup(FIGHTER, &[]);
    catalog_dal::insert_source(
        &mut conn,
        &crate::models::catalog::NewCatalogSource::new(
            "UA",
            "Unearthed Arcana",
            true,
            "2024-01-20T12:00:00Z",
        ),
    )
    .unwrap();
    catalog_dal::insert_class(
        &mut conn,
        &crate::models::catalog::NewClass {
            name: "Mystic",
            source: "UA",
            data: r#"{"name":"Mystic","source":"UA","hd":{"number":1,"faces":12}}"#,
            fluff: None,
        },
    )
    .unwrap();
    let request = LevelUpRequest {
        class_source: "UA".into(),
        ..req("Mystic", HpGainMethod::Roll(12))
    };
    let r = CharacterService::new(&mut conn)
        .level_up(&id, request)
        .expect("d12 roll of 12 is valid");
    assert_eq!(r.hp_gained, 14, "12 + CON 2");

    // Without a catalog entry the name fallback (d8) applies.
    let (mut conn, id) = setup(FIGHTER, &[]);
    let request = LevelUpRequest {
        class_source: "UA".into(),
        ..req("Mystic", HpGainMethod::Roll(9))
    };
    let err = CharacterService::new(&mut conn)
        .level_up(&id, request)
        .unwrap_err();
    assert!(err.to_string().contains("hit die d8"), "{err}");
}

#[test]
fn missing_character_is_not_found() {
    let (mut conn, _) = setup(FIGHTER, &[]);
    let err = CharacterService::new(&mut conn)
        .level_up("no-such-character", req("Fighter", HpGainMethod::Average))
        .unwrap_err();
    assert!(matches!(err, ServiceError::NotFound { .. }), "{err:?}");
}
