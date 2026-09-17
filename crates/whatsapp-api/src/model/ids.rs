//! Identifier newtypes (README section 9).
//!
//! None of these convert implicitly into another. Where the protocol requires
//! a composite key, a dedicated struct (for example
//! [`crate::model::message::MessageKey`]) is used rather than string
//! concatenation.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::jid::Jid;
use crate::error::{Error, Result};

/// Maximum accepted length for caller-supplied identifier strings.
pub const MAX_ID_LEN: usize = 128;

fn validate_id(field: &'static str, s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(Error::invalid(field, "must not be empty"));
    }
    if s.len() > MAX_ID_LEN {
        return Err(Error::invalid(field, format!("longer than {MAX_ID_LEN} bytes")));
    }
    if !s
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'))
    {
        return Err(Error::invalid(
            field,
            "only ASCII alphanumerics and `-_.:` are allowed",
        ));
    }
    Ok(())
}

macro_rules! string_id {
    ($(#[$meta:meta])* $name:ident, $field:literal) => {
        $(#[$meta])*
        #[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Validate and wrap a string identifier.
            pub fn new(s: impl Into<String>) -> Result<Self> {
                let s = s.into();
                validate_id($field, &s)?;
                Ok(Self(s))
            }

            /// Borrow the identifier text.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

string_id!(
    /// Local identifier of an account database (not a phone number).
    AccountId,
    "account_id"
);
string_id!(
    /// Caller-supplied idempotency key for retry-safe enqueue.
    ClientRequestId,
    "client_request_id"
);
string_id!(
    /// WhatsApp wire message identifier as observed on the protocol.
    ///
    /// The format is profile-specific; this type only bounds and sanitizes it.
    MessageId,
    "message_id"
);

/// Monotonic generation of an account's local state. Bumped on re-registration
/// or restore so that stale records cannot be confused with current ones.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
#[serde(transparent)]
pub struct AccountGeneration(pub u32);

impl AccountGeneration {
    /// The next generation.
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

/// Local, library-generated identifier of a queued message (16 random bytes).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LocalMessageId(pub [u8; 16]);

impl LocalMessageId {
    /// Generate a fresh random identifier from the OS CSPRNG.
    pub fn generate() -> Result<Self> {
        let mut b = [0u8; 16];
        getrandom::fill(&mut b).map_err(|e| {
            Error::Io(std::io::Error::other(format!("entropy source failed: {e}")))
        })?;
        Ok(Self(b))
    }
}

impl fmt::Debug for LocalMessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LocalMessageId({})", hex::encode(self.0))
    }
}

impl fmt::Display for LocalMessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}

/// Stable identifier of a durable domain event (16 random bytes).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EventId(pub [u8; 16]);

impl EventId {
    /// Generate a fresh random identifier from the OS CSPRNG.
    pub fn generate() -> Result<Self> {
        let mut b = [0u8; 16];
        getrandom::fill(&mut b).map_err(|e| {
            Error::Io(std::io::Error::other(format!("entropy source failed: {e}")))
        })?;
        Ok(Self(b))
    }
}

impl fmt::Debug for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EventId({})", hex::encode(self.0))
    }
}

/// Monotonic per-account position in the event journal.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
#[serde(transparent)]
pub struct EventCursor(pub u64);

impl EventCursor {
    /// The position before the first event.
    pub const START: Self = Self(0);

    /// The next cursor.
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

/// Index of a device within an account (0 is conventionally the primary).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
#[serde(transparent)]
pub struct DeviceIndex(pub u16);

/// A contact identity: a user-level JID without a device component.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ContactId(Jid);

impl ContactId {
    /// Build from a JID, stripping any device component.
    pub fn from_jid(jid: &Jid) -> Result<Self> {
        if !jid.is_user_like() {
            return Err(Error::invalid("contact_id", "JID is not a user identity"));
        }
        Ok(Self(jid.to_non_ad()))
    }

    /// The underlying user-level JID.
    pub fn jid(&self) -> &Jid {
        &self.0
    }
}

/// A group identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GroupId(Jid);

impl GroupId {
    /// Build from a group JID.
    pub fn from_jid(jid: &Jid) -> Result<Self> {
        if !jid.is_group() {
            return Err(Error::invalid("group_id", "JID is not a group"));
        }
        Ok(Self(jid.clone()))
    }

    /// The underlying group JID.
    pub fn jid(&self) -> &Jid {
        &self.0
    }
}

/// A chat: a direct conversation with a contact or a group.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ChatId {
    /// Direct conversation.
    Direct(ContactId),
    /// Group conversation.
    Group(GroupId),
}

impl ChatId {
    /// Classify a JID into a chat identity.
    pub fn from_jid(jid: &Jid) -> Result<Self> {
        if jid.is_group() {
            Ok(Self::Group(GroupId::from_jid(jid)?))
        } else {
            Ok(Self::Direct(ContactId::from_jid(jid)?))
        }
    }

    /// The JID that addresses this chat on the wire.
    pub fn jid(&self) -> &Jid {
        match self {
            Self::Direct(c) => c.jid(),
            Self::Group(g) => g.jid(),
        }
    }
}

/// Full device address: the unit that owns a cryptographic session.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DeviceAddress {
    /// The contact (user-level identity).
    pub contact: ContactId,
    /// The device index within that identity.
    pub device: DeviceIndex,
}

impl DeviceAddress {
    /// Build from a JID that may carry a device component.
    pub fn from_jid(jid: &Jid) -> Result<Self> {
        Ok(Self {
            contact: ContactId::from_jid(jid)?,
            device: DeviceIndex(jid.device),
        })
    }

    /// The addressed JID including the device component.
    pub fn to_jid(&self) -> Jid {
        self.contact.jid().with_device(self.device.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_validate() {
        assert!(ClientRequestId::new("").is_err());
        assert!(ClientRequestId::new("a b").is_err());
        assert!(ClientRequestId::new("req-1_2.3:4").is_ok());
        assert!(MessageId::new("x".repeat(MAX_ID_LEN + 1)).is_err());
    }

    #[test]
    fn local_ids_are_unique() {
        let a = LocalMessageId::generate().unwrap();
        let b = LocalMessageId::generate().unwrap();
        assert_ne!(a, b);
        assert_eq!(a.to_string().len(), 32);
    }
}
