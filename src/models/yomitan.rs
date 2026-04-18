//! Yomitan data model.
//!
//! Most of the structs and enums have been simplified and trimmed of unused fields for performance.
//!
//! Ported from the typescript [yomitan-dict-builder] library. See also the [spec].
//!
//! [yomitan-dict-builder]: https://github.com/MarvNC/yomichan-dict-builder/tree/master/src/types/yomitan
//! [spec]: https://github.com/yomidevs/yomitan/tree/master/ext/data/schemas

use serde::ser::{SerializeStruct, SerializeTuple, Serializer};
use serde::{Deserialize, Serialize};

use crate::{Map, models::kaikki::Tag};

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum YomitanEntry {
    TermInfo(TermInfo),         // 120 (24 * 5)
    TermInfoForm(TermInfoForm), // 120 (24 * 5)
    TermMeta(TermMeta),         // 104
}

impl YomitanEntry {
    pub const fn file_prefix(&self) -> &'static str {
        match self {
            Self::TermInfo(_) | Self::TermInfoForm(_) => "term_bank",
            Self::TermMeta(_) => "term_meta_bank",
        }
    }

    pub fn term(&self) -> &str {
        match self {
            YomitanEntry::TermInfo(t) => t.term.as_str(),
            YomitanEntry::TermInfoForm(t) => t.term.as_str(),
            YomitanEntry::TermMeta(t) => {
                let TermMeta::TermPhoneticTranscription(t) = t;
                t.term.as_str()
            }
        }
    }
}

// Simplified version to avoid storing some fields. Those are written later on via serialize.
//
// The skipped fields are (at index): frequency (4), sequence (6), term_tags (7)
//
/// A term information. See [yomitan-dict-builder] and the [spec].
///
/// [yomitan-dict-builder]: https://github.com/MarvNC/yomichan-dict-builder/blob/master/src/types/yomitan/termbank.ts#L159
/// [spec]: https://github.com/yomidevs/yomitan/blob/master/ext/data/schemas/dictionary-term-bank-v3-schema.json
#[derive(Debug, Clone)]
pub struct TermInfo {
    pub term: String,
    pub reading: String,
    // While we could store just a String, and let Yomitan deal with it, it is
    // preferable to keep the full information for other formats based on the
    // Yomitan model, than can not defer this work.
    pub definition_tags: Vec<TagInfo>,
    pub rules: String, // space-separated rules
    pub definitions: Vec<DetailedDefinition>,
}

impl TermInfo {
    pub fn new(
        term: String,
        reading: String,
        definition_tags: Vec<TagInfo>,
        rules: String,
        definitions: Vec<DetailedDefinition>,
    ) -> Self {
        // INVARIANT: yomitan discards the reading if it's equal to term,
        // but we don't want it to pollute other formats.
        debug_assert_ne!(term, reading);
        Self {
            term,
            reading,
            definition_tags,
            rules,
            definitions,
        }
    }
}

impl Serialize for TermInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(8)?;
        tup.serialize_element(&self.term)?;
        tup.serialize_element(&self.reading)?;
        // We only retain the short versions. Yomitan will resolve them via tag_bank.
        let definition_tags_str = self
            .definition_tags
            .iter()
            .map(|tag_info| tag_info.short_tag.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        tup.serialize_element(&definition_tags_str)?;
        tup.serialize_element(&self.rules)?;
        tup.serialize_element(&0u8)?;
        tup.serialize_element(&self.definitions)?;
        tup.serialize_element(&0u8)?;
        tup.serialize_element(&"")?;
        tup.end()
    }
}

/// A term information with hardcoded definition tags. Used in forms.
#[derive(Debug, Clone)]
pub struct TermInfoForm {
    pub term: String,
    pub reading: String,
    pub rules: String, // space-separated rules
    pub definitions: Vec<DetailedDefinition>,
}

impl TermInfoForm {
    pub fn new(
        term: String,
        reading: String,
        rules: String,
        definitions: Vec<DetailedDefinition>,
    ) -> Self {
        // INVARIANT: yomitan discards the reading if it's equal to term,
        // but we don't want it to pollute other formats.
        debug_assert_ne!(term, reading);
        Self {
            term,
            reading,
            rules,
            definitions,
        }
    }
}

