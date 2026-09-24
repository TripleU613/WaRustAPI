//! Setting, changing and removing the account's two-step verification PIN.
//!
//! **Confirmed** from comparison 2.26.37.71 `LX/0lo->A06`
//! (`TwoFactorXmppMethods/sendSetTwoFactorAuth`):
//!
//! ```text
//! <iq to=<server> id=<id> xmlns="urn:xmpp:whatsapp:account" type="set">
//!   <2fa>
//!     <code>…</code>     (only when a code is supplied)
//!     <email>…</email>   (only when an address is supplied)
//!   </2fa>
//! </iq>
//! ```
//!
//! Both children are optional and each is added only when its value is
//! non-null, so the element set is the request: a code alone sets or changes
//! the PIN, an address alone sets the recovery email, and an **empty** code
//! turns two-step verification off.
//!
//! This is distinct from `lab-signup verify-pin`, which *answers* a PIN
//! challenge during registration. That one proves we know an existing PIN; this
//! one changes what the PIN is.
use crate::{Error, Result, binary::Node, model::jid::Jid, store::ProtocolStore};

/// Durable action kind for a two-step verification change.
pub const ACTION_KIND: &str = "two-factor-set";

fn invalid(reason: &'static str) -> Error {
    Error::invalid("two_factor", reason)
}

/// What to change. At least one field must be present, because an empty `<2fa>`
/// asks the server for nothing.
#[derive(Debug, Clone, Default)]
pub struct TwoFactorChange<'a> {
    /// The new PIN, or `Some("")` to turn two-step verification **off**.
    pub code: Option<&'a str>,
    /// Recovery email address, or `Some("")` to clear it.
    pub email: Option<&'a str>,
}

/// Build the request. The PIN is never an attribute and never logged: it is the
/// text content of `<code>`, exactly as the APK places it.
pub fn set_request(id: &str, change: &TwoFactorChange<'_>) -> Result<Node> {
    if id.is_empty() || id.len() > 128 {
        return Err(invalid("invalid request id"));
    }
    if change.code.is_none() && change.email.is_none() {
        return Err(invalid("nothing to change"));
    }
    if let Some(code) = change.code
        && !code.is_empty()
        && (code.len() != 6 || !code.bytes().all(|b| b.is_ascii_digit()))
    {
        // Six digits is the app's own PIN, and an empty string is the documented
        // way to switch two-step verification off. Anything else would be set as
        // a PIN the operator could not then enter in WhatsApp.
        return Err(invalid("a PIN is six digits, or empty to turn 2FA off"));
    }
    if change.email.is_some_and(|email| {
        !email.is_empty() && (email.len() > 254 || !email.contains('@') || email.contains(' '))
    }) {
        return Err(invalid("invalid recovery address"));
    }
    let mut children = Vec::new();
    if let Some(code) = change.code {
        children.push(Node::new("code").text(code));
    }
    if let Some(email) = change.email {
        children.push(Node::new("email").text(email));
    }
    Ok(Node::new("iq")
        .attr("to", Jid::parse("s.whatsapp.net")?)
        .attr("id", id)
        .attr("xmlns", "urn:xmpp:whatsapp:account")
        .attr("type", "set")
        .children(vec![Node::new("2fa").children(children)]))
}

