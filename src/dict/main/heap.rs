use std::mem::size_of;

use super::*;
use crate::{
    Map,
    models::yomitan::{
        Ipa, PhoneticTranscription, StructuredContent, TermBankMeta, TermPhoneticTranscription,
    },
};

pub trait HeapSize {
    fn heap_size(&self) -> usize;
}

impl HeapSize for String {
    fn heap_size(&self) -> usize {
        self.capacity()
    }
}

impl HeapSize for Example {
    fn heap_size(&self) -> usize {
        self.text.heap_size() + self.translation.heap_size() + self.reference.heap_size()
    }
}

impl<T: HeapSize> HeapSize for Vec<T> {
    fn heap_size(&self) -> usize {
        self.capacity() * size_of::<T>() + self.iter().map(HeapSize::heap_size).sum::<usize>()
    }
}

impl HeapSize for LemmaKey {
    fn heap_size(&self) -> usize {
        self.lemma.heap_size() + self.reading.heap_size() + self.pos.heap_size()
    }
}

impl HeapSize for FormKey {
    fn heap_size(&self) -> usize {
        self.uninflected.heap_size() + self.inflected.heap_size() + self.pos.heap_size()
    }
}

impl HeapSize for FormSource {
    fn heap_size(&self) -> usize {
        0
    }
}

impl<A, B> HeapSize for (A, B)
where
    A: HeapSize,
    B: HeapSize,
{
    fn heap_size(&self) -> usize {
        self.0.heap_size() + self.1.heap_size()
    }
}

impl<K: HeapSize, V: HeapSize> HeapSize for Map<K, V> {
    fn heap_size(&self) -> usize {
        self.capacity() * (size_of::<K>() + size_of::<V>())
            + self
                .iter()
                .map(|(k, v)| k.heap_size() + v.heap_size())
                .sum::<usize>()
    }
}

impl HeapSize for LemmaInfo {
    fn heap_size(&self) -> usize {
        self.gloss_tree.heap_size()
            + self.etymology_text.as_ref().map_or(0, HeapSize::heap_size)
            + self.head_info_text.as_ref().map_or(0, HeapSize::heap_size)
            + self.link_wiktionary.heap_size()
            + self.link_kaikki.heap_size()
    }
}

impl HeapSize for GlossInfo {
    fn heap_size(&self) -> usize {
        self.tags.heap_size()
            + self.topics.heap_size()
            + self.examples.heap_size()
            + self.children.heap_size()
    }
}

impl HeapSize for LemmaMap {
    fn heap_size(&self) -> usize {
        self.0.heap_size()
    }
}

impl HeapSize for FormMap {
    fn heap_size(&self) -> usize {
        self.0.heap_size()
    }
}

impl HeapSize for Tidy {
    fn heap_size(&self) -> usize {
        self.lemma_map.heap_size() + self.form_map.heap_size()
    }
}

// YomitanEntry
impl<T: HeapSize> HeapSize for Box<T> {
    fn heap_size(&self) -> usize {
        size_of::<T>() + (**self).heap_size()
    }
}

impl HeapSize for YomitanEntry {
    fn heap_size(&self) -> usize {
        match self {
            Self::TermBank(tb) => tb.heap_size(),
            Self::TermBankSimplified(tbs) => tbs.heap_size(),
            Self::TermBankMeta(tbm) => tbm.heap_size(),
        }
    }
}

impl HeapSize for TermBank {
    fn heap_size(&self) -> usize {
        self.0.heap_size() // term
                + self.1.heap_size() // reading
                + self.2.heap_size() // definition_tags
                + self.3.heap_size() // rules
                + self.4.heap_size() // definitions
    }
}

impl HeapSize for TermBankSimplified {
    fn heap_size(&self) -> usize {
        self.0.heap_size() // term
                + self.1.heap_size() // reading
                + self.2.heap_size() // definitions
    }
}

impl HeapSize for TermBankMeta {
    fn heap_size(&self) -> usize {
        match self {
            Self::TermPhoneticTranscription(tpt) => tpt.heap_size(),
        }
    }
}

impl HeapSize for TermPhoneticTranscription {
    fn heap_size(&self) -> usize {
        self.0.heap_size() // term
                + self.1.heap_size() // PhoneticTranscription
    }
}

impl HeapSize for PhoneticTranscription {
    fn heap_size(&self) -> usize {
        self.reading.heap_size() + self.transcriptions.heap_size()
    }
}

impl HeapSize for Ipa {
    fn heap_size(&self) -> usize {
        self.ipa.heap_size() + self.tags.heap_size()
    }
}

impl HeapSize for DetailedDefinition {
    fn heap_size(&self) -> usize {
        match self {
            Self::Text(s) => s.heap_size(),
            Self::StructuredContent(sc) => sc.heap_size(),
            Self::Inflection((s, v)) => s.heap_size() + v.heap_size(),
        }
    }
}

impl HeapSize for StructuredContent {
    fn heap_size(&self) -> usize {
        self.ty.heap_size() + self.content.heap_size()
    }
}

impl HeapSize for Node {
    fn heap_size(&self) -> usize {
        match self {
            Self::Text(s) => s.heap_size(),
            Self::Array(v) => v.heap_size(),
            Self::Generic(boxed) => boxed.heap_size(),
            Self::Backlink(bl) => bl.heap_size(),
        }
    }
}

impl HeapSize for GenericNode {
    fn heap_size(&self) -> usize {
        self.title.as_ref().map_or(0, HeapSize::heap_size)
            + self.data.as_ref().map_or(0, HeapSize::heap_size)
            + self.content.heap_size()
    }
}

impl HeapSize for NodeData {
    fn heap_size(&self) -> usize {
        self.0.heap_size()
    }
}

impl HeapSize for BacklinkContent {
    fn heap_size(&self) -> usize {
        self.href.heap_size()
    }
}

impl HeapSize for BacklinkContentKind {
    fn heap_size(&self) -> usize {
        0 // enum discriminant is on the stack
    }
}

impl HeapSize for LabelledYomitanEntries {
    fn heap_size(&self) -> usize {
        0
        // self.entries.heap_size()
    }
}
