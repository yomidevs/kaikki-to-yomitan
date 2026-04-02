use std::{fs::File, io::BufWriter, sync::LazyLock};

use anyhow::Result;
use indexmap::map::Entry;
use regex::Regex;
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use crate::{
    Map, Set,
    cli::{LangSpecs, MainArgs, Options},
    dict::{
        Dictionary, Intermediate, LabelledYomitanEntry, Langs,
        locale::{
            localize_etymology_string, localize_examples_string, localize_grammar_string,
            localize_synonyms_string,
        },
    },
    lang::{Edition, Lang},
    models::{
        kaikki::{Example, Form, HeadTemplate, Offset, Pos, Sense, Synonym, Tag, WordEntry},
        yomitan::{
            BacklinkContent, BacklinkContentKind, DetailedDefinition, GenericNode, NTag, Node,
            NodeData, TermBank, TermBankSimplified, YomitanEntry, wrap,
        },
    },
    path::PathManager,
    tags::{
        REDUNDANT_FORM_TAGS, find_short_pos_or_default, find_tag_in_bank, merge_person_tags,
        remove_redundant_tags, sort_tags, sort_tags_by_similar, tags_localization::localize_tag,
    },
    utils::{link_kaikki, link_wiktionary, pretty_println_at_path},
};

pub mod heap {
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

    impl HeapSize for LabelledYomitanEntry {
        fn heap_size(&self) -> usize {
            0
            // self.entries.heap_size()
        }
    }
}

use heap::HeapSize;

#[derive(Debug, Clone, Copy)]
pub struct DMain;

impl Intermediate for Tidy {
    fn len(&self) -> usize {
        self.len()
    }

    fn write(&self, pm: &PathManager) -> Result<()> {
        self.write(pm)
    }
}

impl Dictionary for DMain {
    type I = Tidy;
    type A = MainArgs;

    fn preprocess(&self, langs: Langs, entry: &mut WordEntry, opts: &Options, irs: &mut Self::I) {
        preprocess_main(langs.edition, langs.source, opts, entry, irs);
    }

    fn process(&self, langs: Langs, entry: &WordEntry, irs: &mut Self::I) {
        process_main(langs.edition, langs.source, entry, irs);
    }

    fn found_ir_message(&self, irs: &Self::I) {
        let n_lemmas = irs.lemma_map.len();
        let n_forms = irs.form_map.len();
        let n_irs = n_lemmas + n_forms;
        let n_forms_inflection = irs.form_map.len_inflection();
        let n_forms_extracted = irs.form_map.len_extracted();
        let n_forms_alt_of = irs.form_map.len_alt_of();

        debug_assert_eq!(
            n_forms,
            n_forms_inflection + n_forms_extracted + n_forms_alt_of,
            "mismatch in form counts"
        );

        let lemma_heap = irs.lemma_map.heap_size() as f64;
        let form_heap = irs.form_map.heap_size() as f64;
        let irs_heap = lemma_heap + form_heap;
        let lemma_heap_msg = crate::utils::human_size(lemma_heap);
        let form_heap_msg = crate::utils::human_size(form_heap);
        let irs_heap_msg = crate::utils::human_size(irs_heap);

        //         println!(
        //             "Found {n_irs} irs: {n_lemmas} lemmas, {n_forms} forms \
        // ({n_forms_inflection} inflections, {n_forms_extracted} extracted, {n_forms_alt_of} alt_of)"
        //         );

        const MB: f64 = 1024.0 * 1024.0;
        if irs_heap > 500.0 * MB {
            tracing::debug!("Found {} irs ({})", n_irs, irs_heap_msg,);
            tracing::debug!("├─ lemmas: {} ({})", n_lemmas, lemma_heap_msg,);
            tracing::debug!(
                "└─ forms : {} ({}) [infl {}, extr {}, alt {}]",
                n_forms,
                form_heap_msg,
                n_forms_inflection,
                n_forms_extracted,
                n_forms_alt_of,
            );
        }
    }

    fn write_ir(&self) -> bool {
        true
    }

    fn postprocess(&self, irs: &mut Self::I) {
        postprocess_forms(&mut irs.form_map);
    }

    fn to_yomitan(&self, langs: LangSpecs, irs: Self::I) -> Vec<LabelledYomitanEntry> {
        vec![
            LabelledYomitanEntry::new("lemma", to_yomitan_lemmas(langs.target, irs.lemma_map)),
            LabelledYomitanEntry::new("form", to_yomitan_forms(langs.source, irs.form_map)),
        ]
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct LemmaKey {
    lemma: String,
    reading: String,
    pos: Pos,
}

#[derive(Debug, Default)]
struct LemmaMap(Map<LemmaKey, Vec<LemmaInfo>>);

// We only serialize for debugging in the testsuite, so having this tmp nested is easy to write and
// has no overhead when building the dictionary without --save-temps. This way, we avoid storing
// nested structures that are less performant (both for cache locality, and number of lookups).
impl Serialize for LemmaMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut nested: Map<&str, Map<&str, Map<&str, &Vec<LemmaInfo>>>> = Map::default();

        for (key, infos) in &self.0 {
            nested
                .entry(&key.lemma)
                .or_default()
                .entry(&key.reading)
                .or_default()
                .insert(&key.pos, infos);
        }

        nested.serialize(serializer)
    }
}

impl LemmaMap {
    pub fn into_flat_iter(self) -> impl Iterator<Item = (String, String, Pos, LemmaInfo)> {
        self.0.into_iter().flat_map(|(key, infos)| {
            let lemma = key.lemma;
            let reading = key.reading;
            let pos = key.pos;

            infos
                .into_iter()
                .map(move |info| (lemma.clone(), reading.clone(), pos.clone(), info))
        })
    }

    fn len(&self) -> usize {
        self.0.values().map(Vec::len).sum()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct FormKey {
    uninflected: String,
    inflected: String,
    pos: Pos,
}

#[derive(Debug, Default)]
struct FormMap(Map<FormKey, (FormSource, Vec<String>)>);

// We only serialize for debugging in the testsuite, so having this tmp nested is easy to write and
// has no overhead when building the dictionary without --save-temps. This way, we avoid storing
// nested structures that are less performant (both for cache locality, and number of lookups).
impl Serialize for FormMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[expect(clippy::type_complexity)]
        let mut nested: Map<&str, Map<&str, Map<&str, &(FormSource, Vec<String>)>>> =
            Map::default();

        for (key, infos) in &self.0 {
            nested
                .entry(&key.uninflected)
                .or_default()
                .entry(&key.inflected)
                .or_default()
                .insert(&key.pos, infos);
        }

        nested.serialize(serializer)
    }
}