/// Queue the change on the durable action queue. Nothing is sent until a
/// session drains it, and the PIN lives only in the encrypted request record.
pub fn queue_set(store: &ProtocolStore, change: &TwoFactorChange<'_>) -> Result<String> {
    // Validate before reserving, so a refused PIN never occupies an action.
    set_request("validation", change)?;
    let code = change.code.map(str::to_owned);
    let email = change.email.map(str::to_owned);
    super::actions::queue(store, ACTION_KIND, "two-step verification", move |id| {
        set_request(
            id,
            &TwoFactorChange {
                code: code.as_deref(),
                email: email.as_deref(),
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::NodeContent;

    fn text_of(node: &Node, name: &str) -> Option<String> {
        match &node.child(name)?.content {
            NodeContent::Text(t) => Some(t.clone()),
            _ => None,
        }
    }

    /// The element set *is* the request: a code alone changes the PIN, an
    /// address alone changes the recovery email, and each child appears only
    /// when supplied. Sending an unwanted empty `<email/>` would clear the
    /// operator's recovery address as a side effect of setting a PIN.
    #[test]
    fn each_child_appears_only_when_asked_for() {
        let node = set_request(
            "A1",
            &TwoFactorChange {
                code: Some("123456"),
                email: None,
            },
        )
        .unwrap();
        assert_eq!(node.get_attr("type").unwrap().as_text(), Some("set"));
        assert_eq!(
            node.get_attr("xmlns").unwrap().as_text(),
            Some("urn:xmpp:whatsapp:account")
        );
        let twofa = node.child("2fa").unwrap();
        assert_eq!(text_of(twofa, "code").as_deref(), Some("123456"));
        assert!(twofa.child("email").is_none(), "email must not be cleared");

        let email_only = set_request(
            "A2",
            &TwoFactorChange {
                code: None,
                email: Some("a@example.com"),
            },
        )
        .unwrap();
        let twofa = email_only.child("2fa").unwrap();
        assert!(twofa.child("code").is_none(), "the PIN must not be cleared");
        assert_eq!(text_of(twofa, "email").as_deref(), Some("a@example.com"));

        // An empty code is how two-step verification is turned off.
        let off = set_request(
            "A3",
            &TwoFactorChange {
                code: Some(""),
                email: None,
            },
        )
        .unwrap();
        assert_eq!(
            text_of(off.child("2fa").unwrap(), "code").as_deref(),
            Some("")
        );
    }

    /// A PIN the operator could not then type into WhatsApp is worse than a
    /// refusal, so anything that is not six digits (or the empty off switch) is
    /// rejected before it can be set.
    #[test]
    fn a_pin_that_could_not_be_entered_is_refused() {
        for bad in ["12345", "1234567", "12345a", "abcdef"] {
            assert!(
                set_request(
                    "A",
                    &TwoFactorChange {
                        code: Some(bad),
                        email: None
                    }
                )
                .is_err(),
                "{bad} must be refused"
            );
        }
        // Nothing to change is not a request.
        assert!(set_request("A", &TwoFactorChange::default()).is_err());
        // A malformed recovery address is refused rather than stored.
        assert!(
            set_request(
                "A",
                &TwoFactorChange {
                    code: None,
                    email: Some("not an address")
                }
            )
            .is_err()
        );
    }

    /// The request must survive the wire codec it will actually go out on.
    #[test]
    fn the_request_round_trips_through_the_comparison_codec() {
        let node = set_request(
            "A1",
            &TwoFactorChange {
                code: Some("654321"),
                email: Some("a@example.com"),
            },
        )
        .unwrap();
        let dict = crate::binary::TokenDictionary::comparison_2_26_37_70();
        let encoded = crate::binary::Encoder::new(dict).encode(&node).unwrap();
        let decoded =
            crate::binary::Decoder::new(&encoded[1..], dict, &crate::profile::Limits::default())
                .decode_node()
                .unwrap();
        // Node equality is too strict here: a string the token dictionary does
        // not cover comes back as `Bytes` rather than `Text`, which is the
        // decoder's representation and not a difference on the wire. What must
        // survive is the attributes and each child's exact content.
        assert_eq!(decoded.attributes, node.attributes);
        let content = |node: &Node, name: &str| match &node.child("2fa")?.child(name)?.content {
            NodeContent::Text(t) => Some(t.as_bytes().to_vec()),
            NodeContent::Bytes(b) => Some(b.to_vec()),
            _ => None,
        };
        assert_eq!(content(&decoded, "code"), Some(b"654321".to_vec()));
        assert_eq!(
            content(&decoded, "email"),
            Some(b"a@example.com".to_vec()),
            "the recovery address must survive the wire exactly"
        );
    }
}
