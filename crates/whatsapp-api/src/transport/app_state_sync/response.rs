//! Pure, opaque collection-response classification from APK 2.26.43.72
//! `F2T.<init>/A00/A01/A02`. See docs/protocol/app-state-fetch-completion.md.
//! This is not the complete protobuf parser, authenticated result processor or
//! request-correlation gate. Original nodes retain all unsupported information.
use super::Collection;
use crate::{
    Error, Result,
    binary::{Node, NodeContent},
};
use std::fmt;

// Local defensive limits, not recovered WhatsApp protocol requirements.
const MAX_ITEMS: usize = 4096;
const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;
const MAX_ATTRIBUTES: usize = 64;
const MAX_TEXT_BYTES: usize = 4096;

/// Borrowed response; no original content is discarded or rewritten.
#[derive(PartialEq, Eq)]
pub struct Response<'a> {
    /// Entire original IQ, including unknown attributes and children.
    pub original: &'a Node,
    /// Collection summaries in original wire order.
    pub collections: Vec<CollectionReply<'a>>,
}

/// One response collection, in the original wire order. Unknown names survive.
#[derive(PartialEq, Eq)]
pub struct CollectionReply<'a> {
    /// Entire original collection node, without normalization.
    pub original: &'a Node,
    /// Exact name, including names unsupported by this client.
    pub name: &'a str,
    /// Known collection when the exact name matches; never inferred.
    pub known_collection: Option<Collection>,
    /// Opaque classification, not a cryptographic or completion verdict.
    pub category: Category<'a>,
}

/// Native category precedence, without claiming protobuf/MAC validity.
#[derive(PartialEq, Eq)]
pub enum Category<'a> {
    /// Collection-level refusal, which precedes version/payload classification.
    Error {
        /// Signed native integer, or native sentinel -1 when absent.
        code: i32,
        /// Original optional text. Callers must not log its private contents.
        text: Option<&'a str>,
        /// Opaque data only for 409 with a nonempty patches-node child list.
        conflict_data: Option<CollectionData<'a>>,
    },
    /// Collection version acknowledgement, preceding payload classification.
    VersionAck {
        /// Signed native long; state eligibility is not established here.
        version: i64,
    },
    /// No snapshot and no nonempty patches-node child list.
    AlreadySynced,
    /// Opaque data requiring separate protobuf/cryptographic processing.
    Data(CollectionData<'a>),
}

/// Opaque snapshot reference and patch bodies. Snapshot reference bytes are
/// ExternalBlobReference, not downloaded SyncdSnapshot bytes.
#[derive(PartialEq, Eq)]
pub struct CollectionData<'a> {
    /// Original external-reference body, not downloaded snapshot data.
    pub snapshot_ref: Option<&'a [u8]>,
    /// Original patch bodies, not decoded or authenticated.
    pub patches: Vec<&'a [u8]>,
    /// True only for the exact attribute value `true`.
    pub has_more_patches: bool,
}

impl fmt::Debug for Response<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Response")
            .field("collection_count", &self.collections.len())
            .finish_non_exhaustive()
    }
}
impl fmt::Debug for CollectionReply<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CollectionReply")
            .field("known_collection", &self.known_collection.is_some())
            .field("category", &self.category)
            .finish_non_exhaustive()
    }
}
impl fmt::Debug for Category<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error { conflict_data, .. } => f
                .debug_struct("Error")
                .field("has_conflict_data", &conflict_data.is_some())
                .finish_non_exhaustive(),
            Self::VersionAck { .. } => f.write_str("VersionAck { .. }"),
            Self::AlreadySynced => f.write_str("AlreadySynced"),
            Self::Data(data) => f.debug_tuple("Data").field(data).finish(),
        }
    }
}
impl fmt::Debug for CollectionData<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CollectionData")
            .field("has_snapshot", &self.snapshot_ref.is_some())
            .field("patch_count", &self.patches.len())
            .field("has_more_patches", &self.has_more_patches)
            .finish_non_exhaustive()
    }
}