impl FormMap {
    /// Iterates over: uninflected, inflected, pos, source, tags
    fn flat_iter(&self) -> impl Iterator<Item = (&str, &str, &str, &FormSource, &Vec<String>)> {
        self.0.iter().map(|(key, (source, tags))| {
            (
                key.uninflected.as_str(),
                key.inflected.as_str(),
                key.pos.as_str(),
                source,
                tags,
            )
        })
    }

    fn into_flat_iter(
        self,
    ) -> impl Iterator<Item = (String, String, String, FormSource, Vec<String>)> {
        self.0
            .into_iter()
            .map(|(key, (source, tags))| (key.uninflected, key.inflected, key.pos, source, tags))
    }

    /// Iterates over: uninflected, inflected, pos, source, tags
    fn flat_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&str, &str, &str, &mut FormSource, &mut Vec<String>)> {
        self.0.iter_mut().map(|(key, (source, tags))| {
            (
                key.uninflected.as_str(),
                key.inflected.as_str(),
                key.pos.as_str(),
                source,
                tags,
            )
        })
    }

    fn len(&self) -> usize {
        self.flat_iter().count()
    }

    fn len_of(&self, source: FormSource) -> usize {
        self.flat_iter()
            .filter(|(_, _, _, src, _)| **src == source)
            .count()
    }

    fn len_extracted(&self) -> usize {
        self.len_of(FormSource::Extracted)
    }

    fn len_inflection(&self) -> usize {
        self.len_of(FormSource::Inflection)
    }

    fn len_alt_of(&self) -> usize {
        self.len_of(FormSource::AltOf)
    }
}

/// Enum used exclusively for debugging. This information doesn't appear on the dictionary.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum FormSource {
    /// Form extracted from `entry.forms`
    Extracted,
    /// Form added via gloss analysis ("is inflection of...")
    Inflection,
    /// Alternative forms
    AltOf,
}

// NOTE: the less we have here the better. For example, the links could be entirely moved to the
// yomitan side of things. It all depends on what we may or may not consider useful for debugging.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct LemmaInfo {
    gloss_tree: GlossTree,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<Tag>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    synonyms: Vec<Synonym>,

    #[serde(skip_serializing_if = "Option::is_none")]
    etymology_text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    head_info_text: Option<String>,

    #[serde(rename = "wlink")]
    link_wiktionary: String,

    #[serde(rename = "klink")]
    link_kaikki: String,
}

type GlossTree = Map<String, GlossInfo>;

// ... its really SenseInfo but oh well
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
struct GlossInfo {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<Tag>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    topics: Vec<Tag>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    examples: Vec<Example>,

    #[serde(skip_serializing_if = "Map::is_empty")]
    children: GlossTree,
}

/// Intermediate representation of the main dictionary.
#[derive(Debug, Default)]
pub struct Tidy {
    lemma_map: LemmaMap, // 56
    form_map: FormMap,   // 56
}

impl Tidy {
    fn len(&self) -> usize {
        self.lemma_map.len() + self.form_map.len()
    }

    // This is usually called at the end, so it could just move the arguments...
    fn insert_lemma(&mut self, lemma: &str, reading: &str, pos: &str, entry: LemmaInfo) {
        debug_assert!(!entry.gloss_tree.is_empty());

        let key = LemmaKey {
            lemma: lemma.into(),
            reading: reading.into(),
            pos: pos.into(),
        };

        match self.lemma_map.0.entry(key) {
            Entry::Vacant(e) => {
                e.insert(vec![entry]);
            }
            Entry::Occupied(mut e) => {
                e.get_mut().push(entry);
            }
        }
    }

    fn insert_form(
        &mut self,
        uninflected: &str,
        inflected: &str,
        pos: &str,
        source: FormSource,
        tags: Vec<Tag>,
    ) {
        debug_assert_ne!(uninflected, inflected);
        debug_assert!(!tags.is_empty());

        let key = FormKey {
            uninflected: uninflected.into(),
            inflected: inflected.into(),
            pos: pos.into(),
        };

        match self.form_map.0.entry(key) {
            Entry::Vacant(e) => {
                e.insert((source, tags));
            }
            Entry::Occupied(mut e) => {
                e.get_mut().1.extend(tags);
            }
        }
    }

    // NOTE: we write stuff even if irs.attribute is empty
    #[tracing::instrument(skip_all)]
    fn write(&self, pm: &PathManager) -> Result<()> {
        let opath = pm.path_lemmas();
        let file = File::create(&opath)?;
        let writer = BufWriter::new(file);

        if pm.opts.pretty {
            serde_json::to_writer_pretty(writer, &self.lemma_map)?;
        } else {
            serde_json::to_writer(writer, &self.lemma_map)?;
        }
        if !pm.opts.quiet {
            pretty_println_at_path("Wrote tidy lemmas", &opath);
        }

        let opath = pm.path_forms();
        let file = File::create(&opath)?;
        let writer = BufWriter::new(file);

        if pm.opts.pretty {
            serde_json::to_writer_pretty(writer, &self.form_map)?;
        } else {
            serde_json::to_writer(writer, &self.form_map)?;
        }
        if !pm.opts.quiet {
            pretty_println_at_path("Wrote tidy forms", &opath);
        }

        Ok(())
    }
}

fn postprocess_forms(form_map: &mut FormMap) {
    for (_, _, _, _, tags) in form_map.flat_iter_mut() {
        // Keep only unique tags and remove tags subsets
        remove_redundant_tags(tags);

        // Merge person tags
        merge_person_tags(tags);

        // Sort inner words
        for tag in tags.iter_mut() {
            let mut words: Vec<&str> = tag.split(' ').collect();
            sort_tags(&mut words);
            *tag = words.join(" ");
        }

        sort_tags_by_similar(tags);
    }
}

fn process_main(edition: Edition, source: Lang, entry: &WordEntry, irs: &mut Tidy) {
    if should_skip_entry(entry) {
        return;
    }

    process_forms(edition, source, entry, irs);

    process_alt_forms(entry, irs);

    // All of these are scuffed pages that should be using the {{ja-wagokanji}} template, i.e.
    // (correct) https://ja.wiktionary.org/wiki/減らす
    //
    // Also redirections of this type:
    // https://ja.wiktionary.org/wiki/好み#Japanese
    // このみの漢字表記。
    //
    // https://ja.wiktionary.org/wiki/諄い
    // くどいを参照。
    // match edition {
    //     Edition::Ja => {
    //         if let Some(sense) = entry.senses.first() {
    //             if let Some(gloss) = sense.glosses.first() {
    //                 if gloss.ends_with("参照") && gloss.split_whitespace().count() == 2 {
    //                     tracing::warn!(
    //                         "Japanese redirection @ {}",
    //                         link_wiktionary(edition, source, &entry.word)
    //                     );
    //                 }
    //             }
    //         }
    //     }
    //     _ => (),
    // };

    if entry.contains_no_gloss() {
        process_no_gloss(edition, entry, irs);
    } else {
        irs.insert_lemma(
            &entry.word,
            &get_reading(edition, source, entry).unwrap_or_else(|| entry.word.clone()),
            &entry.pos,
            process_entry(edition, source, entry),
        );
    }
}

