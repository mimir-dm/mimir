#![allow(clippy::redundant_closure_call)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::match_single_binding)]
#![allow(clippy::clone_on_copy)]

#[doc = r" Error types."]
pub mod error {
    #[doc = r" Error from a `TryFrom` or `FromStr` implementation."]
    pub struct ConversionError(::std::borrow::Cow<'static, str>);
    impl ::std::error::Error for ConversionError {}
    impl ::std::fmt::Display for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Display::fmt(&self.0, f)
        }
    }
    impl ::std::fmt::Debug for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Debug::fmt(&self.0, f)
        }
    }
    impl From<&'static str> for ConversionError {
        fn from(value: &'static str) -> Self {
            Self(value.into())
        }
    }
    impl From<String> for ConversionError {
        fn from(value: String) -> Self {
            Self(value.into())
        }
    }
}
#[doc = "D&D 5e ability scores"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"D&D 5e ability scores\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"str\","]
#[doc = "    \"dex\","]
#[doc = "    \"con\","]
#[doc = "    \"int\","]
#[doc = "    \"wis\","]
#[doc = "    \"cha\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum AbilityScore {
    #[serde(rename = "str")]
    Str,
    #[serde(rename = "dex")]
    Dex,
    #[serde(rename = "con")]
    Con,
    #[serde(rename = "int")]
    Int,
    #[serde(rename = "wis")]
    Wis,
    #[serde(rename = "cha")]
    Cha,
}
impl ::std::convert::From<&Self> for AbilityScore {
    fn from(value: &AbilityScore) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for AbilityScore {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Str => f.write_str("str"),
            Self::Dex => f.write_str("dex"),
            Self::Con => f.write_str("con"),
            Self::Int => f.write_str("int"),
            Self::Wis => f.write_str("wis"),
            Self::Cha => f.write_str("cha"),
        }
    }
}
impl ::std::str::FromStr for AbilityScore {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "str" => Ok(Self::Str),
            "dex" => Ok(Self::Dex),
            "con" => Ok(Self::Con),
            "int" => Ok(Self::Int),
            "wis" => Ok(Self::Wis),
            "cha" => Ok(Self::Cha),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for AbilityScore {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for AbilityScore {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for AbilityScore {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Single alignment component: L=Lawful, N=Neutral, NX=Neutral (law/chaos), NY=Neutral (good/evil), C=Chaotic, G=Good, E=Evil, U=Unaligned, A=Any"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Single alignment component: L=Lawful, N=Neutral, NX=Neutral (law/chaos), NY=Neutral (good/evil), C=Chaotic, G=Good, E=Evil, U=Unaligned, A=Any\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"L\","]
#[doc = "    \"N\","]
#[doc = "    \"NX\","]
#[doc = "    \"NY\","]
#[doc = "    \"C\","]
#[doc = "    \"G\","]
#[doc = "    \"E\","]
#[doc = "    \"U\","]
#[doc = "    \"A\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum Alignment {
    L,
    N,
    #[serde(rename = "NX")]
    Nx,
    #[serde(rename = "NY")]
    Ny,
    C,
    G,
    E,
    U,
    A,
}
impl ::std::convert::From<&Self> for Alignment {
    fn from(value: &Alignment) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for Alignment {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::L => f.write_str("L"),
            Self::N => f.write_str("N"),
            Self::Nx => f.write_str("NX"),
            Self::Ny => f.write_str("NY"),
            Self::C => f.write_str("C"),
            Self::G => f.write_str("G"),
            Self::E => f.write_str("E"),
            Self::U => f.write_str("U"),
            Self::A => f.write_str("A"),
        }
    }
}
impl ::std::str::FromStr for Alignment {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "L" => Ok(Self::L),
            "N" => Ok(Self::N),
            "NX" => Ok(Self::Nx),
            "NY" => Ok(Self::Ny),
            "C" => Ok(Self::C),
            "G" => Ok(Self::G),
            "E" => Ok(Self::E),
            "U" => Ok(Self::U),
            "A" => Ok(Self::A),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for Alignment {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Alignment {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Alignment {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Self-contained schema demonstrating typify generation from 5etools-style definitions"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"alignment_poc.json\","]
#[doc = "  \"title\": \"Alignment Types (Proof of Concept)\","]
#[doc = "  \"description\": \"Self-contained schema demonstrating typify generation from 5etools-style definitions\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"alignment\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/alignment\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"conditionImmunities\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/condition\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"damageImmunities\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/damageType\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"savingThrows\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"additionalProperties\": {"]
#[doc = "        \"type\": \"integer\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"size\": {"]
#[doc = "      \"$ref\": \"#/$defs/size\""]
#[doc = "    },"]
#[doc = "    \"skills\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"additionalProperties\": {"]
#[doc = "        \"type\": \"integer\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"$ref\": \"#/$defs/creatureType\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct AlignmentTypesProofOfConcept {
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub alignment: ::std::vec::Vec<Alignment>,
    #[serde(
        rename = "conditionImmunities",
        default,
        skip_serializing_if = "::std::vec::Vec::is_empty"
    )]
    pub condition_immunities: ::std::vec::Vec<Condition>,
    #[serde(
        rename = "damageImmunities",
        default,
        skip_serializing_if = "::std::vec::Vec::is_empty"
    )]
    pub damage_immunities: ::std::vec::Vec<DamageType>,
    #[serde(
        rename = "savingThrows",
        default,
        skip_serializing_if = ":: std :: collections :: HashMap::is_empty"
    )]
    pub saving_throws: ::std::collections::HashMap<::std::string::String, i64>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub size: ::std::option::Option<Size>,
    #[serde(
        default,
        skip_serializing_if = ":: std :: collections :: HashMap::is_empty"
    )]
    pub skills: ::std::collections::HashMap<::std::string::String, i64>,
    #[serde(
        rename = "type",
        default,
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    pub type_: ::std::option::Option<CreatureType>,
}
impl ::std::convert::From<&AlignmentTypesProofOfConcept> for AlignmentTypesProofOfConcept {
    fn from(value: &AlignmentTypesProofOfConcept) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for AlignmentTypesProofOfConcept {
    fn default() -> Self {
        Self {
            alignment: Default::default(),
            condition_immunities: Default::default(),
            damage_immunities: Default::default(),
            saving_throws: Default::default(),
            size: Default::default(),
            skills: Default::default(),
            type_: Default::default(),
        }
    }
}
impl AlignmentTypesProofOfConcept {
    pub fn builder() -> builder::AlignmentTypesProofOfConcept {
        Default::default()
    }
}
#[doc = "D&D 5e conditions"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"D&D 5e conditions\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"blinded\","]
#[doc = "    \"charmed\","]
#[doc = "    \"deafened\","]
#[doc = "    \"exhaustion\","]
#[doc = "    \"frightened\","]
#[doc = "    \"grappled\","]
#[doc = "    \"incapacitated\","]
#[doc = "    \"invisible\","]
#[doc = "    \"paralyzed\","]
#[doc = "    \"petrified\","]
#[doc = "    \"poisoned\","]
#[doc = "    \"prone\","]
#[doc = "    \"restrained\","]
#[doc = "    \"stunned\","]
#[doc = "    \"unconscious\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum Condition {
    #[serde(rename = "blinded")]
    Blinded,
    #[serde(rename = "charmed")]
    Charmed,
    #[serde(rename = "deafened")]
    Deafened,
    #[serde(rename = "exhaustion")]
    Exhaustion,
    #[serde(rename = "frightened")]
    Frightened,
    #[serde(rename = "grappled")]
    Grappled,
    #[serde(rename = "incapacitated")]
    Incapacitated,
    #[serde(rename = "invisible")]
    Invisible,
    #[serde(rename = "paralyzed")]
    Paralyzed,
    #[serde(rename = "petrified")]
    Petrified,
    #[serde(rename = "poisoned")]
    Poisoned,
    #[serde(rename = "prone")]
    Prone,
    #[serde(rename = "restrained")]
    Restrained,
    #[serde(rename = "stunned")]
    Stunned,
    #[serde(rename = "unconscious")]
    Unconscious,
}
impl ::std::convert::From<&Self> for Condition {
    fn from(value: &Condition) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for Condition {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Blinded => f.write_str("blinded"),
            Self::Charmed => f.write_str("charmed"),
            Self::Deafened => f.write_str("deafened"),
            Self::Exhaustion => f.write_str("exhaustion"),
            Self::Frightened => f.write_str("frightened"),
            Self::Grappled => f.write_str("grappled"),
            Self::Incapacitated => f.write_str("incapacitated"),
            Self::Invisible => f.write_str("invisible"),
            Self::Paralyzed => f.write_str("paralyzed"),
            Self::Petrified => f.write_str("petrified"),
            Self::Poisoned => f.write_str("poisoned"),
            Self::Prone => f.write_str("prone"),
            Self::Restrained => f.write_str("restrained"),
            Self::Stunned => f.write_str("stunned"),
            Self::Unconscious => f.write_str("unconscious"),
        }
    }
}
impl ::std::str::FromStr for Condition {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "blinded" => Ok(Self::Blinded),
            "charmed" => Ok(Self::Charmed),
            "deafened" => Ok(Self::Deafened),
            "exhaustion" => Ok(Self::Exhaustion),
            "frightened" => Ok(Self::Frightened),
            "grappled" => Ok(Self::Grappled),
            "incapacitated" => Ok(Self::Incapacitated),
            "invisible" => Ok(Self::Invisible),
            "paralyzed" => Ok(Self::Paralyzed),
            "petrified" => Ok(Self::Petrified),
            "poisoned" => Ok(Self::Poisoned),
            "prone" => Ok(Self::Prone),
            "restrained" => Ok(Self::Restrained),
            "stunned" => Ok(Self::Stunned),
            "unconscious" => Ok(Self::Unconscious),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for Condition {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Condition {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Condition {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "D&D 5e creature types"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"D&D 5e creature types\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"aberration\","]
#[doc = "    \"beast\","]
#[doc = "    \"celestial\","]
#[doc = "    \"construct\","]
#[doc = "    \"dragon\","]
#[doc = "    \"elemental\","]
#[doc = "    \"fey\","]
#[doc = "    \"fiend\","]
#[doc = "    \"giant\","]
#[doc = "    \"humanoid\","]
#[doc = "    \"monstrosity\","]
#[doc = "    \"ooze\","]
#[doc = "    \"plant\","]
#[doc = "    \"undead\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum CreatureType {
    #[serde(rename = "aberration")]
    Aberration,
    #[serde(rename = "beast")]
    Beast,
    #[serde(rename = "celestial")]
    Celestial,
    #[serde(rename = "construct")]
    Construct,
    #[serde(rename = "dragon")]
    Dragon,
    #[serde(rename = "elemental")]
    Elemental,
    #[serde(rename = "fey")]
    Fey,
    #[serde(rename = "fiend")]
    Fiend,
    #[serde(rename = "giant")]
    Giant,
    #[serde(rename = "humanoid")]
    Humanoid,
    #[serde(rename = "monstrosity")]
    Monstrosity,
    #[serde(rename = "ooze")]
    Ooze,
    #[serde(rename = "plant")]
    Plant,
    #[serde(rename = "undead")]
    Undead,
}
impl ::std::convert::From<&Self> for CreatureType {
    fn from(value: &CreatureType) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for CreatureType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Aberration => f.write_str("aberration"),
            Self::Beast => f.write_str("beast"),
            Self::Celestial => f.write_str("celestial"),
            Self::Construct => f.write_str("construct"),
            Self::Dragon => f.write_str("dragon"),
            Self::Elemental => f.write_str("elemental"),
            Self::Fey => f.write_str("fey"),
            Self::Fiend => f.write_str("fiend"),
            Self::Giant => f.write_str("giant"),
            Self::Humanoid => f.write_str("humanoid"),
            Self::Monstrosity => f.write_str("monstrosity"),
            Self::Ooze => f.write_str("ooze"),
            Self::Plant => f.write_str("plant"),
            Self::Undead => f.write_str("undead"),
        }
    }
}
impl ::std::str::FromStr for CreatureType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "aberration" => Ok(Self::Aberration),
            "beast" => Ok(Self::Beast),
            "celestial" => Ok(Self::Celestial),
            "construct" => Ok(Self::Construct),
            "dragon" => Ok(Self::Dragon),
            "elemental" => Ok(Self::Elemental),
            "fey" => Ok(Self::Fey),
            "fiend" => Ok(Self::Fiend),
            "giant" => Ok(Self::Giant),
            "humanoid" => Ok(Self::Humanoid),
            "monstrosity" => Ok(Self::Monstrosity),
            "ooze" => Ok(Self::Ooze),
            "plant" => Ok(Self::Plant),
            "undead" => Ok(Self::Undead),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CreatureType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CreatureType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CreatureType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "D&D 5e damage types"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"D&D 5e damage types\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"acid\","]
