//! Setting our profile photo and looking up someone's.
//!
//! **Confirmed static, 2.26.37.71.** `SetProfilePhotoProtocolHelper`
//! (`IXF.A00(byte[], …)`) builds
//!
//! ```text
//! <iq id xmlns="w:profile:picture" type="set" [target=<group>]>
//!   <picture type="image" [reupload="true"]>JPEG bytes</picture>
//! </iq>
//! ```
//!
//! `target` is added only when its first flag is false (not our own photo);
//! `reupload` and the account-center `encryption_metadata` branch are not used
//! here. Current 2.26.43.72 Hyd.A00 calls B22.A0F, which adds xmlns
//! and returns the `to=s.whatsapp.net` attribute for the caller to append.
//! The existing address is therefore confirmed. `8kn.A00` builds the lookup
//! `<iq id xmlns="w:profile:picture" type="get" target=<jid>>
//! <picture type="image" query="url"/></iq>` (the optional picture attribute
//! `common_gid` and nested `tctoken` child are not sent). The reply carries the URL; nothing is
//! downloaded here.
//!
//! Current Hyd.A00 set and 6p1.A00 lookup both pass callback duration0;
//! neither inherits a generic20/32s expiry. Upload success reads the first
//! picture child and ignores later extensions; lookup has a separate parser.
//! Evidence: docs/protocol/apk/2.26.43.72-profile-photo-callbacks/README.md.
//! The app's crop/resize/JPEG settings remain untraced.
use crate::{Error, Result, binary::Node, model::jid::Jid, store::ProtocolStore};

/// Durable action kinds.
pub const SET_KIND: &str = "profile-picture-set";
/// Durable action kind for a lookup.
pub const GET_KIND: &str = "profile-picture-get";
/// Our bound, not WhatsApp's (not traced).
pub const MAX_BYTES: usize = 2 * 1024 * 1024;

fn invalid(reason: &'static str) -> Error {
    Error::invalid("profile_picture", reason)
}

/// Native photo error callback outcome; separate from outer IQ Error status.
/// This diagnostic does not apply photo/cache state or authorize another request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PhotoErrorOutcome {
    /// Native signed32 code; zero includes missing/invalid-code fallback.
    pub code: i32,
    /// Lookup404/410 calls the empty-metadata callback, not its refusal callback.
    pub empty_lookup: bool,
}

pub(super) fn is_lookup_request(request: &Node) -> bool {
    request.get_attr("xmlns").and_then(|v| v.as_text()) == Some("w:profile:picture")
        && request.get_attr("type").and_then(|v| v.as_text()) == Some("get")
        && request.get_attr("target").is_some()
        && matches!(request.child_nodes(), [picture]
            if picture.description == "picture"
                && picture.get_attr("type").and_then(|v| v.as_text()) == Some("image")
                && picture.get_attr("query").and_then(|v| v.as_text()) == Some("url"))
}

pub(super) fn error_outcome(kind: &str, request: &Node, reply: &Node) -> Option<PhotoErrorOutcome> {
    if reply.get_attr("type").and_then(|v| v.as_text()) != Some("error")
        || request.get_attr("xmlns").and_then(|v| v.as_text()) != Some("w:profile:picture")
    {
        return None;
    }
    let upload = kind == SET_KIND
        && request.get_attr("type").and_then(|v| v.as_text()) == Some("set")
        && matches!(request.child_nodes(), [picture]
            if picture.description == "picture"
                && picture.get_attr("type").and_then(|v| v.as_text()) == Some("image")
                && matches!(&picture.content, crate::binary::NodeContent::Bytes(_)));
    let lookup = kind == GET_KIND && is_lookup_request(request);
    if !upload && !lookup {
        return None;
    }
    // APK43.72 Hyd.Bvj -> 1ud.A00 and 6p1.Bvj -> 6oj.A05 -> 1ud.A00.
    // The first direct coded error wins, even when its code parses to fallback0.
    // 6p1 maps404/410 to CAe(empty H5d), all other codes to CAd.
    // Evidence: docs/protocol/apk/2.26.43.72-photo-error-callbacks/README.md.
    let code = reply
        .child_nodes()
        .iter()
        .find(|child| {
            child.description == "error"
                && child.get_attr("code").and_then(|v| v.as_text()).is_some()
        })
        .and_then(|child| child.get_attr("code"))
        .and_then(|v| v.as_text())
        .and_then(super::mex::java_decimal_long)
        .and_then(|code| i32::try_from(code).ok())
        .unwrap_or(0);
    Some(PhotoErrorOutcome {
        code,
        empty_lookup: lookup && matches!(code, 404 | 410),
    })
}

