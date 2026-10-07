//! Scoped JSONObject ordering and Android JSONStringer conversion reference.
//! Strict serde lexical parsing remains unchanged. Source: pinned AOSP JSON.java,
//! JSONObject/JSONStringer in docs/protocol/mex.md. No global preserve_order flag.

use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::Number;
#[cfg(test)]
use serde_json::Value;
use std::{collections::HashMap, fmt};

// No Debug/Serialize: structured metadata can contain private content.
#[derive(Clone)]
pub(super) enum OrderedJson {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Self>),
    Object(Vec<(String, Self)>),
}

impl OrderedJson {
    pub(super) fn get(&self, name: &str) -> Option<&Self> {
        match self {
            Self::Object(entries) => entries
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value),
            _ => None,
        }
    }
    pub(super) fn as_array(&self) -> Option<&[Self]> {
        if let Self::Array(values) = self {
            Some(values)
        } else {
            None
        }
    }
    pub(super) fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
    pub(super) fn platform_string(&self) -> Option<String> {
        // JSON.toString uses String.valueOf for scalars; containers call their
        // JSONStringer serializer, whose number formatting is different.
        match self {
            Self::String(value) => Some(value.clone()),
            Self::Number(value) => number_scalar(value),
            Self::Bool(value) => Some(value.to_string()),
            Self::Null => Some("null".into()),
            Self::Array(_) | Self::Object(_) => {
                let mut out = String::new();
                self.append_json(&mut out)?;
                Some(out)
            }
        }
    }
    fn append_json(&self, out: &mut String) -> Option<()> {
        match self {
            Self::Null => out.push_str("null"),
            Self::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
            Self::Number(value) => out.push_str(&number_container(value)?),
            Self::String(value) => quote(value, out),
            Self::Array(values) => {
                out.push('[');
                for (index, value) in values.iter().enumerate() {
                    if index != 0 {
                        out.push(',');
                    }
                    value.append_json(out)?;
                }
                out.push(']');
            }
            Self::Object(entries) => {
                out.push('{');
                for (index, (key, value)) in entries.iter().enumerate() {
                    if index != 0 {
                        out.push(',');
                    }
                    quote(key, out);
                    out.push(':');
                    value.append_json(out)?;
                }
                out.push('}');
            }
        }
        Some(())
    }
}

