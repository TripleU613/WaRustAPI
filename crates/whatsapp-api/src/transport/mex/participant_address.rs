//! Strict UserJid prerequisite for APK 37.71 ADv.A0D, not send eligibility.
//! Evidence: docs/protocol/mex-participant-address-evidence.txt and
//! mex-participant-special-address-evidence.txt. Do not use is_user_like here.

use crate::{Error, Result};

/// Plain APK UserJid subtype accepted by the member-primary type check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantAddressKind {
    /// Ordinary PhoneUserJid, including plain hosted input.
    Phone,
    /// Plain LID, including plain hosted.lid input.
    Lid,
    /// Plain bot; device bots fail the UserJid check.
    Bot,
    /// Plain interop account with Java-parsed integrator.
    Interop,
    /// APK 8l6: the literal zero user singleton.
    ServerZero,
    /// APK Blj: the case-sensitive Server user singleton.
    ServerName,
    /// APK 1xv: bare lid_me placeholder, not a numbered LID.
    LidMe,
    /// APK 0DR: bare status_me placeholder.
    StatusMe,
}

/// Private participant address with APK canonical domain, separate from send policy.
/// Debug hides the address; explicit access can reveal account identifiers.
#[derive(Clone, PartialEq, Eq)]
pub struct ParticipantAddress {
    kind: ParticipantAddressKind,
    user: String,
    server: &'static str,
}

impl std::fmt::Debug for ParticipantAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParticipantAddress")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

fn invalid() -> Error {
    Error::InvalidInput {
        field: "mex.participant.address",
        reason: "expected a valid plain APK UserJid; device and non-user addresses are rejected"
            .into(),
    }
}

fn ascii_number(user: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&user.len())
        && !user.starts_with('0')
        && user.bytes().all(|b| b.is_ascii_digit())
}

impl ParticipantAddress {
    /// Parse the strict member-primary UserJid accepted by 0D8.A01.
    /// The input is never included in failures. This does not authorize sending.
    /// Platform digit behavior uses the independent host-Java BMP reference;
    /// complete Android Unicode-version parity remains unverified.
    pub fn parse(value: &str) -> Result<Self> {
        // 0D1.A00 accepts these bare UserJid singletons, not @lid_me domains.
        let (user, server, kind) = match value {
            "lid_me" => ("", "lid_me", ParticipantAddressKind::LidMe),
            "status_me" => ("", "status_me", ParticipantAddressKind::StatusMe),
            _ => {
                let (user, domain) = value.rsplit_once('@').ok_or_else(invalid)?;
                // 0D1.A00 routes dot/colon local parts to DeviceJid subclasses.
                // Literal class definitions prove these are outside UserJid,
                // including an explicitly zero suffix. Never collapse it to plain.
                if user.is_empty() || user.contains(['.', ':']) {
                    return Err(invalid());
                }
                match domain {
                    "s.whatsapp.net" | "hosted" => {
                        let kind = match user {
                            "0" => ParticipantAddressKind::ServerZero,
                            "Server" => ParticipantAddressKind::ServerName,
                            _ if ascii_number(user, 5, 20)
                                && (!user.starts_with("10") || user.starts_with("1008")) =>
                            {
                                ParticipantAddressKind::Phone
                            }
                            _ => return Err(invalid()),
                        };
                        // 0D2.A02 factory; all three getServer implementations.
                        (user, "s.whatsapp.net", kind)
                    }
                    "lid" | "hosted.lid" if ascii_number(user, 1, 15) => {
                        (user, "lid", ParticipantAddressKind::Lid)
                    }
                    "bot" if ascii_number(user, 1, 20) => {
                        (user, "bot", ParticipantAddressKind::Bot)
                    }
                    "interop" => {
                        // 9Lm uses lastIndexOf('-'), signed Integer.parseInt,
                        // not an unsigned ASCII-only integrator or first split.
                        let (integrator, account) = user.rsplit_once('-').ok_or_else(invalid)?;
                        if !(1..=3).contains(&integrator.encode_utf16().count())
                            || integrator.starts_with('0')
                            || super::java_decimal_long(integrator)
                                .and_then(|v| i32::try_from(v).ok())
                                .is_none()
                            || !ascii_number(account, 1, 15)
                        {
                            return Err(invalid());
                        }
                        (user, "interop", ParticipantAddressKind::Interop)
                    }
                    _ => return Err(invalid()),
                }
            }
        };
        Ok(Self {
            kind,
            user: user.into(),
            server,
        })
    }