/// Whether we should completely skip this entry.
///
/// The function is trivial at the moment and only relevant for the [ja-en] dict.
fn should_skip_entry(entry: &WordEntry) -> bool {
    // https://en.wiktionary.org/wiki/toraware#Japanese
    entry.pos == "romanization"
}

// Everything that mutates entry
fn preprocess_main(
    edition: Edition,
    source: Lang,
    opts: &Options,
    entry: &mut WordEntry,
    irs: &mut Tidy,
) {
    // WARN: mutates entry::senses::sense::tags
    match edition {
        Edition::En => {
            // The original fetched them from head_templates but it is better not to touch that
            // and we can do the same by looking at the tags of the canonical form.
            if let Some(cform) = entry.canonical_form() {
                let cform_tags: Vec<_> = cform.tags.clone();
                for sense in &mut entry.senses {
                    for tag in &cform_tags {
                        if tag != "canonical" && !sense.tags.contains(tag) {
                            sense.tags.push(tag.into());
                        }
                    }
                }
            }
        }
        Edition::El => {
            // Fetch gender from a matching form
            let gender_tags = ["masculine", "feminine", "neuter"];
            for form in &entry.forms {
                if form.form == entry.word {
                    for sense in &mut entry.senses {
                        for tag in &form.tags {
                            if gender_tags.contains(&tag.as_str()) && !sense.tags.contains(tag) {
                                sense.tags.push(tag.into());
                            }
                        }
                    }
                }
            }
        }
        _ => (),
    }

    // WARN: mutates entry::senses
    //
    // Deal with "no definition" glosses, cf. https://it.wiktionary.org/wiki/cartoccio#Italian
    // That is, glosses that are of no value, usually of the shape "Empty definition, add one at
    // this link etc."
    //
    // See also: https://it.wiktionary.org/wiki/Template:Nodef
    if edition == Edition::It {
        for sense in &mut entry.senses {
            sense
                .glosses
                .retain(|gloss| *gloss != "definizione mancante; se vuoi, aggiungila tu");
        }
    }

    // WARN: mutates entry::senses
    //
    // What if the current word is an inflection but *also* has an inflection table?
    // https://el.wiktionary.org/wiki/ψηφίσας
    //
    // That is, imagine participle A comes from verb B, but A is treated as an adjective, so
    // it has a declension table. If we are not careful, every word C in the table that is a form
    // of A will not appear in the dictionary!
    //
    // It does not happen in English, but bear with this fake example:
    // * C = runnings < A = running < B = run
    // then, by saying that A is just a form of B, we will remove the sense, and the entry won't be
    // added to lemmas because there are no senses at all. All the information in the declension
    // table saying C < A will yield no results. Effectively, hovering over C in yomitan will show
    // nothing. Not ideal.
    //
    // There are two choices, make C point to B, or keep A as a non-lemma. We opt for the latter,
    // checking that there are no trivial forms (C) in WordEntry. Only then we can safely delete
    // the sense.
    //
    // Note that deleting senses is a good decision overall: it reduces clutter and forces the
    // redirect. One just has to be careful about when to do it
    //
    let old_senses = std::mem::take(&mut entry.senses);
    let mut senses_without_inflections = Vec::new();
    for sense in old_senses {
        if is_inflection_sense(edition, &sense)
            && (!opts.experimental || entry.non_trivial_forms().next().is_none())
        {
            handle_inflection_sense(edition, source, entry, &sense, irs);
        } else {
            senses_without_inflections.push(sense);
        }
    }
    entry.senses = senses_without_inflections;

    // WARN: mutates entry::senses::glosses
    //
    // rg: full stop
    // https://github.com/yomidevs/yomitan/issues/2232
    // Add an empty whitespace at the end... and it works!
    if opts.experimental {
        static TRAILING_PUNCT_RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"\p{P}$").unwrap());
        for sense in &mut entry.senses {
            for gloss in &mut sense.glosses {
                if !TRAILING_PUNCT_RE.is_match(gloss) {
                    gloss.push(' ');
                }
            }
        }
    }
}

/// Add Extracted forms. That is, forms from `entry.forms`.
fn process_forms(edition: Edition, source: Lang, entry: &WordEntry, irs: &mut Tidy) {
    for form in entry.non_trivial_forms() {
        if should_break_at_finish_forms(edition, source, form) {
            break;
        }

        if should_skip_form(edition, source, form) {
            continue;
        }

        let filtered_tags = form
            .tags
            .iter()
            .map(String::as_str)
            .filter(|tag| !REDUNDANT_FORM_TAGS.contains(tag))
            .collect::<Vec<_>>()
            .join(" ");

        irs.insert_form(
            &entry.word,
            &form.form,
            &entry.pos,
            FormSource::Extracted,
            vec![filtered_tags],
        );
    }
}

// This requires knowledge of witktionary table structures, the edition language and Yomitan.
// The point being to skip unnecessary forms because either:
// - They are already included in a shorter form which is equally useful
//   Ex. if we have the fr "arrive", don't add "j'arrive"
// - (This for later) Yomitan has inflection rules that make adding the form pointless
//   Ex. "runs" for "run", since there is already a plural rule
// - They are useless
//   - forms with multiple articles (useless for a dictionary). Ex. "il/elle/on arrive"
//   - most composite verbs are in most languages
//
// Eventually it would be preferable if these were done at wiktextract level, but let's do the work
// ourselves for now
fn should_skip_form(edition: Edition, source: Lang, form: &Form) -> bool {
    match (edition, source) {
        (Edition::Fr, Lang::Fr) => {
            // Objectively better
            if ["qu’", "que ", "il/elle/on", "ils/elles", "en "]
                .iter()
                .any(|p| form.form.starts_with(p))
            {
                return true;
            }
            // Debatable, but since we contain the part participle (mangé), we most likely don't
            // need "j’avais mangé"
            if form.tags.iter().any(|tag| tag == "pluperfect") {
                return true;
            }
        }
        (Edition::En, Lang::Ja) => {
            // Skip transliterations: "hashireru", "tanoshikarō" etc.
            if is_japanese_romanization(&form.form) {
                return true;
            }
        }
        _ => (),
    }
    false
}