impl Serialize for TermInfoForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(8)?;
        tup.serialize_element(&self.term)?;
        tup.serialize_element(&self.reading)?;
        tup.serialize_element(&"non-lemma")?;
        tup.serialize_element(&self.rules)?;
        tup.serialize_element(&0u8)?;
        tup.serialize_element(&self.definitions)?;
        tup.serialize_element(&0u8)?;
        tup.serialize_element(&"")?;
        tup.end()
    }
}

/// A term meta entry. See [yomitan-dict-builder].
///
/// Trivial enum. There are other variants that we don't use.
///
/// [yomitan-dict-builder]: https://github.com/MarvNC/yomichan-dict-builder/blob/master/src/types/yomitan/termbankmeta.ts#L49
#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum TermMeta {
    TermPhoneticTranscription(TermPhoneticTranscription),
}

// https://github.com/yomidevs/yomitan/blob/f271fc0da3e55a98fa91c9834d75fccc96deae27/ext/data/schemas/dictionary-term-meta-bank-v3-schema.json
//
// https://github.com/MarvNC/yomichan-dict-builder/blob/master/src/types/yomitan/termbankmeta.ts
#[derive(Debug, Clone)]
pub struct TermPhoneticTranscription {
    pub term: String,
    pub transcription: PhoneticTranscription,
}

impl TermPhoneticTranscription {
    pub const fn new(term: String, transcription: PhoneticTranscription) -> Self {
        Self {
            term,
            transcription,
        }
    }
}

impl Serialize for TermPhoneticTranscription {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(3)?;
        tup.serialize_element(&self.term)?;
        tup.serialize_element(&"ipa")?;
        tup.serialize_element(&self.transcription)?;
        tup.end()
    }
}

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

/// A structured content node. See [yomitan-dict-builder].
///
/// [yomitan-dict-builder]: https://github.com/MarvNC/yomichan-dict-builder/blob/master/src/types/yomitan/termbank.ts#L91
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

#[derive(Debug, Serialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum NodeDataKey {
    Content,
    Category,
}

#[derive(Debug, Serialize, Clone)]
pub struct NodeData(pub Map<NodeDataKey, String>);

impl<V> FromIterator<(NodeDataKey, V)> for NodeData
where
    V: Into<String>,
{
    fn from_iter<I: IntoIterator<Item = (NodeDataKey, V)>>(iter: I) -> Self {
        let inner = iter.into_iter().map(|(k, v)| (k, v.into())).collect();
        Self(inner)
    }
}

/// A [`GenericNode`] tag.
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

/// One of the possible values of the [`Node`] enum.
///
/// Note that fields are ordered for visualization and may be different from yomitan builder order.
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
#[derive(Debug, Clone, Serialize)]
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
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("BacklinkContent", 3)?;
        state.serialize_field("tag", "a")?;
        state.serialize_field("href", &self.href)?;
        state.serialize_field("content", &self.content)?;
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
        Self::StructuredContent(StructuredContent { content })
    }
}

#[derive(Debug, Clone)]
pub struct StructuredContent {
    pub content: Node,
}

// Custom Serialize to not have to store the constant 'structured-content' type
impl Serialize for StructuredContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("StructuredContent", 2)?;
        state.serialize_field("type", "structured-content")?;
        state.serialize_field("content", &self.content)?;
        state.end()
    }
}

pub fn wrap(tag: NTag, content_ty: &str, content: Node) -> Node {
    GenericNode {
        tag,
        title: None, // hardcoded since most of the wrap calls don't use it
        data: match content_ty {
            "" => None,
            _ => Some(NodeData::from_iter([(NodeDataKey::Content, content_ty)])),
        },
        content,
    }
    .into_node()
}

/// A tag. See [yomitan-dict-builder].
///
/// [yomitan-dict-builder]: https://github.com/MarvNC/yomichan-dict-builder/blob/master/src/types/yomitan/tagbank.ts
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagInfo {
    pub short_tag: String, // tagName
    pub category: String,  // category
    pub sort_order: i32,   // sortingOrder
    pub long_tag: String,  // notes (only this changes)
    pub popularity_score: i32,
}

impl TagInfo {
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

impl Serialize for TagInfo {
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
