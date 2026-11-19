//! The rules of the character wizard (MIMIR-T-0721), as the desktop app:
//! ability scores by the standard array, point buy or manual entry; the
//! class's skill pick; when a step may go on. Pure, host-tested.

/// The ability names, in score order.
pub const ABILITIES: [&str; 6] = [
    "Strength",
    "Dexterity",
    "Constitution",
    "Intelligence",
    "Wisdom",
    "Charisma",
];

pub const STANDARD_ARRAY: [i32; 6] = [15, 14, 13, 12, 10, 8];
pub const POINT_BUY_POINTS: i32 = 27;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Standard,
    PointBuy,
    Manual,
}

impl Method {
    pub const ALL: [(Method, &'static str, &'static str); 3] = [
        (
            Method::Standard,
            "standard",
            "Standard array (15, 14, 13, 12, 10, 8)",
        ),
        (Method::PointBuy, "point-buy", "Point buy (27 points)"),
        (Method::Manual, "manual", "Manual entry (3 to 18)"),
    ];

    pub fn key(self) -> &'static str {
        Method::ALL
            .iter()
            .find(|m| m.0 == self)
            .map(|m| m.1)
            .unwrap_or("standard")
    }

    pub fn from_key(key: &str) -> Method {
        Method::ALL
            .iter()
            .find(|m| m.1 == key)
            .map(|m| m.0)
            .unwrap_or(Method::Standard)
    }

    /// The scores a method starts from (0: not yet picked).
    pub fn start(self) -> [i32; 6] {
        match self {
            Method::Standard => [0; 6],
            Method::PointBuy => [8; 6],
            Method::Manual => [10; 6],
        }
    }

    /// The lowest and highest score the +/- buttons reach.
    pub fn range(self) -> (i32, i32) {
        match self {
            Method::Standard => (8, 15),
            Method::PointBuy => (8, 15),
            Method::Manual => (3, 18),
        }
    }
}

/// The point-buy cost of a score (8 to 15).
pub fn point_cost(score: i32) -> Option<i32> {
    Some(match score {
        8 => 0,
        9 => 1,
        10 => 2,
        11 => 3,
        12 => 4,
        13 => 5,
        14 => 7,
        15 => 9,
        _ => return None,
    })
}

/// The points left of 27 (negative: too many spent; a score out of 8–15
/// counts as all).
pub fn points_left(scores: &[i32; 6]) -> i32 {
    let spent: Option<i32> = scores.iter().map(|s| point_cost(*s)).sum();
    POINT_BUY_POINTS - spent.unwrap_or(i32::MAX / 2)
}

/// Whether the scores are complete and legal for the method.
pub fn scores_valid(method: Method, scores: &[i32; 6]) -> bool {
    match method {
        Method::Standard => {
            let mut sorted = *scores;
            sorted.sort_unstable_by(|a, b| b.cmp(a));
            sorted == STANDARD_ARRAY
        }
        Method::PointBuy => scores.iter().all(|s| (8..=15).contains(s)) && points_left(scores) == 0,
        Method::Manual => scores.iter().all(|s| (3..=18).contains(s)),
    }
}

/// The standard-array values the ability at `index` may take (its own and
/// those no other ability holds).
pub fn standard_options(scores: &[i32; 6], index: usize) -> Vec<i32> {
    STANDARD_ARRAY
        .iter()
        .copied()
        .filter(|v| scores.iter().enumerate().all(|(i, s)| i == index || s != v))
        .collect()
}

/// Whether +1 is allowed on the ability at `index`.
pub fn can_raise(method: Method, scores: &[i32; 6], index: usize) -> bool {
    let s = scores[index];
    match method {
        Method::Standard => false,
        Method::PointBuy => {
            let (Some(now), Some(next)) = (point_cost(s), point_cost(s + 1)) else {
                return false;
            };
            next - now <= points_left(scores)
        }
        Method::Manual => s < 18,
    }
}

/// Whether -1 is allowed on the ability at `index`.
pub fn can_lower(method: Method, scores: &[i32; 6], index: usize) -> bool {
    let (low, _) = method.range();
    method != Method::Standard && scores[index] > low
}

/// The modifier of a score.
pub fn modifier(score: i32) -> i32 {
    (score - 10).div_euclid(2)
}

/// A modifier as shown: "+2", "-1", "+0".
pub fn signed(n: i32) -> String {
    if n >= 0 {
        format!("+{n}")
    } else {
        n.to_string()
    }
}

/// The class's skill pick is complete.
pub fn skills_valid(choose: usize, picked: usize) -> bool {
    picked == choose
}

/// The steps of a PC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Basics,
    Race,
    Class,
    Background,
    Abilities,
    Skills,
    Review,
}