fn is_japanese_romanization(form: &str) -> bool {
    form.chars()
        .all(|c| c.is_ascii() || matches!(c, 'ā' | 'ī' | 'ū' | 'ē' | 'ō'))
}

// Finnish from the English edition crashes with out-of-memory.
// There are simply too many forms, so we prune the less used (possessive).
//
// https://uusikielemme.fi/finnish-grammar/possessive-suffixes-possessiivisuffiksit#one
fn should_break_at_finish_forms(edition: Edition, source: Lang, form: &Form) -> bool {
    if matches!((edition, source), (Edition::En, Lang::Fi)) {
        // HACK: 1. For tables that parse the title
        // https://kaikki.org/dictionary/Finnish/meaning/p/p%C3%A4/p%C3%A4%C3%A4.html
        if form.form == "See the possessive forms below." {
            return true;
        }
        // HACK: 2. For tables that don't parse the title
        // https://kaikki.org/dictionary/Finnish/meaning/i/is/iso.html
        // https://github.com/tatuylonen/wiktextract/issues/1565
        if form.form == "Rare. Only used with substantive adjectives." {
            return true;
        }
    }
    false
}

/// Add `AltOf` forms. That is, alternative forms.
fn process_alt_forms(entry: &WordEntry, irs: &mut Tidy) {
    let base_tags = vec!["alt-of".to_string()];

    for alt_form in &entry.alt_of {
        irs.insert_form(
            &entry.word,
            &alt_form.word,
            &entry.pos,
            FormSource::AltOf,
            base_tags.clone(),
        );
    }

    for sense in &entry.senses {
        let mut sense_tags = sense.tags.clone();
        sense_tags.extend(base_tags.clone());

        for alt_form in &sense.alt_of {
            irs.insert_form(
                &entry.word,
                &alt_form.word,
                &entry.pos,
                FormSource::AltOf,
                sense_tags.clone(),
            );
        }
    }
}

/// Process "no-gloss" word entries for alternative ways of adding lemmas/forms.
#[expect(clippy::single_match)]
fn process_no_gloss(edition: Edition, entry: &WordEntry, irs: &mut Tidy) {
    match edition {
        // Unfortunately we are in the same A from B, B from C situation discussed in
        // preprocess_entry. There is no easy solution for adding the lemma back because at
        // this point the gloss has been deleted. Maybe reconsider the original approach of
        // deleting glosses, and mark them somehow as "inflection-only".
        //
        // At any rate, this will still add useful redirections.
        Edition::El => {
            if entry.is_participle()
                && let Some(form_of) = entry.form_of.first()
            {
                irs.insert_form(
                    &form_of.word,
                    &entry.word,
                    &entry.pos,
                    FormSource::Inflection,
                    vec![format!("redirected from {}", entry.word)],
                );
            }
        }
        _ => (),
    }
}

// There are potentially more than one, but yomitan doesn't really support it
pub fn get_reading(edition: Edition, source: Lang, entry: &WordEntry) -> Option<String> {
    match (edition, source) {
        (Edition::En, Lang::Ja) => get_japanese_reading(entry),
        (Edition::En, Lang::Fa) => entry.romanization_form().map(|f| f.form.clone()),
        (Edition::Ja, _) => entry.transliteration_form().map(|f| f.form.clone()),
        (Edition::En | Edition::Zh, Lang::Zh) => entry.pinyin().map(String::from),
        _ => get_canonical_word(source, entry),
    }
}

/// The canonical word may contain extra diacritics.
///
/// For most languages, this is equal to word, but for, let's say, Latin, there may be a
/// difference (cf. <https://en.wiktionary.org/wiki/fama>, where `entry.word` is fama, but
/// this will return fāma).
fn get_canonical_word(source: Lang, entry: &WordEntry) -> Option<String> {
    match source {
        Lang::La | Lang::Ru | Lang::Grc | Lang::Ar | Lang::Fa => {
            entry.canonical_form().map(|f| f.form.to_string())
        }
        _ => None,
    }
}

// TODO: work with this to return a &str
//
// Does not support multiple readings
fn get_japanese_reading(entry: &WordEntry) -> Option<String> {
    // The original parses head_templates directly (which probably deserves a PR to
    // wiktextract), although imo pronunciation templates should have been better.
    // There is no pronunciation template info in en-wiktextract, and while I think that
    // information ends up in sounds, it is not always reliable. For example:
    // https://en.wiktionary.org/wiki/お腹が空いた
    // has a pronunciation template:
    // {{ja-pron|おなか が すいた}}
    // but no "other" sounds, which is where pronunciations are usually stored.

    // Ideally we would just do this:
    // for sound in &entry.sounds {
    //     if !sound.other.is_empty() {
    //         return &sound.other;
    //     }
    // }

    // I really don't want to touch templates so instead, replace the ruby
    if let Some(cform) = entry.canonical_form()
        && !cform.ruby.is_empty()
    {
        // https://github.com/tatuylonen/wiktextract/issues/1484
        // let mut cform_lemma = cform.form.clone();
        // if cform_lemma != entry.word {
        //     warn!(
        //         "Canonical form: '{cform_lemma}' != word: '{}'\n{}\n{}\n\n",
        //         entry.word,
        //         link_wiktionary(args, &entry.word),
        //         link_kaikki(args, &entry.word),
        //     );
        // } else {
        //     warn!(
        //         "Equal for word: '{}'\n{}\n{}\n\n",
        //         entry.word,
        //         link_wiktionary(args, &entry.word),
        //         link_kaikki(args, &entry.word),
        //     );
        // }

        // This should be cform.form, but it's not parsed properly:
        // https://github.com/tatuylonen/wiktextract/issues/1484
        let mut cform_lemma = entry.word.clone();
        let mut cursor = 0;
        for (base, reading) in &cform.ruby {
            if let Some(pos) = cform_lemma[cursor..].find(base) {
                let start = cursor + pos;
                let end = start + base.len();
                cform_lemma.replace_range(start..end, reading);
                cursor = start + reading.len();
            } else {
                tracing::warn!("Kanji '{}' not found in '{}'", base, cform_lemma);
                return None;
            }
        }
        return Some(cform_lemma);
    }

    None
}