fn invalid(reason: &'static str) -> Error {
    Error::invalid("app_state_response", reason)
}

// Singleton/shape checks here are local admission policy. Native F2T uses
// first-child lookup; do not attribute duplicate refusal to the APK.
fn check_node(node: &Node) -> Result<()> {
    if node.attributes.len() > MAX_ATTRIBUTES || node.child_nodes().len() > MAX_ITEMS {
        return Err(invalid("response node exceeds local attribute/child bound"));
    }
    Ok(())
}
fn attr<'a>(node: &'a Node, name: &str) -> Result<Option<&'a str>> {
    let mut matches = node.attributes.iter().filter(|a| a.name == name);
    let Some(first) = matches.next() else {
        return Ok(None);
    };
    if matches.next().is_some() {
        return Err(invalid("ambiguous duplicate response attribute"));
    }
    let value = first
        .value
        .as_text()
        .ok_or_else(|| invalid("response attribute is not text"))?;
    if value.len() > MAX_TEXT_BYTES {
        return Err(invalid("response attribute exceeds local text bound"));
    }
    Ok(Some(value))
}
fn child<'a>(node: &'a Node, name: &str) -> Result<Option<&'a Node>> {
    let mut matches = node.child_nodes().iter().filter(|n| n.description == name);
    let value = matches.next();
    if matches.next().is_some() {
        return Err(invalid("ambiguous duplicate response child"));
    }
    Ok(value)
}
#[derive(Default)]
struct Budget {
    patches: usize,
    bytes: usize,
}
impl Budget {
    fn body<'a>(&mut self, node: &'a Node) -> Result<&'a [u8]> {
        check_node(node)?;
        let bytes = match &node.content {
            NodeContent::Bytes(bytes) => bytes.as_ref(),
            NodeContent::Text(text) => text.as_bytes(),
            _ => return Err(invalid("snapshot or patch body is missing")),
        };
        if bytes.is_empty() {
            return Err(invalid("snapshot or patch body is empty"));
        }
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .ok_or_else(|| invalid("response body size overflow"))?;
        if self.bytes > MAX_BODY_BYTES {
            return Err(invalid("response bodies exceed local byte bound"));
        }
        Ok(bytes)
    }
}

fn data<'a>(node: &'a Node, budget: &mut Budget) -> Result<CollectionData<'a>> {
    let snapshot = child(node, "snapshot")?;
    let patches_node = child(node, "patches")?;
    if snapshot.is_none() && patches_node.is_none() {
        return Err(invalid("collection data has no snapshot or patches"));
    }
    let snapshot_ref = snapshot.map(|n| budget.body(n)).transpose()?;
    let mut patches = Vec::new();
    if let Some(parent) = patches_node {
        check_node(parent)?;
        for patch in parent
            .child_nodes()
            .iter()
            .filter(|n| n.description == "patch")
        {
            budget.patches += 1;
            if budget.patches > MAX_ITEMS {
                return Err(invalid("response exceeds local patch count bound"));
            }
            patches.push(budget.body(patch)?);
        }
    }
    Ok(CollectionData {
        snapshot_ref,
        patches,
        has_more_patches: attr(node, "has_more_patches")? == Some("true"),
    })
}