impl Step {
    pub const PC: [Step; 7] = [
        Step::Basics,
        Step::Race,
        Step::Class,
        Step::Background,
        Step::Abilities,
        Step::Skills,
        Step::Review,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Step::Basics => "Basics",
            Step::Race => "Race",
            Step::Class => "Class",
            Step::Background => "Background",
            Step::Abilities => "Ability scores",
            Step::Skills => "Skills",
            Step::Review => "Review",
        }
    }
}

/// What a step needs to go on.
#[derive(Debug, Clone, Default)]
pub struct Draft {
    pub name: String,
    pub player_name: String,
    pub race: Option<(String, String)>,
    pub class: Option<(String, String)>,
    pub background: Option<(String, String)>,
    pub method: Option<Method>,
    pub scores: [i32; 6],
    pub choose: usize,
    pub picked: usize,
}

/// Whether the PC wizard may leave `step`.
pub fn can_proceed(step: Step, d: &Draft) -> bool {
    match step {
        Step::Basics => !d.name.trim().is_empty() && !d.player_name.trim().is_empty(),
        Step::Race => d.race.is_some(),
        Step::Class => d.class.is_some(),
        Step::Background => d.background.is_some(),
        Step::Abilities => scores_valid(d.method.unwrap_or(Method::Standard), &d.scores),
        Step::Skills => skills_valid(d.choose, d.picked),
        Step::Review => true,
    }
}

/// A catalog pick as one select value: "name|source".
pub fn pick_key(name: &str, source: &str) -> String {
    format!("{name}|{source}")
}

pub fn parse_pick(key: &str) -> Option<(String, String)> {
    let (n, s) = key.rsplit_once('|')?;
    (!n.is_empty() && !s.is_empty()).then(|| (n.to_string(), s.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_array() {
        assert!(scores_valid(Method::Standard, &[8, 10, 12, 13, 14, 15]));
        assert!(!scores_valid(Method::Standard, &[15, 15, 13, 12, 10, 8]));
        assert!(!scores_valid(Method::Standard, &[0; 6]));
        // A value held elsewhere is not offered.
        let s = [15, 0, 0, 0, 0, 0];
        assert!(!standard_options(&s, 1).contains(&15));
        assert!(standard_options(&s, 0).contains(&15));
    }

    #[test]
    fn point_buy() {
        assert_eq!(points_left(&[8; 6]), 27);
        // 15, 15, 15 = 27 points.
        let full = [15, 15, 15, 8, 8, 8];
        assert_eq!(points_left(&full), 0);
        assert!(scores_valid(Method::PointBuy, &full));
        assert!(!scores_valid(Method::PointBuy, &[8; 6]), "points left over");
        assert!(!can_raise(Method::PointBuy, &full, 3), "no points left");
        assert!(
            !can_raise(Method::PointBuy, &[15, 8, 8, 8, 8, 8], 0),
            "15 is the top"
        );
        // 13 → 14 costs 2: not with 1 point left.
        let s = [13, 15, 15, 9, 8, 8];
        assert_eq!(points_left(&s), 27 - 5 - 9 - 9 - 1);
        assert_eq!(points_left(&s), 3);
        assert!(can_raise(Method::PointBuy, &s, 0));
        assert!(!can_lower(Method::PointBuy, &[8; 6], 0));
    }

    #[test]
    fn manual() {
        assert!(scores_valid(Method::Manual, &[3, 18, 10, 10, 10, 10]));
        assert!(!scores_valid(Method::Manual, &[2, 10, 10, 10, 10, 10]));
        assert!(!can_raise(Method::Manual, &[18; 6], 0));
        assert!(!can_lower(Method::Manual, &[3; 6], 0));
    }

    #[test]
    fn modifiers() {
        assert_eq!(signed(modifier(15)), "+2");
        assert_eq!(signed(modifier(8)), "-1");
        assert_eq!(signed(modifier(10)), "+0");
        assert_eq!(modifier(3), -4);
    }

    #[test]
    fn steps() {
        let mut d = Draft::default();
        assert!(!can_proceed(Step::Basics, &d));
        d.name = "Robin".into();
        d.player_name = "Sam".into();
        assert!(can_proceed(Step::Basics, &d));
        assert!(!can_proceed(Step::Race, &d));
        d.race = Some(("Human".into(), "PHB".into()));
        assert!(can_proceed(Step::Race, &d));
        d.method = Some(Method::PointBuy);
        d.scores = [15, 15, 15, 8, 8, 8];
        assert!(can_proceed(Step::Abilities, &d));
        d.choose = 2;
        d.picked = 1;
        assert!(!can_proceed(Step::Skills, &d));
        d.picked = 2;
        assert!(can_proceed(Step::Skills, &d));
    }

    #[test]
    fn picks() {
        assert_eq!(
            parse_pick(&pick_key("Half-Elf", "PHB")),
            Some(("Half-Elf".into(), "PHB".into()))
        );
        assert_eq!(parse_pick(""), None);
        assert_eq!(Method::from_key(Method::PointBuy.key()), Method::PointBuy);
    }
}
