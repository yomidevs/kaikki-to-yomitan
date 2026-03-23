use crate::{Map, models::kaikki::Tag};
use serde::ser::{SerializeTuple, Serializer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum YomitanEntry {
    TermBank(TermBank),                     // 120 (24 * 5)
    TermBankSimplified(TermBankSimplified), // 120 (24 * 5)
    TermBankMeta(TermBankMeta),             // 104
}

impl YomitanEntry {
    pub const fn file_prefix(&self) -> &str {
        match self {
            Self::TermBank(_) | Self::TermBankSimplified(_) => "term_bank",
            Self::TermBankMeta(_) => "term_meta_bank",
        }
    }
}

// Simplified version to avoid storing fields that we don't use. Those are written later on via the
// serialize implementation.
//
// The skipped fields are (at index): frequency (4), sequence (6), term_tags (7)
//
// https://github.com/yomidevs/yomitan/blob/f271fc0da3e55a98fa91c9834d75fccc96deae27/ext/data/schemas/dictionary-term-bank-v3-schema.json
//
// https://github.com/MarvNC/yomichan-dict-builder/blob/master/src/types/yomitan/termbank.ts
// @ TermInformation
#[derive(Debug, Clone)]
pub struct TermBank(
    pub String,                  // term
    pub String,                  // reading
    pub String,                  // definition_tags
    pub String,                  // space-separated rules
    pub Vec<DetailedDefinition>, // definitions
);

impl Serialize for TermBank {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(8)?;
        tup.serialize_element(&self.0)?;
        tup.serialize_element(&self.1)?;
        tup.serialize_element(&self.2)?;
        tup.serialize_element(&self.3)?;
        tup.serialize_element(&0u8)?;
        tup.serialize_element(&self.4)?;
        tup.serialize_element(&0u8)?;
        tup.serialize_element(&"")?;
        tup.end()
    }
}

// Used for forms in the main dictionary: definition_tags and rules do not change.
// The objective is to minimize memory storage.
#[derive(Debug, Clone)]
pub struct TermBankSimplified(
    pub String,                  // term
    pub String,                  // reading
    pub String,                  // space-separated rules
    pub Vec<DetailedDefinition>, // definitions
);

impl Serialize for TermBankSimplified {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(8)?;
        tup.serialize_element(&self.0)?;
        tup.serialize_element(&self.1)?;
        tup.serialize_element(&"non-lemma")?;
        tup.serialize_element(&self.2)?;
        tup.serialize_element(&0u8)?;
        tup.serialize_element(&self.3)?;
        tup.serialize_element(&0u8)?;
        tup.serialize_element(&"")?;
        tup.end()
    }
}

// There are other variants that we don't use at the moment.
#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum TermBankMeta {
    TermPhoneticTranscription(TermPhoneticTranscription),
}

// https://github.com/yomidevs/yomitan/blob/f271fc0da3e55a98fa91c9834d75fccc96deae27/ext/data/schemas/dictionary-term-meta-bank-v3-schema.json
//
// https://github.com/MarvNC/yomichan-dict-builder/blob/master/src/types/yomitan/termbankmeta.ts
#[derive(Debug, Serialize, Clone)]
pub struct TermPhoneticTranscription(
    pub String,                // term
    pub String,                // static: "ipa"
    pub PhoneticTranscription, // phonetic transcription
);

#[derive(Debug, Serialize, Clone, PartialEq, Eq, Hash, Default)]
pub struct PhoneticTranscription {
    pub reading: String,
    pub transcriptions: Vec<Ipa>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct Ipa {
    pub ipa: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
}

// https://github.com/MarvNC/yomichan-dict-builder/blob/master/src/types/yomitan/termbank.ts
// @ StructuredContentNode
#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum Node {
    Text(String),              // 32
    Array(Vec<Node>),          // 32
    Generic(Box<GenericNode>), // 16
    Backlink(BacklinkContent), // 32
}

impl Node {
    /// Push a new node into the array variant.
    pub fn push(&mut self, node: Self) {
        match self {
            Self::Array(boxed_vec) => boxed_vec.push(node),
            _ => panic!("Error: called 'push' with a non Node::Array"),
        }
    }

    pub const fn new_array() -> Self {
        Self::Array(vec![])
    }

