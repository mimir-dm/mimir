//! Godot type wrappers with custom serde (de)serialization.
//!
//! Dungeondraft uses Godot's serialization format for certain types.
//! These are encoded as strings in the JSON, e.g. `"Vector2( 100, 200 )"`.

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A 2D vector, serialized as `"Vector2( x, y )"`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl Serialize for Vector2 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("Vector2( {}, {} )", format_f64(self.x), format_f64(self.y)))
    }
}

impl<'de> Deserialize<'de> for Vector2 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        parse_vector2(&s).map_err(de::Error::custom)
    }
}

fn parse_vector2(s: &str) -> Result<Vector2, String> {
    let s = s.trim();
    if s == "null" {
        return Ok(Vector2::zero());
    }
    let inner = s
        .strip_prefix("Vector2(")
        .or_else(|| s.strip_prefix("Vector2 ("))
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| format!("Invalid Vector2: {}", s))?;
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() != 2 {
        return Err(format!("Vector2 needs 2 components, got {}", parts.len()));
    }
    Ok(Vector2 {
        x: parts[0].trim().parse().map_err(|e| format!("Bad x: {}", e))?,
        y: parts[1].trim().parse().map_err(|e| format!("Bad y: {}", e))?,
    })
}

/// A pool of bytes, serialized as `"PoolByteArray( 0, 1, 2, ... )"`.
#[derive(Debug, Clone, PartialEq)]
pub struct PoolByteArray(pub Vec<u8>);

impl PoolByteArray {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from_vec(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl Default for PoolByteArray {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for PoolByteArray {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let inner = self
            .0
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        serializer.serialize_str(&format!("PoolByteArray( {} )", inner))
    }
}

impl<'de> Deserialize<'de> for PoolByteArray {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        parse_pool_byte_array(&s).map_err(de::Error::custom)
    }
}

fn parse_pool_byte_array(s: &str) -> Result<PoolByteArray, String> {
    let inner = s
        .trim()
        .strip_prefix("PoolByteArray(")
        .or_else(|| s.trim().strip_prefix("PoolByteArray ("))
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| format!("Invalid PoolByteArray: {}...", &s[..s.len().min(40)]))?;
    let trimmed = inner.trim();
    if trimmed.is_empty() {
        return Ok(PoolByteArray::new());
    }
    let bytes = trimmed
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<u8>()
                .map_err(|e| format!("Bad byte '{}': {}", s.trim(), e))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PoolByteArray(bytes))
}

/// A pool of 2D vectors, serialized as `"PoolVector2Array( x1, y1, x2, y2, ... )"`.
#[derive(Debug, Clone, PartialEq)]
pub struct PoolVector2Array(pub Vec<Vector2>);

impl PoolVector2Array {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from_points(points: Vec<Vector2>) -> Self {
        Self(points)
    }
}

impl Default for PoolVector2Array {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for PoolVector2Array {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let inner = self
            .0
            .iter()
            .flat_map(|v| [format_f64(v.x), format_f64(v.y)])
            .collect::<Vec<_>>()
            .join(", ");
        serializer.serialize_str(&format!("PoolVector2Array( {} )", inner))
    }
}

impl<'de> Deserialize<'de> for PoolVector2Array {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        parse_pool_vector2_array(&s).map_err(de::Error::custom)
    }
}

fn parse_pool_vector2_array(s: &str) -> Result<PoolVector2Array, String> {
    let inner = s
        .trim()
        .strip_prefix("PoolVector2Array(")
        .or_else(|| s.trim().strip_prefix("PoolVector2Array ("))
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| format!("Invalid PoolVector2Array: {}...", &s[..s.len().min(40)]))?;
    let trimmed = inner.trim();
    if trimmed.is_empty() {
        return Ok(PoolVector2Array::new());
    }
    let nums: Vec<f64> = trimmed
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<f64>()
                .map_err(|e| format!("Bad float '{}': {}", s.trim(), e))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if nums.len() % 2 != 0 {
        return Err(format!(
            "PoolVector2Array has odd number of floats: {}",
            nums.len()
        ));
    }
    let points = nums
        .chunks(2)
        .map(|c| Vector2::new(c[0], c[1]))
        .collect();
    Ok(PoolVector2Array(points))
}

/// A pool of integers, serialized as `"PoolIntArray( 1, 2, 3, ... )"`.
#[derive(Debug, Clone, PartialEq)]
pub struct PoolIntArray(pub Vec<i32>);