/// Classify an IQ success response without authenticating or applying it.
///
/// Category precedence follows current native F2T. Protobuf version/exit-code/
/// external-reference checks from F2T.A00 remain a separate, unimplemented
/// stage: opaque data here must never be interpreted as verified app state.
/// Requiring IQ/result and rejecting ambiguous consumed singletons are local
/// admission policies; unknown children and collection names remain accessible
/// through the borrowed original nodes. Outer IQ errors belong to their own
/// existing refusal handler, not this success-response grammar.
pub fn classify_response(node: &Node) -> Result<Response<'_>> {
    check_node(node)?;
    if node.description != "iq" || attr(node, "type")? != Some("result") {
        return Err(invalid(
            "expected an IQ result for collection classification",
        ));
    }
    let sync = child(node, "sync")?.ok_or_else(|| invalid("response has no sync child"))?;
    check_node(sync)?;
    let mut collections = Vec::new();
    let mut budget = Budget::default();
    for collection in sync
        .child_nodes()
        .iter()
        .filter(|n| n.description == "collection")
    {
        check_node(collection)?;
        let name =
            attr(collection, "name")?.ok_or_else(|| invalid("collection name is missing"))?;
        let category = if attr(collection, "type")? == Some("error") {
            let error = child(collection, "error")?
                .ok_or_else(|| invalid("collection error child is missing"))?;
            check_node(error)?;
            // Current 0Tp.A05(code,-1) defaults only when the attribute is absent.
            let code = match attr(error, "code")? {
                None => -1,
                Some(raw) => super::super::mex::java_decimal_long(raw)
                    .and_then(|n| i32::try_from(n).ok())
                    .ok_or_else(|| invalid("collection error code is not a signed Java integer"))?,
            };
            let text = attr(error, "text")?;
            let conflict_data = if code == 409 {
                let patches = child(collection, "patches")?;
                if let Some(patches) = patches {
                    check_node(patches)?;
                }
                if patches.is_some_and(|p| !p.child_nodes().is_empty()) {
                    Some(data(collection, &mut budget)?)
                } else {
                    None
                }
            } else {
                None
            };
            Category::Error {
                code,
                text,
                conflict_data,
            }
        } else if let Some(raw) = attr(collection, "version")? {
            // Reuse the retained Java UTF-16/BMP-digit parser; ASCII-only
            // parsing would refuse values accepted by the native long helper.
            let version = super::super::mex::java_decimal_long(raw)
                .ok_or_else(|| invalid("collection version is not a signed Java long"))?;
            if version == -1 {
                return Err(invalid(
                    "collection version is the missing-version sentinel",
                ));
            }
            Category::VersionAck { version }
        } else {
            let patches = child(collection, "patches")?;
            if let Some(patches) = patches {
                check_node(patches)?;
            }
            let snapshot = child(collection, "snapshot")?;
            // F2T tests ALL children for emptiness, then A00 selects patch tags.
            if patches.is_none_or(|p| p.child_nodes().is_empty()) && snapshot.is_none() {
                Category::AlreadySynced
            } else {
                Category::Data(data(collection, &mut budget)?)
            }
        };
        collections.push(CollectionReply {
            original: collection,
            name,
            known_collection: Collection::from_name(name),
            category,
        });
    }
    Ok(Response {
        original: node,
        collections,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::{Attribute, AttributeValue};

    fn response(collections: Vec<Node>) -> Node {
        Node::new("iq")
            .attr("type", "result")
            .children(vec![Node::new("sync").children(collections)])
    }
    fn collection() -> Node {
        Node::new("collection").attr("name", "regular")
    }
    fn patches(children: Vec<Node>) -> Node {
        Node::new("patches").children(children)
    }

    #[test]
    fn categories_preserve_order_unknown_names_and_original_nodes() {
        let nodes = response(vec![
            Node::new("opaque").bytes(vec![99]),
            collection()
                .attr("version", "+2")
                .children(vec![Node::new("snapshot")]),
            Node::new("collection")
                .attr("name", "future-secret-name")
                .attr("opaque", "private")
                .children(vec![Node::new("future").bytes(vec![4])]),
            collection().children(vec![patches(vec![Node::new("patch").bytes(vec![0xff, 0])])]),
        ]);
        let parsed = classify_response(&nodes).unwrap();
        assert!(std::ptr::eq(parsed.original, &nodes));
        assert_eq!(parsed.collections.len(), 3);
        assert!(matches!(
            parsed.collections[0].category,
            Category::VersionAck { version: 2 }
        ));
        assert_eq!(parsed.collections[1].name, "future-secret-name");
        assert!(parsed.collections[1].known_collection.is_none());
        assert!(std::ptr::eq(
            parsed.collections[1].original,
            &nodes.child("sync").unwrap().child_nodes()[2]
        ));
        assert!(matches!(
            parsed.collections[1].category,
            Category::AlreadySynced
        ));
        let Category::Data(data) = &parsed.collections[2].category else {
            panic!("data category");
        };
        assert_eq!(data.patches, vec![&[0xff, 0][..]]);
    }

    #[test]
    fn error_precedes_version_and_defaults_absent_code() {
        let nodes = response(vec![
            collection()
                .attr("type", "error")
                .attr("version", "invalid")
                .children(vec![
                    Node::new("error").attr("text", "secret"),
                    Node::new("snapshot"),
                ]),
        ]);
        let parsed = classify_response(&nodes).unwrap();
        let Category::Error {
            code,
            text,
            conflict_data,
        } = &parsed.collections[0].category
        else {
            panic!("error category");
        };
        assert_eq!(*code, -1);
        assert_eq!(*text, Some("secret"));
        assert!(conflict_data.is_none());
    }

    #[test]
    fn conflict_data_requires_any_patches_child_and_preserves_opaque_body() {
        for code in [409, 400] {
            for content in [vec![], vec![Node::new("future")]] {
                let nodes = response(vec![collection().attr("type", "error").children(vec![
                    Node::new("error").attr("code", code.to_string()),
                    patches(content.clone()),
                    Node::new("snapshot").bytes(vec![0xff]),
                ])]);
                let parsed = classify_response(&nodes).unwrap();
                let Category::Error { conflict_data, .. } = &parsed.collections[0].category else {
                    panic!("error category");
                };
                assert_eq!(conflict_data.is_some(), code == 409 && !content.is_empty());
                if let Some(data) = conflict_data {
                    assert_eq!(data.snapshot_ref, Some(&[0xff][..]));
                    assert!(data.patches.is_empty());
                }
            }
        }
        let nodes = response(vec![collection().attr("type", "error").children(vec![
            Node::new("error").attr("code", "409"),
            Node::new("snapshot"),
        ])]);
        assert!(matches!(
            classify_response(&nodes).unwrap().collections[0].category,
            Category::Error {
                conflict_data: None,
                ..
            }
        ));
    }

    #[test]
    fn empty_and_unknown_only_patches_have_distinct_categories() {
        for children in [vec![], vec![patches(vec![])]] {
            let nodes = response(vec![collection().children(children)]);
            assert!(matches!(
                classify_response(&nodes).unwrap().collections[0].category,
                Category::AlreadySynced
            ));
        }
        let nodes = response(vec![
            collection().children(vec![patches(vec![Node::new("unknown")])]),
        ]);
        let parsed = classify_response(&nodes).unwrap();
        assert!(
            matches!(&parsed.collections[0].category, Category::Data(data) if data.patches.is_empty())
        );
    }

    #[test]
    fn has_more_is_literal_true_and_text_bodies_are_borrowed_bytes() {
        for value in ["true", "TRUE", "1", "false", ""] {
            let nodes = response(vec![
                collection()
                    .attr("has_more_patches", value)
                    .children(vec![Node::new("snapshot").text("opaque")]),
            ]);
            let parsed = classify_response(&nodes).unwrap();
            let Category::Data(data) = &parsed.collections[0].category else {
                panic!("data category");
            };
            assert_eq!(data.has_more_patches, value == "true");
            assert_eq!(data.snapshot_ref, Some(&b"opaque"[..]));
        }
    }

    #[test]
    fn malformed_numeric_and_missing_fields_fail_without_echoing_content() {
        for value in ["", "-", " 1", "1 ", "1_0", "private-number", "2147483648"] {
            let nodes = response(vec![
                collection()
                    .attr("type", "error")
                    .children(vec![Node::new("error").attr("code", value)]),
            ]);
            assert!(classify_response(&nodes).is_err());
        }
        for value in ["", "-1", "1.0", "9223372036854775808", "private-version"] {
            assert!(
                classify_response(&response(vec![collection().attr("version", value)])).is_err()
            );
        }
        for node in [
            Node::new("collection"),
            collection().attr("type", "error"),
            collection().children(vec![Node::new("snapshot")]),
            collection().children(vec![patches(vec![Node::new("patch").bytes(Vec::new())])]),
        ] {
            assert!(classify_response(&response(vec![node])).is_err());
        }
        let nodes = response(vec![collection().attr("version", "-2")]);
        assert!(matches!(
            classify_response(&nodes).unwrap().collections[0].category,
            Category::VersionAck { version: -2 }
        ));
    }

    #[test]
    fn native_java_numeric_domains_are_preserved() {
        let nodes = response(vec![
            collection().attr("version", "+１２３"),
            collection().attr("version", "-9223372036854775808"),
            collection()
                .attr("type", "error")
                .children(vec![Node::new("error").attr("code", "４０９")]),
        ]);
        let parsed = classify_response(&nodes).unwrap();
        assert!(matches!(
            parsed.collections[0].category,
            Category::VersionAck { version: 123 }
        ));
        assert!(matches!(
            parsed.collections[1].category,
            Category::VersionAck { version: i64::MIN }
        ));
        assert!(matches!(
            parsed.collections[2].category,
            Category::Error { code: 409, .. }
        ));
        let nodes = response(vec![collection().attr("version", "𝟙")]);
        assert!(
            classify_response(&nodes).is_err(),
            "supplementary digits are two Java chars"
        );
    }

    #[test]
    fn local_ambiguity_guards_and_outer_success_boundary() {
        assert!(classify_response(&Node::new("iq").attr("type", "result")).is_err());
        assert!(classify_response(&response(vec![]).attr("type", "error")).is_err());
        let mut duplicate = collection();
        duplicate.attributes.push(Attribute {
            name: "name".into(),
            value: AttributeValue::from("private-name"),
        });
        assert!(classify_response(&response(vec![duplicate])).is_err());
        assert!(
            classify_response(&response(vec![collection().children(vec![
                Node::new("snapshot").bytes(vec![1]),
                Node::new("snapshot").bytes(vec![2])
            ])]))
            .is_err()
        );
        // Global duplicate collection names are preserved, not attributed to a
        // native uniqueness rule absent from the constructor evidence.
        assert_eq!(
            classify_response(&response(vec![collection(), collection()]))
                .unwrap()
                .collections
                .len(),
            2
        );
    }

    #[test]
    fn local_count_and_aggregate_body_limits() {
        let nodes = response(vec![collection(); MAX_ITEMS + 1]);
        assert!(classify_response(&nodes).is_err());
        let nodes = response(vec![collection().children(vec![
            Node::new("snapshot").bytes(vec![0; MAX_BODY_BYTES + 1]),
        ])]);
        assert!(classify_response(&nodes).is_err());
        let nodes = response(vec![
            collection().attr("name", "x".repeat(MAX_TEXT_BYTES + 1)),
        ]);
        assert!(classify_response(&nodes).is_err());
    }

    #[test]
    fn debug_redacts_names_text_versions_originals_and_bodies() {
        let nodes = response(vec![
            collection()
                .attr("name", "private-name")
                .attr("type", "error")
                .children(vec![
                    Node::new("error")
                        .attr("code", "409")
                        .attr("text", "private-error"),
                    patches(vec![Node::new("patch").text("private-patch")]),
                    Node::new("snapshot").text("private-reference"),
                ]),
            collection().attr("version", "123456789"),
        ]);
        let parsed = classify_response(&nodes).unwrap();
        let debug = format!(
            "{parsed:?} {:?} {:?}",
            parsed.collections, parsed.collections[0].category
        );
        for secret in [
            "private-name",
            "private-error",
            "private-patch",
            "private-reference",
            "123456789",
            "409",
        ] {
            assert!(!debug.contains(secret));
        }
    }
}