    /// Recognized subtype only, without account identifier.
    pub const fn kind(&self) -> ParticipantAddressKind {
        self.kind
    }

    /// Explicit private user identifier. Placeholders carry an empty user.
    pub fn user(&self) -> &str {
        &self.user
    }

    /// APK plain-address canonical domain; hosted inputs normalize here.
    pub const fn server(&self) -> &'static str {
        self.server
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apk_constructor_and_type_boundaries() {
        // Literal constructor predicates, factory domains and instanceof
        // evidence, not claimed live server member records.
        for (value, kind, server) in [
            (
                "12345@s.whatsapp.net",
                ParticipantAddressKind::Phone,
                "s.whatsapp.net",
            ),
            (
                "10081@hosted",
                ParticipantAddressKind::Phone,
                "s.whatsapp.net",
            ),
            ("1@hosted.lid", ParticipantAddressKind::Lid, "lid"),
            ("1@bot", ParticipantAddressKind::Bot, "bot"),
            ("+1-1@interop", ParticipantAddressKind::Interop, "interop"),
            ("-1-1@interop", ParticipantAddressKind::Interop, "interop"),
            ("١-1@interop", ParticipantAddressKind::Interop, "interop"),
            (
                "0@hosted",
                ParticipantAddressKind::ServerZero,
                "s.whatsapp.net",
            ),
            (
                "Server@s.whatsapp.net",
                ParticipantAddressKind::ServerName,
                "s.whatsapp.net",
            ),
            ("lid_me", ParticipantAddressKind::LidMe, "lid_me"),
            ("status_me", ParticipantAddressKind::StatusMe, "status_me"),
        ] {
            let parsed = ParticipantAddress::parse(value).unwrap();
            assert_eq!((parsed.kind(), parsed.server()), (kind, server));
            assert!(!format!("{parsed:?}").contains(value));
        }
        for value in [
            "",
            "s.whatsapp.net",
            "gdpr@s.whatsapp.net",
            "12345@c.us",
            "12345@msgr",
            "1@g.us",
            "x@lid_me",
            "x@status_me",
            "1234@s.whatsapp.net",
            "10123@s.whatsapp.net",
            "01234@s.whatsapp.net",
            "١٢٣٤٥@s.whatsapp.net",
            "0@lid",
            "0@bot",
            "1-0@interop",
            "01-1@interop",
            "1234-1@interop",
            "1-١@interop",
            "1-@interop",
            "12345:0@s.whatsapp.net",
            "12345.0@s.whatsapp.net",
            "1:99@hosted.lid",
            "1:0@bot",
            "1-1:0@interop",
            "12345@s.whatsapp.net ",
            " 12345@s.whatsapp.net",
            "@s.whatsapp.net",
        ] {
            let error = ParticipantAddress::parse(value).unwrap_err().to_string();
            if !value.is_empty() {
                assert!(!error.contains(value), "{value:?}");
            }
        }
        for (domain, maximum) in [("lid", 15), ("bot", 20), ("s.whatsapp.net", 20)] {
            assert!(
                ParticipantAddress::parse(&format!("{}@{domain}", "9".repeat(maximum))).is_ok()
            );
            assert!(
                ParticipantAddress::parse(&format!("{}@{domain}", "9".repeat(maximum + 1)))
                    .is_err()
            );
        }
    }
    #[test]
    fn interop_integrators_match_independent_java_reference() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/mex/2.26.37.71-integrator-reference.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let integrator = case["integrator"].as_str().unwrap();
            let parsed =
                super::super::java_decimal_long(integrator).and_then(|v| i32::try_from(v).ok());
            assert_eq!(
                parsed.map(i64::from),
                case["java_integer"].as_i64(),
                "{integrator:?}"
            );
            let result = ParticipantAddress::parse(&format!("{integrator}-1@interop"));
            assert_eq!(
                result.is_ok(),
                case["accepted"].as_bool().unwrap(),
                "{integrator:?}"
            );
        }
    }
}