#[doc = "    \"bludgeoning\","]
#[doc = "    \"cold\","]
#[doc = "    \"fire\","]
#[doc = "    \"force\","]
#[doc = "    \"lightning\","]
#[doc = "    \"necrotic\","]
#[doc = "    \"piercing\","]
#[doc = "    \"poison\","]
#[doc = "    \"psychic\","]
#[doc = "    \"radiant\","]
#[doc = "    \"slashing\","]
#[doc = "    \"thunder\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum DamageType {
    #[serde(rename = "acid")]
    Acid,
    #[serde(rename = "bludgeoning")]
    Bludgeoning,
    #[serde(rename = "cold")]
    Cold,
    #[serde(rename = "fire")]
    Fire,
    #[serde(rename = "force")]
    Force,
    #[serde(rename = "lightning")]
    Lightning,
    #[serde(rename = "necrotic")]
    Necrotic,
    #[serde(rename = "piercing")]
    Piercing,
    #[serde(rename = "poison")]
    Poison,
    #[serde(rename = "psychic")]
    Psychic,
    #[serde(rename = "radiant")]
    Radiant,
    #[serde(rename = "slashing")]
    Slashing,
    #[serde(rename = "thunder")]
    Thunder,
}
impl ::std::convert::From<&Self> for DamageType {
    fn from(value: &DamageType) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for DamageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Acid => f.write_str("acid"),
            Self::Bludgeoning => f.write_str("bludgeoning"),
            Self::Cold => f.write_str("cold"),
            Self::Fire => f.write_str("fire"),
            Self::Force => f.write_str("force"),
            Self::Lightning => f.write_str("lightning"),
            Self::Necrotic => f.write_str("necrotic"),
            Self::Piercing => f.write_str("piercing"),
            Self::Poison => f.write_str("poison"),
            Self::Psychic => f.write_str("psychic"),
            Self::Radiant => f.write_str("radiant"),
            Self::Slashing => f.write_str("slashing"),
            Self::Thunder => f.write_str("thunder"),
        }
    }
}
impl ::std::str::FromStr for DamageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "acid" => Ok(Self::Acid),
            "bludgeoning" => Ok(Self::Bludgeoning),
            "cold" => Ok(Self::Cold),
            "fire" => Ok(Self::Fire),
            "force" => Ok(Self::Force),
            "lightning" => Ok(Self::Lightning),
            "necrotic" => Ok(Self::Necrotic),
            "piercing" => Ok(Self::Piercing),
            "poison" => Ok(Self::Poison),
            "psychic" => Ok(Self::Psychic),
            "radiant" => Ok(Self::Radiant),
            "slashing" => Ok(Self::Slashing),
            "thunder" => Ok(Self::Thunder),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for DamageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for DamageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for DamageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Creature size categories"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Creature size categories\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"T\","]