fn process_entry(edition: Edition, source: Lang, entry: &WordEntry) -> LemmaInfo {
    LemmaInfo {
        gloss_tree: get_gloss_tree(entry),
        tags: entry.tags.clone(),
        synonyms: entry.synonyms.iter().take(3).cloned().collect(),
        etymology_text: entry
            .etymology_texts()
            .map(|etymology_text| etymology_text.join("\n")),
        head_info_text: get_head_info(&entry.head_templates)
            .map(|head_info_text| head_info_text.join("\n")),
        link_wiktionary: link_wiktionary(edition, source, &entry.word),
        link_kaikki: link_kaikki(edition, source, &entry.word),
    }
}

static PARENS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\(.+?\)").unwrap());

// rg: getheadinfo
// For consistency, it's better if this function shares return type with .etymology_texts()
fn get_head_info(head_templates: &[HeadTemplate]) -> Option<Vec<&str>> {
    let result: Vec<_> = head_templates
        .iter()
        .filter_map(|head_template| {
            if PARENS_RE.is_match(&head_template.expansion) {
                Some(head_template.expansion.as_str())
            } else {
                None
            }
        })
        .collect();

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn get_gloss_tree(entry: &WordEntry) -> GlossTree {
    let mut gloss_tree = GlossTree::default();

    for sense in &entry.senses {
        // rg: examplefiltering
        // bunch of example filtering: skip

        let mut filtered_examples: Vec<_> = sense
            .examples
            .iter()
            .filter(|ex| !ex.text.is_empty() && ex.text.chars().count() <= 120) // equal to JS length
            .cloned()
            .collect();
        // Place examples with translations first
        filtered_examples.sort_by_key(|ex| ex.translation.is_empty());

        insert_glosses(
            &mut gloss_tree,
            &sense.glosses,
            &sense.tags,
            &sense.topics,
            &filtered_examples,
        );
    }

    gloss_tree
}

/// Recursive helper to deal with nested glosses
fn insert_glosses(
    gloss_tree: &mut GlossTree,
    glosses: &[String],
    tags: &[Tag],
    topics: &[Tag],
    examples: &[Example],
) {
    let Some(head) = glosses.first() else {
        return;
    };

    let tail = &glosses[1..];

    // get or insert node with only tags at this level
    let node = gloss_tree.entry(head.clone()).or_insert_with(|| GlossInfo {
        tags: tags.to_vec(),
        topics: topics.to_vec(),
        ..Default::default()
    });

    // intersect tags if node already exists
    if !node.tags.is_empty() {
        node.tags = tags
            .iter()
            .filter(|&t| node.tags.contains(t))
            .cloned()
            .collect();
    }

    // assign examples to the last level
    if tail.is_empty() {
        node.examples = examples.to_vec();
        return;
    }

    insert_glosses(&mut node.children, tail, tags, topics, examples);
}

static DE_INFLECTION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(.*)des (?:Verbs|Adjektivs|Substantivs|Demonstrativpronomens|Possessivpronomens|Pronomens) (.*)$"
    ).unwrap()
});

// rg: isinflectiongloss
fn is_inflection_sense(edition: Edition, sense: &Sense) -> bool {
    match edition {
        Edition::De => sense
            .glosses
            .iter()
            .any(|gloss| DE_INFLECTION_RE.is_match(gloss)),
        Edition::El => {
            !sense.form_of.is_empty() && sense.glosses.iter().any(|gloss| gloss.contains("του"))
        }
        Edition::En => {
            sense.glosses.iter().any(|gloss| {
                if gloss.contains("inflection of") {
                    return true;
                }

                for form in &sense.form_of {
                    if form.word.is_empty() {
                        continue;
                    }
                    // We are looking for "... of {word}$" or "... of {word} (text)$"
                    //
                    // Cf.
                    // ... imperative of iki
                    // ... perfective of возни́кнуть (vozníknutʹ)
                    // But no
                    // ... agent noun of fahren; driver (person)
                    let subs = format!("of {}", form.word);
                    if gloss.ends_with(&subs)
                        || (gloss.contains(&format!("{subs} (")) && gloss.ends_with(')'))
                    {
                        return true;
                    }
                }

                false
            })
        }
        Edition::Fr | Edition::It | Edition::Ja => {
            // Cf. https://kaikki.org/itwiktionary/Italiano/meaning/s/sc/scorrevo.html
            //
            // This is the most generic way of dealing with inflection, but assumes that the
            // edition is parsed by kaikki with "form_of" at sense level. It could be extended to
            // other languages.
            //
            // Note: because of how "etymologies" (read, definitions) are processed by Kaikki, we
            // are guaranteed that if we find an inflection, all the glosses are variations of that
            // inflection.
            // This means that a word like "cura", that both in Spanish and Italian is both a verb
            // and a noun, will not have the glosses intermingled: we will only find glosses for
            // the verb in the verb "etymology", and similarly for the noun.
            // Therefore, we can safely assume that, if we have a form_of, every gloss is an
            // explanation about the inflection, and that the only condition we really care about,
            // is the presence of a "form_of".
            //
            // Cf. https://it.wiktionary.org/wiki/cura#Italiano (fixed recently)
            // Cf. https://it.wiktionary.org/wiki/ceni#Italiano
            sense.form_of.len() == 1
        }
        _ => false,
    }
}

const TAGS_RETAINED_EL: [&str; 9] = [
    "masculine",
    "feminine",
    "neuter",
    "singular",
    "plural",
    "nominative",
    "accusative",
    "genitive",
    "vocative",
];

