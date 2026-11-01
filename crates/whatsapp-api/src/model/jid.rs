//! WhatsApp addresses (JIDs).
//!
//! The wire representation is `user[.agent][:device]@server`. Distinct
//! representations (phone-number JIDs, logical IDs, device suffixes, groups)
//! are preserved; nothing here merges identities (docs/design/README.md section 9).
//!
//! Server names are `ReferenceOnly` for the Android primary profile: they are
//! taken from public multi-device implementations and recorded in
//! `docs/protocol/evidence-ledger.md` under `jid.servers`.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Maximum accepted length of a JID's user part.
pub const MAX_USER_LEN: usize = 64;
/// Maximum accepted length of a JID's server part.
pub const MAX_SERVER_LEN: usize = 64;

/// The server (domain) component of a JID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Server {
    /// Phone-number user identities (`s.whatsapp.net`).
    User,
    /// Groups (`g.us`).
    Group,
    /// Logical (non-phone) identities (`lid`).
    Lid,
    /// Broadcast lists and status (`broadcast`).
    Broadcast,
    /// Newsletters / channels (`newsletter`).
    Newsletter,
    /// Legacy user server (`c.us`).
    LegacyUser,
    /// Hosted business identities (`hosted`).
    Hosted,
    /// Hosted logical identities (`hosted.lid`).
    HostedLid,
    /// Messenger interop identities (`msgr`).
    Messenger,
    /// Third-party interop identities (`interop`).
    Interop,
    /// Any other server, preserved verbatim.
    Other(String),
}

impl Server {
    /// Wire text of the server.
    pub fn as_str(&self) -> &str {
        match self {
            Self::User => "s.whatsapp.net",
            Self::Group => "g.us",
            Self::Lid => "lid",
            Self::Broadcast => "broadcast",
            Self::Newsletter => "newsletter",
            Self::LegacyUser => "c.us",
            Self::Hosted => "hosted",
            Self::HostedLid => "hosted.lid",
            Self::Messenger => "msgr",
            Self::Interop => "interop",
            Self::Other(s) => s,
        }
    }

    /// Parse wire text into a server variant.
    pub fn parse(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Err(Error::invalid("jid.server", "empty server"));
        }
        if s.len() > MAX_SERVER_LEN {
            return Err(Error::invalid("jid.server", "server too long"));
        }
        if !s
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-'))
        {
            return Err(Error::invalid("jid.server", "invalid characters"));
        }
        Ok(match s {
            "s.whatsapp.net" => Self::User,
            "g.us" => Self::Group,
            "lid" => Self::Lid,
            "broadcast" => Self::Broadcast,
            "newsletter" => Self::Newsletter,
            "c.us" => Self::LegacyUser,
            "hosted" => Self::Hosted,
            "hosted.lid" => Self::HostedLid,
            "msgr" => Self::Messenger,
            "interop" => Self::Interop,
            other => Self::Other(other.to_owned()),
        })
    }
}

/// Which wire encoding a JID uses. Preserved so re-encoding does not silently
/// change the representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JidKind {
    /// `user@server` without agent/device (`JID_PAIR` in reference codecs).
    Plain,
    /// Agent/device-qualified (`AD_JID` in reference codecs).
    AgentDevice,
    /// Interop JID (`INTEROP_JID` in reference codecs).
    Interop,
    /// Facebook JID (`FB_JID` in reference codecs).
    Facebook,
}

/// A validated JID.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Jid {
    /// User part (may be empty for server-only JIDs).
    pub user: String,
    /// Agent component (0 when absent).
    pub agent: u8,
    /// Device component (0 when absent).
    pub device: u16,
    /// Server.
    pub server: Server,
    /// Wire encoding kind.
    pub kind: JidKind,
}

impl Jid {
    /// Build a plain user@server JID.
    pub fn new(user: impl Into<String>, server: Server) -> Result<Self> {
        let user = user.into();
        Self::validate_user(&user)?;
        Ok(Self {
            user,
            agent: 0,
            device: 0,
            server,
            kind: JidKind::Plain,
        })
    }

    /// Build an agent/device JID.
    pub fn new_ad(user: impl Into<String>, agent: u8, device: u16, server: Server) -> Result<Self> {
        let user = user.into();
        Self::validate_user(&user)?;
        Ok(Self {
            user,
            agent,
            device,
            server,
            kind: JidKind::AgentDevice,
        })
    }

    fn validate_user(user: &str) -> Result<()> {
        if user.len() > MAX_USER_LEN {
            return Err(Error::invalid("jid.user", "user too long"));
        }
        if user
            .bytes()
            .any(|b| matches!(b, b'@' | b':' | b'/') || b.is_ascii_control())
        {
            return Err(Error::invalid("jid.user", "invalid characters"));
        }
        Ok(())
    }

    /// Whether this is a user-like identity (phone-number, LID, hosted, or legacy).
    pub fn is_user_like(&self) -> bool {
        matches!(
            self.server,
            Server::User | Server::Lid | Server::LegacyUser | Server::Hosted | Server::HostedLid
        ) && !self.user.is_empty()
    }

    /// Whether this is a group.
    pub fn is_group(&self) -> bool {
        matches!(self.server, Server::Group)
    }

    /// Whether this carries an agent or device component.
    pub fn is_ad(&self) -> bool {
        self.agent != 0 || self.device != 0 || self.kind == JidKind::AgentDevice
    }

    /// Copy with the device component replaced (agent preserved).
    pub fn with_device(&self, device: u16) -> Self {
        let mut j = self.clone();
        j.device = device;
        j.kind = if device == 0 && j.agent == 0 {
            JidKind::Plain
        } else {
            JidKind::AgentDevice
        };
        j
    }

