//! Psionic Model
//!
//! Represents psionic disciplines and talents from UA/homebrew content.

use crate::schema::psionics;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A psionic discipline or talent from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = psionics)]
#[diesel(primary_key(id))]
pub struct Psionic {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    /// Type code (D = Discipline, T = Talent)
    pub psionic_type: Option<String>,
    /// Psionic order (Avatar, Awakened, Immortal, Nomad, Wu Jen)
    pub psionic_order: Option<String>,
    pub data: String,
}

impl Psionic {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }

    /// Check if this is a discipline.
    pub fn is_discipline(&self) -> bool {
        self.psionic_type.as_ref().map_or(false, |t| t == "D")
    }

    /// Check if this is a talent.
    pub fn is_talent(&self) -> bool {
        self.psionic_type.as_ref().map_or(false, |t| t == "T")
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = psionics)]
pub struct NewPsionic<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub psionic_type: Option<&'a str>,
    pub psionic_order: Option<&'a str>,
    pub data: &'a str,
}

impl<'a> NewPsionic<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self {
            name,
            source,
            psionic_type: None,
            psionic_order: None,
            data,
        }
    }

    pub fn with_type(mut self, psionic_type: &'a str) -> Self {
        self.psionic_type = Some(psionic_type);
        self
    }

    pub fn with_order(mut self, order: &'a str) -> Self {
        self.psionic_order = Some(order);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_psionic() {
        let psionic = NewPsionic::new("Mastery of Force", "UAMystic", r#"{"name":"Mastery of Force"}"#)
            .with_type("D")
            .with_order("Wu Jen");
        assert_eq!(psionic.name, "Mastery of Force");
        assert_eq!(psionic.psionic_type, Some("D"));
        assert_eq!(psionic.psionic_order, Some("Wu Jen"));
    }
}
