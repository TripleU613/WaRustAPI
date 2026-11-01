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
//! here. The builder writes no `to`; like this client's other server IQs we
//! address `s.whatsapp.net` (**inferred**). `8kn.A00` builds the lookup
//! `<iq id xmlns="w:profile:picture" type="get" target=<jid>>
//! <picture type="image" query="url"/></iq>` (its optional `common_gid` and
//! `tctoken` children are not sent). The reply carries the URL; nothing is
//! downloaded here.
//!
//! **Not traced:** the app's crop/resize/JPEG settings. Supply a square JPEG
//! (the official app uploads 640×640); the server may refuse others.
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
        let picture = get.child("picture").unwrap();
        assert_eq!(picture.get_attr("query").unwrap().as_text(), Some("url"));
    }
}