    /// Copy without agent/device components.
    pub fn to_non_ad(&self) -> Self {
        Self {
            user: self.user.clone(),
            agent: 0,
            device: 0,
            server: self.server.clone(),
            kind: JidKind::Plain,
        }
    }

    /// Require a well-formed group JID: `g.us`, no agent/device, and a user
    /// of digits or the legacy `creator-timestamp` digits form.
    ///
    /// The generic parser admits any user text, so a masked display ID such as
    /// `***…023@g.us` (what `group list` prints) parsed and was sent; the
    /// server silently ignores a query to a group that cannot exist
    /// (observed 2026-11-01: group info got no reply for 30–45 s).
    pub fn require_group_id(&self) -> Result<()> {
        let digits = |part: &str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
        let well_formed = match self.user.split_once('-') {
            Some((creator, created)) => digits(creator) && digits(created),
            None => digits(&self.user),
        };
        if self.server != Server::Group || self.agent != 0 || self.device != 0 || !well_formed {
            return Err(Error::invalid(
                "jid",
                "not a group id: expected digits@g.us (or creator-timestamp@g.us); a masked id copied from display output is refused",
            ));
        }
        Ok(())
    }

    /// Parse the textual form `user[.agent][:device]@server`.
    pub fn parse(s: &str) -> Result<Self> {
        if s.len() > MAX_USER_LEN + MAX_SERVER_LEN + 16 {
            return Err(Error::invalid("jid", "too long"));
        }
        let Some((local, server)) = s.rsplit_once('@') else {
            // Server-only JID.
            return Ok(Self {
                user: String::new(),
                agent: 0,
                device: 0,
                server: Server::parse(s)?,
                kind: JidKind::Plain,
            });
        };
        let server = Server::parse(server)?;
        let (local, device) = match local.split_once(':') {
            Some((u, d)) => (
                u,
                d.parse::<u16>()
                    .map_err(|_| Error::invalid("jid.device", "not a u16"))?,
            ),
            None => (local, 0),
        };
        let (user, agent) = match local.split_once('.') {
            Some((u, a)) if a.bytes().all(|b| b.is_ascii_digit()) && !a.is_empty() => (
                u,
                a.parse::<u8>()
                    .map_err(|_| Error::invalid("jid.agent", "not a u8"))?,
            ),
            _ => (local, 0),
        };
        Self::validate_user(user)?;
        let kind = if agent == 0 && device == 0 {
            JidKind::Plain
        } else {
            JidKind::AgentDevice
        };
        Ok(Self {
            user: user.to_owned(),
            agent,
            device,
            server,
            kind,
        })
    }
}

impl FromStr for Jid {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl fmt::Display for Jid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.user.is_empty() {
            return f.write_str(self.server.as_str());
        }
        f.write_str(&self.user)?;
        if self.agent != 0 {
            write!(f, ".{}", self.agent)?;
        }
        if self.device != 0 {
            write!(f, ":{}", self.device)?;
        }
        write!(f, "@{}", self.server.as_str())
    }
}

impl fmt::Debug for Jid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Redact the user part: it is frequently a phone number.
        if self.user.is_empty() {
            write!(f, "Jid({})", self.server.as_str())
        } else {
            write!(
                f,
                "Jid(<{} chars>{}{}@{})",
                self.user.len(),
                if self.agent != 0 { ".a" } else { "" },
                if self.device != 0 { ":d" } else { "" },
                self.server.as_str()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_ids_must_be_digits_and_masked_display_ids_are_refused() {
        for ok in ["120363000000000001@g.us", "15551234567-1600000000@g.us"] {
            Jid::parse(ok).unwrap().require_group_id().unwrap();
        }
        for bad in [
            "***************023@g.us",
            "12036300000000000a@g.us",
            "1555-@g.us",
            "-1600000000@g.us",
            "g.us",
            "15551234567@s.whatsapp.net",
        ] {
            let error = Jid::parse(bad).unwrap().require_group_id().unwrap_err();
            assert!(error.to_string().contains("masked id"), "{bad}: {error}");
        }
    }

    #[test]
    fn roundtrip_plain_and_ad() {
        let j = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        assert_eq!(j.kind, JidKind::Plain);
        assert_eq!(j.to_string(), "15551234567@s.whatsapp.net");
        let ad = Jid::parse("15551234567.3:7@s.whatsapp.net").unwrap();
        assert_eq!((ad.agent, ad.device), (3, 7));
        assert_eq!(ad.kind, JidKind::AgentDevice);
        assert_eq!(ad.to_string(), "15551234567.3:7@s.whatsapp.net");
        assert_eq!(ad.to_non_ad(), j);
    }

    #[test]
    fn server_only_and_groups() {
        let s = Jid::parse("s.whatsapp.net").unwrap();
        assert!(s.user.is_empty());
        assert_eq!(s.to_string(), "s.whatsapp.net");
        let g = Jid::parse("123456789-1600000000@g.us").unwrap();
        assert!(g.is_group());
        assert!(!g.is_user_like());
    }

    #[test]
    fn rejects_garbage() {
        assert!(Jid::parse("a@b@c@").is_err());
        assert!(Jid::parse("user:70000@s.whatsapp.net").is_err());
        assert!(Jid::parse("user@bad server").is_err());
    }

    #[test]
    fn debug_redacts_user() {
        let j = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let d = format!("{j:?}");
        assert!(!d.contains("15551234567"));
    }
}