#[doc = "    \"S\","]
#[doc = "    \"M\","]
#[doc = "    \"L\","]
#[doc = "    \"H\","]
#[doc = "    \"G\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum Size {
    T,
    S,
    M,
    L,
    H,
    G,
}
impl ::std::convert::From<&Self> for Size {
    fn from(value: &Size) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for Size {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::T => f.write_str("T"),
            Self::S => f.write_str("S"),
            Self::M => f.write_str("M"),
            Self::L => f.write_str("L"),
            Self::H => f.write_str("H"),
            Self::G => f.write_str("G"),
        }
    }
}
impl ::std::str::FromStr for Size {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "T" => Ok(Self::T),
            "S" => Ok(Self::S),
            "M" => Ok(Self::M),
            "L" => Ok(Self::L),
            "H" => Ok(Self::H),
            "G" => Ok(Self::G),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for Size {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Size {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Size {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "D&D 5e skills"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"D&D 5e skills\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"acrobatics\","]
#[doc = "    \"animal handling\","]
#[doc = "    \"arcana\","]
#[doc = "    \"athletics\","]
#[doc = "    \"deception\","]
#[doc = "    \"history\","]
#[doc = "    \"insight\","]
#[doc = "    \"intimidation\","]
#[doc = "    \"investigation\","]
#[doc = "    \"medicine\","]
#[doc = "    \"nature\","]
#[doc = "    \"perception\","]
#[doc = "    \"performance\","]
#[doc = "    \"persuasion\","]
#[doc = "    \"religion\","]
#[doc = "    \"sleight of hand\","]
#[doc = "    \"stealth\","]
#[doc = "    \"survival\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum Skill {
    #[serde(rename = "acrobatics")]
    Acrobatics,
    #[serde(rename = "animal handling")]
    AnimalHandling,
    #[serde(rename = "arcana")]
    Arcana,
    #[serde(rename = "athletics")]
    Athletics,
    #[serde(rename = "deception")]
    Deception,
    #[serde(rename = "history")]
    History,
    #[serde(rename = "insight")]
    Insight,
    #[serde(rename = "intimidation")]
    Intimidation,
    #[serde(rename = "investigation")]
    Investigation,
    #[serde(rename = "medicine")]
    Medicine,
    #[serde(rename = "nature")]
    Nature,
    #[serde(rename = "perception")]
    Perception,
    #[serde(rename = "performance")]
    Performance,
    #[serde(rename = "persuasion")]
    Persuasion,
    #[serde(rename = "religion")]
    Religion,
    #[serde(rename = "sleight of hand")]
    SleightOfHand,
    #[serde(rename = "stealth")]
    Stealth,
    #[serde(rename = "survival")]
    Survival,
}
impl ::std::convert::From<&Self> for Skill {
    fn from(value: &Skill) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for Skill {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Acrobatics => f.write_str("acrobatics"),
            Self::AnimalHandling => f.write_str("animal handling"),
            Self::Arcana => f.write_str("arcana"),
            Self::Athletics => f.write_str("athletics"),
            Self::Deception => f.write_str("deception"),
            Self::History => f.write_str("history"),
            Self::Insight => f.write_str("insight"),
            Self::Intimidation => f.write_str("intimidation"),
            Self::Investigation => f.write_str("investigation"),
            Self::Medicine => f.write_str("medicine"),
            Self::Nature => f.write_str("nature"),
            Self::Perception => f.write_str("perception"),
            Self::Performance => f.write_str("performance"),
            Self::Persuasion => f.write_str("persuasion"),
            Self::Religion => f.write_str("religion"),
            Self::SleightOfHand => f.write_str("sleight of hand"),
            Self::Stealth => f.write_str("stealth"),
            Self::Survival => f.write_str("survival"),
        }
    }
}
impl ::std::str::FromStr for Skill {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "acrobatics" => Ok(Self::Acrobatics),
            "animal handling" => Ok(Self::AnimalHandling),
            "arcana" => Ok(Self::Arcana),
            "athletics" => Ok(Self::Athletics),
            "deception" => Ok(Self::Deception),
            "history" => Ok(Self::History),
            "insight" => Ok(Self::Insight),
            "intimidation" => Ok(Self::Intimidation),
            "investigation" => Ok(Self::Investigation),
            "medicine" => Ok(Self::Medicine),
            "nature" => Ok(Self::Nature),
            "perception" => Ok(Self::Perception),
            "performance" => Ok(Self::Performance),
            "persuasion" => Ok(Self::Persuasion),
            "religion" => Ok(Self::Religion),
            "sleight of hand" => Ok(Self::SleightOfHand),
            "stealth" => Ok(Self::Stealth),
            "survival" => Ok(Self::Survival),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for Skill {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Skill {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Skill {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct AlignmentTypesProofOfConcept {
        alignment: ::std::result::Result<::std::vec::Vec<super::Alignment>, ::std::string::String>,
        condition_immunities:
            ::std::result::Result<::std::vec::Vec<super::Condition>, ::std::string::String>,
        damage_immunities:
            ::std::result::Result<::std::vec::Vec<super::DamageType>, ::std::string::String>,
        saving_throws: ::std::result::Result<
            ::std::collections::HashMap<::std::string::String, i64>,
            ::std::string::String,
        >,
        size: ::std::result::Result<::std::option::Option<super::Size>, ::std::string::String>,
        skills: ::std::result::Result<
            ::std::collections::HashMap<::std::string::String, i64>,
            ::std::string::String,
        >,
        type_: ::std::result::Result<
            ::std::option::Option<super::CreatureType>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for AlignmentTypesProofOfConcept {
        fn default() -> Self {
            Self {
                alignment: Ok(Default::default()),
                condition_immunities: Ok(Default::default()),
                damage_immunities: Ok(Default::default()),
                saving_throws: Ok(Default::default()),
                size: Ok(Default::default()),
                skills: Ok(Default::default()),
                type_: Ok(Default::default()),
            }
        }
    }
    impl AlignmentTypesProofOfConcept {
        pub fn alignment<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Alignment>>,
            T::Error: ::std::fmt::Display,
        {
            self.alignment = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for alignment: {}", e));
            self
        }
        pub fn condition_immunities<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Condition>>,
            T::Error: ::std::fmt::Display,
        {
            self.condition_immunities = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for condition_immunities: {}",
                    e
                )
            });
            self
        }
        pub fn damage_immunities<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::DamageType>>,
            T::Error: ::std::fmt::Display,
        {
            self.damage_immunities = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for damage_immunities: {}",
                    e
                )
            });
            self
        }
        pub fn saving_throws<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::collections::HashMap<::std::string::String, i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.saving_throws = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for saving_throws: {}", e));
            self
        }
        pub fn size<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Size>>,
            T::Error: ::std::fmt::Display,
        {
            self.size = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for size: {}", e));
            self
        }
        pub fn skills<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::collections::HashMap<::std::string::String, i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.skills = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for skills: {}", e));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::CreatureType>>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<AlignmentTypesProofOfConcept> for super::AlignmentTypesProofOfConcept {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AlignmentTypesProofOfConcept,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                alignment: value.alignment?,
                condition_immunities: value.condition_immunities?,
                damage_immunities: value.damage_immunities?,
                saving_throws: value.saving_throws?,
                size: value.size?,
                skills: value.skills?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::AlignmentTypesProofOfConcept> for AlignmentTypesProofOfConcept {
        fn from(value: super::AlignmentTypesProofOfConcept) -> Self {
            Self {
                alignment: Ok(value.alignment),
                condition_immunities: Ok(value.condition_immunities),
                damage_immunities: Ok(value.damage_immunities),
                saving_throws: Ok(value.saving_throws),
                size: Ok(value.size),
                skills: Ok(value.skills),
                type_: Ok(value.type_),
            }
        }
    }
}