fn handle_inflection_sense(
    edition: Edition,
    source: Lang,
    entry: &WordEntry,
    sense: &Sense,
    irs: &mut Tidy,
) {
    debug_assert!(!sense.glosses.is_empty()); // we checked @ is_inflection_sense

    match edition {
        Edition::De => {
            if let Some(caps) = DE_INFLECTION_RE.captures(&sense.glosses[0])
                && let (Some(inflection_tags), Some(uninflected)) = (caps.get(1), caps.get(2))
            {
                let inflection_tags = inflection_tags.as_str().trim();

                if !inflection_tags.is_empty() {
                    irs.insert_form(
                        uninflected.as_str(),
                        &entry.word,
                        &entry.pos,
                        FormSource::Inflection,
                        vec![inflection_tags.to_string()],
                    );
                }
            }
        }
        Edition::El => {
            let allowed_tags: Vec<_> = sense
                .tags
                .iter()
                .filter(|tag| TAGS_RETAINED_EL.contains(&tag.as_str()))
                .map(String::from)
                .collect();
            let inflection_tags: Vec<_> = if allowed_tags.is_empty() {
                // very rare
                vec![format!("redirected from {}", entry.word)]
            } else {
                allowed_tags
            };
            for form in &sense.form_of {
                irs.insert_form(
                    &form.word,
                    &entry.word,
                    &entry.pos,
                    FormSource::Inflection,
                    // Unfortunate clone. Most sense.form_of only contain one form...
                    inflection_tags.clone(),
                );
            }
        }
        Edition::En => handle_inflection_sense_en(source, entry, sense, irs),
        Edition::Fr | Edition::It | Edition::Ja => {
            // One could use sense::glosses as tags, and while they carry important information
            // they are obscenely verbose.
            match sense.form_of.as_slice() {
                [form_of] => {
                    // Some redirections may not contain the "form-of" tag at sense level!
                    // https://kaikki.org/jawiktionary/日本語/meaning/名/名前/名前.html
                    debug_assert!(
                        sense.tags.iter().any(|tag| *tag == "form-of")
                            || entry.tags.iter().any(|tag| *tag == "form-of")
                    );
                    let allowed_tags: Vec<_> = sense
                        .tags
                        .iter()
                        .filter(|tag| *tag != "form-of")
                        .map(String::from)
                        .collect();
                    let inflection_tags: Vec<_> = if allowed_tags.is_empty() {
                        vec![format!("redirected from {}", entry.word)]
                    } else {
                        allowed_tags
                    };
                    // We need to normalize, for instance, for the la-ja, so that appellantur redirects to
                    // appellare and not to appellāre (since only appellare appears as lemma)
                    let norm_form_of_word = normalize_orthography(source, &form_of.word);
                    irs.insert_form(
                        &norm_form_of_word,
                        &entry.word,
                        &entry.pos,
                        FormSource::Inflection,
                        inflection_tags,
                    );
                }
                // SAFETY: we checked that only one form_of is present.
                _ => unreachable!(),
            }
        }
        _ => unreachable!("Unhandled lang that implements is_inflection_sense"),
    }
}

fn handle_inflection_sense_en(source: Lang, entry: &WordEntry, sense: &Sense, irs: &mut Tidy) {
    let uninflected = match sense.form_of.as_slice() {
        [alt_form] => &alt_form.word,
        _ => return,
    };

    // Not sure if this is better (cf. ru-en) over entry.word but it is what was done in
    // the original, so lets not change that for the moment.
    let inflected = get_canonical_word(source, entry).unwrap_or_else(|| entry.word.clone());

    if inflected == *uninflected {
        return;
    }

    let mut inflections = Set::default();
    let of_uninflected = format!("of {uninflected}");
    for gloss in &sense.glosses {
        let cleaned = gloss
            .replace("inflection of ", "")
            .replace(&of_uninflected, "")
            .replace(uninflected, "")
            .replace(':', "");

        let inflection = PARENS_RE.replace_all(&cleaned, "").trim().to_string();

        if !inflection.is_empty() {
            inflections.insert(inflection);
        }
    }

    for inflection in inflections {
        irs.insert_form(
            uninflected,
            &inflected,
            &entry.pos,
            FormSource::Inflection,
            vec![inflection],
        );
    }
}

fn normalize_orthography(source: Lang, word: &str) -> String {
    const ARABIC_DIACRITICS: [char; 16] = [
        '\u{0618}', '\u{0619}', '\u{061A}', '\u{064B}', '\u{064C}', '\u{064D}', '\u{064E}',
        '\u{064F}', '\u{0650}', '\u{0651}', '\u{0652}', '\u{0653}', '\u{0654}', '\u{0655}',
        '\u{0656}', '\u{0670}',
    ];
    match source {
        Lang::Ar | Lang::Fa => word
            .chars()
            .filter(|c| !ARABIC_DIACRITICS.contains(c))
            .collect(),
        Lang::La | Lang::Ang | Lang::Sga | Lang::Grc | Lang::Ro | Lang::Id => word
            .nfd()
            .filter(|c| !('\u{0300}'..='\u{036F}').contains(c))
            .nfc()
            .collect(),
        Lang::Tl => word
            .nfd()
            .filter(|c| !('\u{0300}'..='\u{036F}').contains(c) && *c != '-' && *c != '\'')
            .nfc()
            .collect(),
        Lang::Sh => {
            let mut last_base: Option<char> = None;
            let filtered = word.nfd().filter(|&c| {
                if ('\u{0300}'..='\u{036F}').contains(&c) {
                    !matches!(
                        last_base,
                        Some('a' | 'e' | 'i' | 'o' | 'u' | 'r' | 'A' | 'E' | 'I' | 'O' | 'U' | 'R')
                    )
                } else {
                    last_base = Some(c);
                    true
                }
            });
            filtered.nfc().collect()
        }
        Lang::Uk | Lang::Ru => word.replace('\u{0301}', ""),
        _ => word.to_string(),
    }
}

#[tracing::instrument(skip_all, level = "trace")]
fn to_yomitan_lemmas(target: Lang, lemma_map: LemmaMap) -> Vec<YomitanEntry> {
    lemma_map
        .into_flat_iter()
        .map(move |(lemma, reading, pos, info)| to_yomitan_lemma(target, lemma, reading, pos, info))
        .collect()
}

// TODO: consume info
fn to_yomitan_lemma(
    target: Lang,
    lemma: String,
    reading: String,
    pos: String,
    info: LemmaInfo,
) -> YomitanEntry {
    let short_pos = find_short_pos_or_default(&pos);

    let yomitan_reading = if reading == lemma {
        "".to_string()
    } else {
        reading
    };

    let common_short_tags_found = get_found_tags(&pos, &info);
    let definition_tags = common_short_tags_found
        .iter()
        .map(|short_tag| match localize_tag(target, short_tag) {
            Some((short, _)) => short,
            None => short_tag,
        })
        .collect::<Vec<_>>()
        .join(" ");

    let mut detailed_definition_content = Node::new_array();

    if info.etymology_text.is_some() || info.head_info_text.is_some() {
        detailed_definition_content.push(structured_preamble(
            target,
            info.etymology_text,
            info.head_info_text,
        ));
    }

    detailed_definition_content.push(structured_glosses(
        target,
        info.gloss_tree,
        &common_short_tags_found,
    ));

    if let Some(synonyms_node) = structured_synonyms(target, &info.synonyms) {
        detailed_definition_content.push(synonyms_node);
    }

    detailed_definition_content.push(structured_backlink(info.link_wiktionary, info.link_kaikki));

    YomitanEntry::TermBank(TermBank(
        lemma,
        yomitan_reading,
        definition_tags,
        get_rule_identifier(short_pos),
        vec![DetailedDefinition::structured(detailed_definition_content)],
    ))
}

