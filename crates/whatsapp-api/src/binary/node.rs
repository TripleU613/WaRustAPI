//! In-memory node shape.

use bytes::Bytes;

use crate::model::jid::Jid;

/// An attribute value. Encodings are preserved so a decoded node re-encodes
/// to an equivalent form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeValue {
    /// Text (token, packed or raw string).
    Text(String),
    /// A JID.
    Jid(Jid),
    /// Raw bytes that were not valid UTF-8. Never lossily converted.
    Bytes(Bytes),
}

impl AttributeValue {
    /// Text view, if this is text.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            _ => None,
        }
    }

    /// JID view, if this is a JID.
    pub fn as_jid(&self) -> Option<&Jid> {
        match self {
            Self::Jid(j) => Some(j),
            _ => None,
        }
    }
}

impl From<&str> for AttributeValue {
    fn from(s: &str) -> Self {
        Self::Text(s.to_owned())
    }
}

impl From<String> for AttributeValue {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<Jid> for AttributeValue {
    fn from(j: Jid) -> Self {
        Self::Jid(j)
    }
}

/// A named attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    /// Attribute name.
    pub name: String,
    /// Attribute value.
    pub value: AttributeValue,
}

/// Node content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum NodeContent {
    /// No content.
    #[default]
    Empty,
    /// Raw bytes.
    Bytes(Bytes),
    /// A token/packed/raw string that decoded as valid UTF-8 text.
    Text(String),
    /// A JID.
    Jid(Jid),
    /// Child nodes.
    Children(Vec<Node>),
}

/// A stanza node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Description (tag name).
    pub description: String,
    /// Attributes in wire order. Names are unique.
    pub attributes: Vec<Attribute>,
    /// Content.
    pub content: NodeContent,
}

impl Node {
    /// A node with no attributes and no content.
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            attributes: Vec::new(),
            content: NodeContent::Empty,
        }
    }

    /// Builder: add an attribute. Replaces an existing attribute of the same name.
    #[must_use]
    pub fn attr(mut self, name: impl Into<String>, value: impl Into<AttributeValue>) -> Self {
        let name = name.into();
        let value = value.into();
        if let Some(a) = self.attributes.iter_mut().find(|a| a.name == name) {
            a.value = value;
        } else {
            self.attributes.push(Attribute { name, value });
        }
        self
    }

    /// Builder: set children.
    #[must_use]
    pub fn children(mut self, children: Vec<Node>) -> Self {
        self.content = NodeContent::Children(children);
        self
    }

    /// Builder: set byte content.
    #[must_use]
    pub fn bytes(mut self, b: impl Into<Bytes>) -> Self {
        self.content = NodeContent::Bytes(b.into());
        self
    }

    /// Builder: set text content.
    #[must_use]
    pub fn text(mut self, t: impl Into<String>) -> Self {
        self.content = NodeContent::Text(t.into());
        self
    }

    /// Look up an attribute by name.
    pub fn get_attr(&self, name: &str) -> Option<&AttributeValue> {
        self.attributes
            .iter()
            .find(|a| a.name == name)
            .map(|a| &a.value)
    }

    /// Child nodes (empty slice when content is not children).
    pub fn child_nodes(&self) -> &[Node] {
        match &self.content {
            NodeContent::Children(c) => c,
            _ => &[],
        }
    }

    /// First child with the given description.
    pub fn child(&self, description: &str) -> Option<&Node> {
        self.child_nodes()
            .iter()
            .find(|n| n.description == description)
    }
}