/// Build the set request for our own photo.
pub fn set_request(id: &str, jpeg: &[u8]) -> Result<Node> {
    if id.is_empty() || id.len() > 128 {
        return Err(invalid("invalid request id"));
    }
    if jpeg.len() > MAX_BYTES {
        return Err(invalid("a profile photo is at most 2 MiB here"));
    }
    if !jpeg.starts_with(&[0xff, 0xd8, 0xff]) || !jpeg.ends_with(&[0xff, 0xd9]) {
        return Err(invalid("a profile photo must be a JPEG file"));
    }
    Ok(Node::new("iq")
        .attr("id", id)
        .attr("xmlns", "w:profile:picture")
        .attr("type", "set")
        .attr("to", Jid::parse("s.whatsapp.net")?)
        .children(vec![
            Node::new("picture")
                .attr("type", "image")
                .bytes(jpeg.to_vec()),
        ]))
}

/// Build the lookup for `target`'s photo URL.
pub fn get_request(id: &str, target: &Jid) -> Result<Node> {
    if id.is_empty() || id.len() > 128 {
        return Err(invalid("invalid request id"));
    }
    if target.user.is_empty() {
        return Err(invalid("a lookup needs a user or group"));
    }
    Ok(Node::new("iq")
        .attr("id", id)
        .attr("xmlns", "w:profile:picture")
        .attr("type", "get")
        .attr("to", Jid::parse("s.whatsapp.net")?)
        .attr("target", target.to_non_ad())
        .children(vec![
            Node::new("picture")
                .attr("type", "image")
                .attr("query", "url"),
        ]))
}

/// Queue setting our photo; a session such as `listen` sends it.
pub fn queue_set(store: &ProtocolStore, jpeg: Vec<u8>) -> Result<String> {
    set_request("validation", &jpeg)?;
    super::actions::queue(store, SET_KIND, "profile photo", move |id| {
        set_request(id, &jpeg)
    })
}

/// Queue a photo lookup; `actions --show` reports the reply.
pub fn queue_get(store: &ProtocolStore, target: &Jid) -> Result<String> {
    get_request("validation", target)?;
    let target = target.clone();
    super::actions::queue(store, GET_KIND, "profile photo lookup", move |id| {
        get_request(id, &target)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_match_the_traced_builders() {
        let jpeg = [0xff, 0xd8, 0xff, 0xe0, 1, 2, 0xff, 0xd9];
        let set = set_request("A1", &jpeg).unwrap();
        assert_eq!(
            set.get_attr("xmlns").unwrap().as_text(),
            Some("w:profile:picture")
        );
        assert_eq!(set.get_attr("type").unwrap().as_text(), Some("set"));
        assert!(set.get_attr("target").is_none(), "own photo has no target");
        assert_eq!(set.attributes.len(), 4);
        assert_eq!(
            set.get_attr("to").unwrap().as_jid(),
            Some(&Jid::parse("s.whatsapp.net").unwrap())
        );
        let picture = set.child("picture").unwrap();
        assert_eq!(picture.get_attr("type").unwrap().as_text(), Some("image"));
        assert!(
            matches!(&picture.content, crate::binary::NodeContent::Bytes(b) if b.as_ref() == jpeg)
        );
        assert!(set_request("A1", b"not a jpeg").is_err());
        assert!(set_request("A1", &vec![0xff; MAX_BYTES + 1]).is_err());

        let target = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let get = get_request("A2", &target).unwrap();
        assert_eq!(get.get_attr("type").unwrap().as_text(), Some("get"));
        assert!(get.get_attr("target").is_some());
        assert_eq!(get.attributes.len(), 5);
        assert_eq!(
            get.get_attr("to").unwrap().as_jid(),
            Some(&Jid::parse("s.whatsapp.net").unwrap())
        );
        let picture = get.child("picture").unwrap();
        assert_eq!(picture.get_attr("query").unwrap().as_text(), Some("url"));
    }
}