    #[must_use]
    pub fn into_array_node(self) -> Self {
        Self::Array(vec![self])
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct NodeData(pub Map<String, String>);

impl<K, V> FromIterator<(K, V)> for NodeData
where
    K: Into<String>,
    V: Into<String>,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let inner = iter
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        Self(inner)
    }
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum NTag {
    Span,
    Div,
    Ol,
    Ul,
    Li,
    Details,
    Summary,
}

// Fields are ordered for visualization and may be different from yomitan builder order.
#[derive(Debug, Serialize, Clone)]
pub struct GenericNode {
    pub tag: NTag,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<NodeData>,

    pub content: Node,
}

impl GenericNode {
    pub fn into_node(self) -> Node {
        Node::Generic(Box::new(self))
    }
}

// In the general case, this should be a String. We use an enum to shrink the size of Node.
#[derive(Debug, Clone)]
pub enum BacklinkContentKind {
    Wiktionary,
    Kaikki,
}

#[derive(Debug, Clone)]
pub struct BacklinkContent {
    pub href: String,
    pub content: BacklinkContentKind,
}

impl BacklinkContent {
    pub const fn new(href: String, content: BacklinkContentKind) -> Self {
        Self { href, content }
    }
}

// Custom Serialize to not have to store the constant 'a' tag
impl Serialize for BacklinkContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("BacklinkContent", 3)?;
        state.serialize_field("tag", "a")?;
        state.serialize_field("href", &self.href)?;
        state.serialize_field(
            "content",
            match &self.content {
                BacklinkContentKind::Wiktionary => "Wiktionary",
                BacklinkContentKind::Kaikki => "Kaikki",
            },
        )?;
        state.end()
    }
}

// https://github.com/MarvNC/yomichan-dict-builder/blob/master/src/types/yomitan/termbank.ts
// @ DetailedDefinition
#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum DetailedDefinition {
    Text(String),
    StructuredContent(StructuredContent),
    Inflection((String, Vec<String>)),
}

impl DetailedDefinition {
    pub fn structured(content: Node) -> Self {
        Self::StructuredContent(StructuredContent {
            ty: "structured-content".to_string(),
            content,
        })
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct StructuredContent {
    #[serde(rename = "type")]
    pub ty: String, // should be hardcoded to "structured-content" (but then to serialize it...)
    pub content: Node,
}

pub fn wrap(tag: NTag, content_ty: &str, content: Node) -> Node {
    GenericNode {
        tag,
        title: None, // hardcoded since most of the wrap calls don't use it
        data: match content_ty {
            "" => None,
            _ => Some(NodeData::from_iter([("content", content_ty)])),
        },
        content,
    }
    .into_node()
}

// Internal legacy types that are just for documentation since we ended up loading
// tag_bank_term.json as a raw list of tuples in tags::mod.rs
//
// #[derive(Deserialize, Default)]
// struct WhitelistedTags(Vec<WhitelistedTag>);
//
// // Internal type
// #[derive(Deserialize, Default)]
// struct WhitelistedTag {
//     short_tag: String,
//     category: String,
//     sort_order: i32,
//     aliases: Vec<String>, // only this changes
//     popularity_score: i32,
// }

// The actual yomitan type.
//
// https://github.com/MarvNC/yomichan-dict-builder/blob/master/src/types/yomitan/tagbank.ts
#[derive(Debug, PartialEq, Eq)]
pub struct TagInformation {
    pub short_tag: String, // tagName
    pub category: String,
    pub sort_order: i32,  // sortingOrder
    pub long_tag: String, // notes (only this changes)
    pub popularity_score: i32,
}

impl TagInformation {
    // The entry plays the role of the WhitelistedTag struct
    pub fn new(entry: &(&str, &str, i32, &[&str], i32)) -> Self {
        // The short tag should not contain a space: yomitan will split it then.
        debug_assert!(!entry.0.contains(' '));
        Self {
            short_tag: entry.0.into(),
            category: entry.1.into(),
            sort_order: entry.2,
            long_tag: entry.3[0].into(), // normalized
            popularity_score: entry.4,
        }
    }
}

impl Serialize for TagInformation {
    // serialize as array
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(5)?;
        tup.serialize_element(&self.short_tag)?;
        tup.serialize_element(&self.category)?;
        tup.serialize_element(&self.sort_order)?;
        tup.serialize_element(&self.long_tag)?;
        tup.serialize_element(&self.popularity_score)?;
        tup.end()
    }
}
