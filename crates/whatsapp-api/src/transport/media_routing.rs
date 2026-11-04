//! Pure media host projection; not authenticated request provenance or a cache.
//! APK 2.26.43.72 0VM.A0A/1xY.CK4; retained evidence:
//! docs/protocol/rea/2.26.43.72-media-route-lifetimes/README.md.
//! Callers must retain originals and establish the prior-route context separately.
use crate::{
    Error, Result,
    binary::{Node, NodeContent},
};

fn invalid(reason: &'static str) -> Error {
    Error::invalid("media_routing", reason)
}

/// Parse the native replacement flag (missing defaults to integer 1).
/// Duplicate attributes and oversized structures are local admission restrictions.
pub fn is_new_route(conn: &Node) -> Result<bool> {
    if conn.description != "media_conn" || conn.attributes.len() > 128 {
        return Err(invalid("unsupported media connection shape"));
    }
    let mut names = std::collections::BTreeSet::new();
    if conn.attributes.iter().any(|attr| !names.insert(&attr.name)) {
        return Err(invalid("ambiguous media connection attributes"));
    }
    if !matches!(conn.content, NodeContent::Empty | NodeContent::Children(_))
        || conn.child_nodes().len() > 1024
    {
        return Err(invalid("unsupported media connection content"));
    }
    match conn.get_attr("is_new") {
        None => Ok(true),
        Some(value) => value
            .as_text()
            .and_then(super::mex::java_decimal_long)
            .and_then(|v| i32::try_from(v).ok())
            .map(|v| v == 1)
            .ok_or_else(|| invalid("invalid media route replacement flag")),
    }
}

/// Project native host-list selection using caller-supplied prior context.
/// This does not authenticate either node, resolve host permissions, check expiry,
/// or prove the prior object was the one held when a request was sent. Returned
/// nodes contain private auth/host metadata and must not be logged. Originals
/// remain untouched. Unknown incoming non-host children are retained separately
/// by the caller's original response, not interpreted as routing metadata here.
pub fn project_media_conn(prior: Option<&Node>, incoming: &Node) -> Result<Node> {
    let is_new = is_new_route(incoming)?;
    if let Some(prior) = prior {
        is_new_route(prior)?;
    }
    // 0VM.A0A: a first route always installs the incoming object. Only a
    // non-new update with an existing object borrows that object's host list.
    let source = if is_new {
        incoming
    } else {
        prior.unwrap_or(incoming)
    };
    let hosts = source
        .child_nodes()
        .iter()
        .filter(|node| node.description == "host")
        .cloned()
        .collect();
    // Every attribute remains from the new object: never pair fresh hosts with
    // old auth/TTL, or old hosts with old auth when the response rotates it.
    Ok(incoming.clone().children(hosts))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn conn(id: &str, auth: &str, host: &str) -> Node {
        Node::new("media_conn")
            .attr("id", id)
            .attr("auth", auth)
            .attr("ttl", "3600")
            .attr("auth_ttl", "1800")
            .children(vec![
                Node::new("host")
                    .attr("hostname", host)
                    .attr("class", "synthetic-class")
                    .children(vec![Node::new("upload").children(vec![Node::new("image")])]),
            ])
    }
    #[test]
    fn conditional_merge_keeps_exact_prior_hosts_and_new_metadata() {
        let old = conn("old-id", "old-auth", "old.whatsapp.net");
        let new = conn("new-id", "new-auth", "new.whatsapp.net")
            .attr("is_new", "0")
            .attr("ttl", "7200")
            .attr("auth_ttl", "90");
        let before = (old.clone(), new.clone());
        let merged = project_media_conn(Some(&old), &new).unwrap();
        assert_eq!(merged.child_nodes(), old.child_nodes());
        assert_eq!(merged.attributes, new.attributes);
        assert_eq!((old, new), before);
        assert_eq!(merged.get_attr("auth").unwrap().as_text(), Some("new-auth"));
    }
    #[test]
    fn first_route_and_new_updates_take_incoming_hosts() {
        let old = conn("old", "old-auth", "old.whatsapp.net");
        for flag in ["0", "1", "2", "-1"] {
            let new = conn("new", "new-auth", "new.whatsapp.net").attr("is_new", flag);
            let first = project_media_conn(None, &new).unwrap();
            assert_eq!(first, new);
            let updated = project_media_conn(Some(&old), &new).unwrap();
            assert_eq!(
                updated.child_nodes(),
                if flag == "1" {
                    new.child_nodes()
                } else {
                    old.child_nodes()
                }
            );
        }
    }
    #[test]
    fn invalid_claims_refuse_without_mutating_inputs() {
        let old = conn("old", "old-auth", "old.whatsapp.net");
        for flag in ["bad", "2147483648", " 1"] {
            let new = conn("new", "new-auth", "new.whatsapp.net").attr("is_new", flag);
            assert!(project_media_conn(Some(&old), &new).is_err());
        }
        let new = conn("new", "new-auth", "new.whatsapp.net");
        let mut duplicate = new.clone();
        duplicate.attributes.push(duplicate.attributes[0].clone());
        assert!(project_media_conn(Some(&old), &duplicate).is_err());
        assert!(project_media_conn(Some(&Node::new("other")), &new).is_err());
        assert_eq!(old.get_attr("auth").unwrap().as_text(), Some("old-auth"));
    }
}
