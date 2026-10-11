//! The skill choices of a new character (MIMIR-T-0721): the background's
//! skills (granted) and the class's pick (how many, from which).

use diesel::SqliteConnection;

use crate::dal::catalog as catalog_dal;
use crate::services::ServiceResult;

/// The eighteen skills, for a class that may pick any.
pub const ALL_SKILLS: [&str; 18] = [
    "Acrobatics",
    "Animal Handling",
    "Arcana",
    "Athletics",
    "Deception",
    "History",
    "Insight",
    "Intimidation",
    "Investigation",
    "Medicine",
    "Nature",
    "Perception",
    "Performance",
    "Persuasion",
    "Religion",
    "Sleight of Hand",
    "Stealth",
    "Survival",
];

/// The skills a new character gets and picks.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CreationSkills {
    /// Granted by the background.
    pub background: Vec<String>,
    /// How many the class picks.
    pub choose: usize,
    /// What the class picks from (without the background's).
    pub from: Vec<String>,
}

/// A skill name as on a sheet: "sleight of hand" → "Sleight of Hand".
pub fn skill_title(name: &str) -> String {
    let name = name.split('|').next().unwrap_or(name);
    name.split_whitespace()
        .enumerate()
        .map(|(i, w)| {
            if i > 0 && w.eq_ignore_ascii_case("of") {
                "of".to_string()
            } else {
                let mut c = w.chars();
                c.next()
                    .map(|f| {
                        f.to_uppercase()
                            .chain(c.flat_map(|x| x.to_lowercase()))
                            .collect()
                    })
                    .unwrap_or_default()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The skill choices from the class and background catalog JSON.
pub fn creation_skills_from(
    class_data: Option<&serde_json::Value>,
    bg_data: Option<&serde_json::Value>,
) -> CreationSkills {
    let mut out = CreationSkills::default();
    if let Some(first) = bg_data
        .and_then(|b| b.get("skillProficiencies"))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_object())
    {
        for (k, v) in first {
            if v.as_bool() == Some(true) {
                out.background.push(skill_title(k));
            }
        }
    }
    if let Some(first) = class_data
        .and_then(|c| c.get("startingProficiencies"))
        .and_then(|s| s.get("skills"))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
    {
        if let Some(choose) = first.get("choose") {
            out.choose = choose.get("count").and_then(|c| c.as_u64()).unwrap_or(1) as usize;
            out.from = choose
                .get("from")
                .and_then(|f| f.as_array())
                .map(|f| {
                    f.iter()
                        .filter_map(|s| s.as_str())
                        .map(skill_title)
                        .collect()
                })
                .unwrap_or_default();
        } else if let Some(n) = first.get("any").and_then(|n| n.as_u64()) {
            out.choose = n as usize;
            out.from = ALL_SKILLS.iter().map(|s| s.to_string()).collect();
        }
    }
    let bg: Vec<String> = out.background.iter().map(|s| s.to_lowercase()).collect();
    out.from.retain(|s| !bg.contains(&s.to_lowercase()));
    out.choose = out.choose.min(out.from.len());
    out
}

/// The skill choices of a class and a background in the catalog (either
/// may be absent or unknown: nothing from it).
pub fn creation_skills(
    conn: &mut SqliteConnection,
    class: Option<(&str, &str)>,
    background: Option<(&str, &str)>,
) -> ServiceResult<CreationSkills> {
    let class_data = match class {
        Some((n, s)) => catalog_dal::get_class_by_name(conn, n, s)?
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c.data).ok()),
        None => None,
    };
    let bg_data = match background {
        Some((n, s)) => catalog_dal::get_background_by_name(conn, n, s)?
            .and_then(|b| serde_json::from_str::<serde_json::Value>(&b.data).ok()),
        None => None,
    };
    Ok(creation_skills_from(class_data.as_ref(), bg_data.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn titles() {
        assert_eq!(skill_title("sleight of hand"), "Sleight of Hand");
        assert_eq!(skill_title("animal handling|phb"), "Animal Handling");
    }

    #[test]
    fn a_class_picks_from_its_list_less_the_background() {
        let class = json!({"startingProficiencies": {"skills": [{"choose": {"count": 2, "from": ["athletics", "insight", "perception"]}}]}});
        let bg = json!({"skillProficiencies": [{"insight": true, "religion": true}]});
        let s = creation_skills_from(Some(&class), Some(&bg));
        assert_eq!(s.background.len(), 2);
        assert!(s.background.contains(&"Insight".to_string()));
        assert_eq!(
            s.from,
            vec!["Athletics".to_string(), "Perception".to_string()]
        );
        assert_eq!(s.choose, 2);
    }

    #[test]
    fn any_skills_and_nothing() {
        let class = json!({"startingProficiencies": {"skills": [{"any": 3}]}});
        let s = creation_skills_from(Some(&class), None);
        assert_eq!((s.choose, s.from.len()), (3, 18));
        assert_eq!(creation_skills_from(None, None), CreationSkills::default());
    }
}
