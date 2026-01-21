//! Skill Model
//!
//! Represents a skill (Athletics, Perception, Stealth, etc.).

use crate::schema::skills;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A skill from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = skills)]
#[diesel(primary_key(id))]
pub struct Skill {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    pub ability: Option<String>,
    pub data: String,
}

impl Skill {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = skills)]
pub struct NewSkill<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub ability: Option<&'a str>,
    pub data: &'a str,
}

impl<'a> NewSkill<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, ability: None, data }
    }

    pub fn with_ability(mut self, ability: &'a str) -> Self {
        self.ability = Some(ability);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_skill() {
        let skill = NewSkill::new("Athletics", "PHB", r#"{"name":"Athletics"}"#)
            .with_ability("str");
        assert_eq!(skill.name, "Athletics");
        assert_eq!(skill.ability, Some("str"));
    }
}
