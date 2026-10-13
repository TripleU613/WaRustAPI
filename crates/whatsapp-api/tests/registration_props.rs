//! Property tests for the registration response parser: server bodies are
//! untrusted network input, so parsing must never panic, must stay bounded,
//! and its `Debug` form must never echo a secret value (registration safety
//! audit, 2026-10-13).

use proptest::prelude::*;
use serde_json::{Map, Value};
use whatsapp_api::registration::wire::RegistrationHttpResponse;
use zeroize::Zeroizing;

/// Every value that is a secret in a registration answer.
const SECRET_KEYS: [&str; 7] = [
    "login",
    "code",
    "wipe_token",
    "passkey_credential",
    "appeal_token",
    "edge_routing_info",
    "email",
];

/// Non-secret keys the parser reads, with values of any JSON type.
const OTHER_KEYS: [&str; 22] = [
    "status",
    "reason",
    "param",
    "retry_after",
    "sms_wait",
    "voice_wait",
    "flash_wait",
    "wipe_wait",
    "email_otp_wait",
    "length",
    "sms_length",
    "voice_length",
    "email_otp_length",
    "second_factor_methods",
    "is_device_trusted",
    "email_otp_eligible",
    "wa_old_eligible",
    "recommended_method",
    "fallback_methods",
    "verify_pn_device",
    "security_code_set",
    "custom_block_screen",
];

fn any_scalar() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::from),
        any::<u64>().prop_map(Value::from),
        any::<f64>()
            .prop_map(|f| serde_json::Number::from_f64(f).map_or(Value::Null, Value::Number)),
        "[a-z_0-9]{0,20}".prop_map(Value::from),
        ".{0,40}".prop_map(Value::from),
    ]
}

fn any_value() -> impl Strategy<Value = Value> {
    any_scalar().prop_recursive(3, 24, 6, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..6).prop_map(Value::Array),
            prop::collection::btree_map("[a-z_]{1,12}", inner, 0..6)
                .prop_map(|m| Value::Object(m.into_iter().collect())),
        ]
    })
}

/// A registration-shaped answer: known keys with values of any type, a
/// plausible status most of the time, and canary secrets.
fn any_answer() -> impl Strategy<Value = Value> {
    (
        prop::collection::btree_map(
            prop::sample::select(OTHER_KEYS.to_vec()),
            any_value(),
            0..12,
        ),
        prop::option::of(prop::sample::select(vec!["ok", "sent", "fail", "attached"])),
        prop::collection::vec(any::<bool>(), SECRET_KEYS.len()),
        prop::collection::btree_map("[a-z_]{1,12}", any_scalar(), 0..4),
    )
        .prop_map(|(known, status, secrets, unknown)| {
            let mut object: Map<String, Value> = unknown.into_iter().collect();
            for (key, value) in known {
                object.insert(key.to_owned(), value);
            }
            if let Some(status) = status {
                object.insert("status".into(), Value::from(status));
            }
            plant_secrets(&mut object, &secrets);
            Value::Object(object)
        })
}

fn plant_secrets(object: &mut Map<String, Value>, present: &[bool]) {
    for (key, present) in SECRET_KEYS.iter().zip(present) {
        if *present {
            object.insert((*key).into(), Value::from(format!("CANARY-{key}")));
        }
    }
}

/// A well-typed answer the parser accepts: valid status and reason tokens,
/// waits as numbers or numeric strings (up to `u64::MAX`), and canary secrets.
fn well_typed_answer() -> impl Strategy<Value = Value> {
    let wait = prop_oneof![
        any::<u64>().prop_map(Value::from),
        any::<u64>().prop_map(|n| Value::from(n.to_string())),
    ];
    (
        prop::sample::select(vec!["ok", "sent", "fail", "attached"]),
        prop::option::of("[a-z_]{1,24}"),
        prop::collection::btree_map(
            prop::sample::select(vec![
                "retry_after",
                "sms_wait",
                "voice_wait",
                "flash_wait",
                "wipe_wait",
                "email_otp_wait",
            ]),
            wait,
            0..6,
        ),
        prop::collection::vec(any::<bool>(), SECRET_KEYS.len()),
    )
        .prop_map(|(status, reason, waits, secrets)| {
            let mut object = Map::new();
            object.insert("status".into(), Value::from(status));
            if let Some(reason) = reason {
                object.insert("reason".into(), Value::from(reason));
            }
            for (key, value) in waits {
                object.insert(key.into(), value);
            }
            plant_secrets(&mut object, &secrets);
            Value::Object(object)
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// Arbitrary bytes under any HTTP status: an error or a value, never a panic.
    #[test]
    fn arbitrary_bodies_never_panic(
        status in 0u16..1000,
        body in prop::collection::vec(any::<u8>(), 0..2048),
    ) {
        if let Ok(response) = RegistrationHttpResponse::new(status, Zeroizing::new(body)) {
            let _ = response.parse_json();
        }
    }

    /// Registration-shaped JSON with any value types: parsing never panics,
    /// and neither the parsed answer's nor the error's text echoes a secret.
    #[test]
    fn shaped_answers_never_panic_or_leak(answer in any_answer()) {
        let body = serde_json::to_vec(&answer).unwrap();
        let response = RegistrationHttpResponse::new(200, Zeroizing::new(body)).unwrap();
        let text = match response.parse_json() {
            Ok(parsed) => format!("{parsed:?}"),
            Err(error) => format!("{error} {error:?}"),
        };
        prop_assert!(!text.contains("CANARY"), "{}", text);
    }

    /// The same for answers the parser must accept, so `Debug` of a parsed
    /// answer is exercised on every case, including the largest waits.
    #[test]
    fn well_typed_answers_parse_without_leaking(answer in well_typed_answer()) {
        let body = serde_json::to_vec(&answer).unwrap();
        let response = RegistrationHttpResponse::new(200, Zeroizing::new(body)).unwrap();
        let parsed = response.parse_json();
        prop_assert!(parsed.is_ok(), "{:?}", parsed.err());
        let text = format!("{:?}", parsed.unwrap());
        prop_assert!(!text.contains("CANARY"), "{}", text);
    }
}