impl PoolIntArray {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn filled(value: i32, count: usize) -> Self {
        Self(vec![value; count])
    }
}

impl Default for PoolIntArray {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for PoolIntArray {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let inner = self
            .0
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        serializer.serialize_str(&format!("PoolIntArray( {} )", inner))
    }
}

impl<'de> Deserialize<'de> for PoolIntArray {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        parse_pool_int_array(&s).map_err(de::Error::custom)
    }
}

fn parse_pool_int_array(s: &str) -> Result<PoolIntArray, String> {
    let inner = s
        .trim()
        .strip_prefix("PoolIntArray(")
        .or_else(|| s.trim().strip_prefix("PoolIntArray ("))
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| format!("Invalid PoolIntArray: {}...", &s[..s.len().min(40)]))?;
    let trimmed = inner.trim();
    if trimmed.is_empty() {
        return Ok(PoolIntArray::new());
    }
    let ints = trimmed
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<i32>()
                .map_err(|e| format!("Bad int '{}': {}", s.trim(), e))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PoolIntArray(ints))
}

/// Format an f64 value, using integer format when there's no fractional part.
fn format_f64(v: f64) -> String {
    if v == v.floor() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

/// A nullable Vector2 field — DD uses the string `"null"` instead of JSON null.
#[derive(Debug, Clone, PartialEq)]
pub enum NullableVector2 {
    Null,
    Value(Vector2),
}

impl Serialize for NullableVector2 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            NullableVector2::Null => serializer.serialize_str("null"),
            NullableVector2::Value(v) => v.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for NullableVector2 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        if s.trim() == "null" {
            Ok(NullableVector2::Null)
        } else {
            parse_vector2(&s)
                .map(NullableVector2::Value)
                .map_err(de::Error::custom)
        }
    }
}

impl fmt::Display for Vector2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vector2( {}, {} )", format_f64(self.x), format_f64(self.y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector2_roundtrip() {
        let v = Vector2::new(100.0, 200.0);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, r#""Vector2( 100, 200 )""#);
        let parsed: Vector2 = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, v);
    }

    #[test]
    fn test_vector2_float() {
        let v = Vector2::new(100.5, 200.75);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, r#""Vector2( 100.5, 200.75 )""#);
        let parsed: Vector2 = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, v);
    }

    #[test]
    fn test_pool_byte_array_roundtrip() {
        let arr = PoolByteArray::from_vec(vec![255, 0, 128, 64]);
        let json = serde_json::to_string(&arr).unwrap();
        assert_eq!(json, r#""PoolByteArray( 255, 0, 128, 64 )""#);
        let parsed: PoolByteArray = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, arr);
    }

    #[test]
    fn test_pool_byte_array_empty() {
        let arr = PoolByteArray::new();
        let json = serde_json::to_string(&arr).unwrap();
        assert_eq!(json, r#""PoolByteArray(  )""#);
        let parsed: PoolByteArray = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, arr);
    }

    #[test]
    fn test_pool_vector2_array_roundtrip() {
        let arr = PoolVector2Array::from_points(vec![
            Vector2::new(10.0, 20.0),
            Vector2::new(30.0, 40.0),
        ]);
        let json = serde_json::to_string(&arr).unwrap();
        assert_eq!(json, r#""PoolVector2Array( 10, 20, 30, 40 )""#);
        let parsed: PoolVector2Array = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, arr);
    }

    #[test]
    fn test_pool_int_array_roundtrip() {
        let arr = PoolIntArray(vec![-1, -1, 0, 1]);
        let json = serde_json::to_string(&arr).unwrap();
        assert_eq!(json, r#""PoolIntArray( -1, -1, 0, 1 )""#);
        let parsed: PoolIntArray = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, arr);
    }

    #[test]
    fn test_nullable_vector2() {
        let null_v: NullableVector2 = serde_json::from_str(r#""null""#).unwrap();
        assert_eq!(null_v, NullableVector2::Null);

        let val: NullableVector2 = serde_json::from_str(r#""Vector2( 10, 20 )""#).unwrap();
        assert_eq!(val, NullableVector2::Value(Vector2::new(10.0, 20.0)));
    }

    #[test]
    fn test_parse_with_extra_spaces() {
        // DD sometimes has inconsistent spacing
        let v: Vector2 = serde_json::from_str(r#""Vector2( 4384, 4656 )""#).unwrap();
        assert_eq!(v, Vector2::new(4384.0, 4656.0));
    }
}