fn quote(value: &str, out: &mut String) {
    // JSONStringer.string escapes slash too; printable Unicode stays literal.
    out.push('"');
    for c in value.chars() {
        match c {
            '"' | '\\' | '/' => {
                out.push('\\');
                out.push(c);
            }
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\x0c' => out.push_str("\\f"),
            c if c <= '\x1f' => {
                // Four lowercase hexadecimal digits, matching String.format.
                const HEX: &[u8; 16] = b"0123456789abcdef";
                out.push_str("\\u00");
                out.push(char::from(HEX[(c as usize) >> 4]));
                out.push(char::from(HEX[(c as usize) & 15]));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn number_scalar(number: &Number) -> Option<String> {
    if let Some(value) = number.as_i64() {
        Some(value.to_string())
    } else {
        number.as_f64().and_then(super::double_string::format)
    }
}
fn number_container(number: &Number) -> Option<String> {
    if let Some(value) = number.as_i64() {
        return Some(value.to_string());
    }
    let value = number.as_f64()?;
    if !value.is_finite() {
        return None;
    }
    // JSONObject.numberToString detects Double negative zero before its
    // integral reduction; Double.longValue saturates, then its f64 equality
    // test can also reduce +2^63 to the signed-long maximum string.
    if value.to_bits() == (-0.0f64).to_bits() {
        return Some("-0".into());
    }
    let integral = value as i64;
    if value == integral as f64 {
        Some(integral.to_string())
    } else {
        super::double_string::format(value)
    }
}

/// Parse strict JSON with scoped Android integer-negative-zero classification.
/// The main MEX parser validates the original input first, preserving its
/// existing lexical/depth checks and public data() representation.
pub(super) fn parse(input: &[u8]) -> Result<OrderedJson, serde_json::Error> {
    use std::borrow::Cow;
    let mut inside_string = false;
    let mut escaped = false;
    let mut copied_until = 0;
    let mut output = Vec::new();
    for (index, &byte) in input.iter().enumerate() {
        if inside_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                inside_string = false;
            }
            continue;
        }
        if byte == b'"' {
            inside_string = true;
            continue;
        }
        let token_start = index == 0
            || matches!(
                input[index - 1],
                b':' | b',' | b'[' | b' ' | b'\t' | b'\r' | b'\n'
            );
        let token_end = input
            .get(index + 2)
            .is_none_or(|b| matches!(b, b',' | b']' | b'}' | b' ' | b'\t' | b'\r' | b'\n'));
        if byte == b'-' && input.get(index + 1) == Some(&b'0') && token_start && token_end {
            // JSONTokener.readLiteral Integer/Long parsing consumes -0 as
            // integer zero. Do not alter -0.0, exponent signs or quoted text.
            output.extend_from_slice(&input[copied_until..index]);
            copied_until = index + 1;
        }
    }
    let normalized = if copied_until == 0 {
        Cow::Borrowed(input)
    } else {
        output.extend_from_slice(&input[copied_until..]);
        Cow::Owned(output)
    };
    serde_json::from_slice(&normalized)
}

impl<'de> Deserialize<'de> for OrderedJson {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct JsonVisitor;
        impl<'de> Visitor<'de> for JsonVisitor {
            type Value = OrderedJson;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a JSON value")
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(OrderedJson::Null)
            }
            fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(OrderedJson::Bool(value))
            }
            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(OrderedJson::Number(value.into()))
            }
            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(OrderedJson::Number(value.into()))
            }
            fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Self::Value, E> {
                Number::from_f64(value)
                    .map(OrderedJson::Number)
                    .ok_or_else(|| E::custom("nonfinite JSON number"))
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(OrderedJson::String(value.into()))
            }
            fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(OrderedJson::String(value))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element()? {
                    values.push(value);
                }
                Ok(OrderedJson::Array(values))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut entries: Vec<(String, OrderedJson)> = Vec::new();
                let mut positions: HashMap<String, usize> = HashMap::new();
                while let Some((key, value)) = map.next_entry::<String, OrderedJson>()? {
                    // JSONObject.put overwrites an existing value while
                    // LinkedHashMap retains its first insertion position.
                    if let Some(&index) = positions.get(&key) {
                        entries[index].1 = value;
                    } else {
                        positions.insert(key.clone(), entries.len());
                        entries.push((key, value));
                    }
                }
                Ok(OrderedJson::Object(entries))
            }
        }
        deserializer.deserialize_any(JsonVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordered_strings_match_executed_aosp_reference() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../fixtures/mex/2.26.37.71-json-string-reference.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let raw = case["raw"].as_str().unwrap();
            let value = parse(raw.as_bytes()).unwrap();
            let field = value.get("value");
            let default = field
                .map(|v| v.platform_string().unwrap())
                .unwrap_or_default();
            let nullable = field
                .filter(|v| !v.is_null())
                .map(|v| v.platform_string().unwrap());
            assert_eq!(
                default,
                case["expected"]["default_string"].as_str().unwrap(),
                "{raw}"
            );
            assert_eq!(
                nullable.as_deref(),
                case["expected"]["nullable_string"].as_str(),
                "{raw}"
            );
            assert_eq!(
                value.platform_string().as_deref(),
                case["expected"]["container"].as_str(),
                "{raw}"
            );
        }
    }
    #[test]
    fn container_double_spellings_match_the_independent_ieee_corpus() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../fixtures/mex/2.26.37.71-double-string-reference.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let bits = u64::from_str_radix(case["bits"].as_str().unwrap(), 16).unwrap();
            if let Some(number) = Number::from_f64(f64::from_bits(bits)) {
                assert_eq!(
                    number_container(&number).as_deref(),
                    case["container"].as_str(),
                    "{bits:016x}"
                );
            } else {
                assert!(case["container"].is_null());
            }
        }
    }
    #[test]
    fn full_member_conversion_preserves_raw_structured_order_and_edge_positions() {
        use super::super::{GroupParticipantMap, parse_json_response};
        use crate::binary::Node;
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../fixtures/mex/2.26.37.71-json-string-reference.json"
        ))
        .unwrap();
        let branches: Value = serde_json::from_str(include_str!(
            "../../../fixtures/mex/2.26.37.71-participant-map.json"
        ))
        .unwrap();
        // Synthetic hash preimage, independently checked by host Java; not
        // an invented server typename or a request transmitted to WhatsApp.
        let typename = serde_json::to_string(&branches["hash_reference"][1]["value"]).unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let raw = case["raw"].as_str().unwrap();
            let payload = format!(
                r#"{{"data":{{"xwa2_group_query_by_id":{{"participants":{{"edges":[null,7,{{"participant":{{"__typename":{typename},"id":"12345@s.whatsapp.net","display_name":{raw}}}}}]}}}}}},"errors":[]}}"#
            );
            for child in ["result", "update"] {
                let response = parse_json_response(
                    &Node::new("iq")
                        .children(vec![Node::new(child).bytes(payload.as_bytes().to_vec())]),
                )
                .unwrap();
                let GroupParticipantMap::Members(members) =
                    response.group_participant_map().unwrap()
                else {
                    panic!("expected converted synthetic member");
                };
                assert_eq!(members.len(), 1);
                assert_eq!(
                    members[0].display_name.as_deref(),
                    case["expected"]["container"].as_str(),
                    "{raw} / {child}"
                );
                assert_eq!(format!("{:?}", members[0]), "GroupParticipant { .. }");
                assert!(!format!("{response:?}").contains("display_name"));
            }
        }
    }
}