/// Extracts and normalizes tags associated with a lemma.
///
/// This function collects tags from three sources:
/// 1. The provided part-of-speech (`pos`) - always included first
/// 2. Top-level tags from `LemmaInfo` (`info.tags`) - usu. non-En edition
/// 3. Tags common to all senses in `info.gloss_tree` - usu. En edition
///
/// For sense-level tags, only those present in *every* gloss are kept
/// (set intersection across all gloss entries).
fn get_found_tags(pos: &Pos, info: &LemmaInfo) -> Vec<Tag> {
    let common_tags_iter = info
        .gloss_tree
        .values()
        .map(|g| Set::from_iter(g.tags.iter().cloned()))
        .reduce(|acc, set| acc.intersection(&set).cloned().collect::<Set<Tag>>())
        .unwrap() // a non-empty gloss_tree has at least one gloss
        .into_iter();

    std::iter::once(pos.to_string())
        .chain(info.tags.clone()) // top level tags (the non-En preferred way)
        .chain(common_tags_iter)
        .filter_map(|tag| match find_tag_in_bank(&tag) {
            Some(tag_info) => Some(tag_info.short_tag),
            None => {
                // log skipped tags
                // if !["alt-of", "alternative", "form-of"].contains(&tag.as_str()) {
                //     tracing::debug!("{} @ {}", tag, info.link_wiktionary);
                // }
                None
            }
        })
        .collect()
}

// There could be multiple identifiers, but let's start with one.
//
// This function is trivial at the moment, but could be worked on to validate identifiers,
// add multiple identifiers, merge tags into more useful identifiers (verb: v, transitive: t > vt),
// remove unused identifiers etc.
fn get_rule_identifier(short_pos: &str) -> String {
    short_pos.to_string()
}

fn build_details_entry(ty: &str, ty_loc: &str, content: String) -> Node {
    wrap(
        NTag::Details,
        &format!("details-entry-{ty}"),
        Node::Array(vec![
            wrap(NTag::Summary, "summary-entry", Node::Text(ty_loc.into())),
            wrap(NTag::Div, &format!("{ty}-content"), Node::Text(content)),
        ]),
    )
}

fn structured_preamble(
    target: Lang,
    etymology_text: Option<String>,
    head_info_text: Option<String>,
) -> Node {
    let mut preamble_content = Node::new_array();
    if let Some(head_info_text) = head_info_text {
        let ty_loc = localize_grammar_string(target);
        preamble_content.push(build_details_entry("Grammar", ty_loc, head_info_text));
    }
    if let Some(etymology_text) = etymology_text {
        let ty_loc = localize_etymology_string(target);
        preamble_content.push(build_details_entry("Etymology", ty_loc, etymology_text));
    }

    wrap(
        NTag::Div,
        "",
        wrap(NTag::Div, "preamble", preamble_content).into_array_node(),
    )
}

fn structured_backlink(wlink: String, klink: String) -> Node {
    wrap(
        NTag::Div,
        "backlink",
        Node::Array(vec![
            Node::Backlink(BacklinkContent::new(wlink, BacklinkContentKind::Wiktionary)),
            Node::Text(" | ".into()), // JMdict uses this separator
            Node::Backlink(BacklinkContent::new(klink, BacklinkContentKind::Kaikki)),
        ]),
    )
}

fn structured_glosses(
    target: Lang,
    gloss_tree: GlossTree,
    common_short_tags_found: &[Tag],
) -> Node {
    wrap(
        NTag::Ol,
        "glosses",
        Node::Array(
            gloss_tree
                .into_iter()
                .map(|gloss_pair| {
                    wrap(
                        NTag::Li,
                        "",
                        Node::Array(structured_glosses_go(
                            target,
                            &GlossTree::from_iter([gloss_pair]),
                            common_short_tags_found,
                            0,
                        )),
                    )
                })
                .collect(),
        ),
    )
}

// Recursive helper ~ should return Node for consistency
fn structured_glosses_go(
    target: Lang,
    gloss_tree: &GlossTree,
    common_short_tags_found: &[Tag],
    level: usize,
) -> Vec<Node> {
    let html_tag = if level == 0 { NTag::Div } else { NTag::Li };
    let mut nested = Vec::new();

    for (gloss, gloss_info) in gloss_tree {
        // Tags/topics that are not common to all glosses (i.e. specific to this gloss)
        let minimal_tags: Vec<_> = gloss_info
            .tags
            .iter()
            .chain(gloss_info.topics.iter())
            .filter(|&tag| !common_short_tags_found.contains(tag))
            .cloned()
            .collect();

        let mut level_content = Node::new_array();

        if let Some(structured_tags) =
            structured_tags(target, &minimal_tags, common_short_tags_found)
        {
            level_content.push(structured_tags);
        }

        level_content.push(Node::Text(gloss.into()));

        if !gloss_info.examples.is_empty() {
            level_content.push(structured_examples(target, &gloss_info.examples));
        }

        nested.push(wrap(html_tag, "", level_content));

        if gloss_info.children.is_empty() {
            continue;
        }

        // We dont want tags from the parent appearing again in the children
        let mut new_common_short_tags_found = minimal_tags;
        new_common_short_tags_found.extend_from_slice(common_short_tags_found);

        nested.push(wrap(
            NTag::Ul,
            "",
            Node::Array(structured_glosses_go(
                target,
                &gloss_info.children,
                &new_common_short_tags_found,
                level + 1,
            )),
        ));
    }

    nested
}

/// Structure inner tags.
///
/// We sort them ourselves since yomitan only sorts top-level tags.
///
/// cf [`crate::models::yomitan::TagInformation`]
fn structured_tags(target: Lang, tags: &[Tag], common_short_tags_found: &[Tag]) -> Option<Node> {
    let mut tag_infos: Vec<_> = tags
        .iter()
        .filter_map(|tag| {
            let tag_info = find_tag_in_bank(tag)?;
            if common_short_tags_found.contains(&tag_info.short_tag) {
                None
            } else {
                Some(tag_info)
            }
        })
        .collect();

    tag_infos.sort_unstable_by_key(|t| t.sort_order);

    let structured_tags_content: Vec<_> = tag_infos
        .into_iter()
        .map(|tag_info| {
            let (short_tag, long_tag) = match localize_tag(target, &tag_info.short_tag) {
                Some((short, long)) => (short.to_string(), long.to_string()),
                None => {
                    // if tag_info.category != "topic" && tag_info.category != "variety" {
                    //     tracing::debug!(
                    //         "Tag not localized to {target}: {} ({})",
                    //         tag_info.short_tag,
                    //         tag_info.long_tag
                    //     );
                    // }
                    (tag_info.short_tag, tag_info.long_tag)
                }
            };
            GenericNode {
                tag: NTag::Span,
                title: Some(long_tag),
                data: Some(NodeData::from_iter([
                    ("content", "tag"),
                    ("category", &tag_info.category),
                ])),
                content: Node::Text(short_tag),
            }
            .into_node()
        })
        .collect();

    if structured_tags_content.is_empty() {
        None
    } else {
        Some(wrap(
            NTag::Div,
            "tags",
            Node::Array(structured_tags_content),
        ))
    }
}

fn structured_examples(target: Lang, examples: &[Example]) -> Node {
    debug_assert!(!examples.is_empty());

    let localized_label = wrap(
        NTag::Summary,
        "summary-entry",
        Node::Text(localize_examples_string(target, examples.len())),
    );

    wrap(
        NTag::Details,
        "details-entry-examples",
        Node::Array(
            std::iter::once(localized_label)
                .chain(examples.iter().map(structured_example))
                .collect(),
        ),
    )
}

// TODO: change a-b-c into a more descriptive name: text/translation/ref
fn structured_example(example: &Example) -> Node {
    let mut structured_example_content = wrap(
        NTag::Div,
        "example-sentence-a",
        structured_example_text(&example.text, &example.bold_text_offsets),
    )
    .into_array_node();

    if !example.translation.is_empty() {
        structured_example_content.push(wrap(
            NTag::Div,
            "example-sentence-b",
            structured_example_text(&example.translation, &example.bold_translation_offsets),
        ));
    }

    if !example.reference.is_empty() {
        let reference = example
            .reference
            .strip_suffix(':')
            .unwrap_or(&example.reference)
            .to_string();
        structured_example_content.push(wrap(
            NTag::Div,
            "example-sentence-c",
            Node::Text(reference),
        ));
    }

    wrap(
        NTag::Div,
        "extra-info",
        wrap(NTag::Div, "example-sentence", structured_example_content),
    )
}

/// Wraps in NTag::Span bold ranges if there are any.
///
/// Note that wiktextract only extracts bold offsets for Examples.
fn structured_example_text(text: &str, offsets: &[Offset]) -> Node {
    if offsets.is_empty() {
        return Node::Text(text.to_string());
    }

    let chars: Vec<_> = text.chars().collect();
    let upto = chars.len();

    let offsets = sanitize_offsets(offsets, upto);
    if offsets.is_empty() {
        return Node::Text(text.to_string());
    }

    let mut content = Node::new_array();
    let mut last = 0;

    for (start, end) in offsets {
        // Push what comes before the bold offset
        if last < start {
            content.push(Node::Text(chars[last..start].iter().collect()));
        }

        // Push the bold offset
        content.push(wrap(
            NTag::Span,
            "bold-text",
            Node::Text(chars[start..end].iter().collect()),
        ));

        last = end;
    }

    if last < chars.len() {
        content.push(Node::Text(chars[last..].iter().collect()));
    }

    content
}

fn structured_synonyms(target: Lang, synonyms: &[Synonym]) -> Option<Node> {
    if synonyms.is_empty() {
        return None;
    }

    Some(wrap(
        NTag::Div,
        "synonyms",
        Node::Array(vec![
            wrap(
                NTag::Div,
                "synonyms-label",
                Node::Text(localize_synonyms_string(target).into()),
            ),
            wrap(
                NTag::Ul,
                "synonyms-list",
                Node::Array(
                    synonyms
                        .iter()
                        .map(|syn| wrap(NTag::Li, "synonym-item", Node::Text(syn.word.clone())))
                        .collect(),
                ),
            ),
        ]),
    ))
}

/// Returns only valid, non-overlapping offsets within bounds of `upto`.
///
/// We ASSUME that offsets are sorted by start. That is, given [O1, O2, ...] with
/// O1 = (a, b) and O2 = (c, d), we assume a <= c etc.
///
/// Merges offsets with a non-trivial intersection. That is, if O1 and O2 overlap, we merge into
/// O3 = (a, max(b, d)).
fn sanitize_offsets(offsets: &[Offset], upto: usize) -> Vec<Offset> {
    let mut sanitized: Vec<Offset> = Vec::new();
    for &(start, end) in offsets {
        debug_assert!(start < end);
        if end > upto {
            // Out of bound offset: skip
            continue;
        }
        match sanitized.last_mut() {
            Some(prev) if start < prev.1 => prev.1 = prev.1.max(end),
            _ => sanitized.push((start, end)),
        }
    }
    sanitized
}

#[tracing::instrument(skip_all, level = "trace")]
fn to_yomitan_forms(source: Lang, form_map: FormMap) -> Vec<YomitanEntry> {
    form_map
        .into_flat_iter()
        .map(move |(uninflected, inflected, pos, _, tags)| {
            // There needs to be DetailedDefinition per tag because yomitan reads
            // multiple tags in a single Inflection as a causal inflection chain.
            let deinflection_definitions: Vec<_> = tags
                .iter()
                .map(|tag| {
                    DetailedDefinition::Inflection((uninflected.to_string(), vec![tag.to_string()]))
                })
                .collect();

            let normalized_inflected = normalize_orthography(source, &inflected);
            let reading = if normalized_inflected == inflected {
                String::new()
            } else {
                inflected
            };

            let short_pos = find_short_pos_or_default(&pos);

            YomitanEntry::TermBankSimplified(TermBankSimplified(
                normalized_inflected,
                reading,
                get_rule_identifier(short_pos),
                deinflection_definitions,
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::sanitize_offsets;

    #[test]
    fn offset_base() {
        assert_eq!(sanitize_offsets(&[], 10), vec![]);
        assert_eq!(
            sanitize_offsets(&[(0, 2), (3, 5)], 10),
            vec![(0, 2), (3, 5)]
        );
    }

    #[test]
    fn offset_out_of_bounds() {
        assert_eq!(sanitize_offsets(&[(0, 2), (3, 20)], 10), vec![(0, 2)]);
    }

    #[test]
    fn offset_overlap() {
        // 1. b == c, treated as non-overlapping
        assert_eq!(
            sanitize_offsets(&[(0, 2), (2, 4)], 10),
            vec![(0, 2), (2, 4)]
        );

        // 2. a == c, keep larger
        assert_eq!(sanitize_offsets(&[(10, 12), (10, 13)], 20), vec![(10, 13)]);

        // 3. b == d, keep larger
        assert_eq!(sanitize_offsets(&[(10, 12), (11, 12)], 20), vec![(10, 12)]);

        // 4. Inner is smaller - discard it
        assert_eq!(sanitize_offsets(&[(0, 10), (2, 5)], 20), vec![(0, 10)]);

        // 5. a < c < b < d - merge into (a, d)
        assert_eq!(sanitize_offsets(&[(0, 5), (3, 8)], 20), vec![(0, 8)]);
    }
}
