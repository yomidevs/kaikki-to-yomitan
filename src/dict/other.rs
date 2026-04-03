use crate::{
    Map, Set,
    cli::{GlossaryArgs, GlossaryExtendedArgs, IpaArgs, IpaMergedArgs, LangSpecs},
    dict::{Dictionary, LabelledYomitanEntry, Langs, get_reading},
    lang::{Edition, Lang},
    models::{
        kaikki::WordEntry,
        yomitan::{
            DetailedDefinition, Ipa, NTag, Node, PhoneticTranscription, TermBank, TermBankMeta,
            TermPhoneticTranscription, YomitanEntry, wrap,
        },
    },
    tags::{find_short_pos_or_default, find_tag_in_bank, tags_localization::localize_tag},
};

#[derive(Debug, Clone, Copy)]
pub struct DGlossary;

#[derive(Debug, Clone, Copy)]
pub struct DGlossaryExtended;

#[derive(Debug, Clone, Copy)]
pub struct DIpa;

#[derive(Debug, Clone, Copy)]
pub struct DIpaMerged;

impl Dictionary for DGlossary {
    type A = GlossaryArgs;
    type I = Vec<YomitanEntry>;

    fn process(&self, langs: Langs, entry: &WordEntry, irs: &mut Self::I) {
        process_glossary(langs.edition, langs.target, entry, irs);
    }

    fn to_yomitan(&self, _: LangSpecs, irs: Self::I) -> Vec<LabelledYomitanEntry> {
        vec![LabelledYomitanEntry::new("term", irs)]
    }
}

impl Dictionary for DGlossaryExtended {
    type A = GlossaryExtendedArgs;
    type I = IGlossaryExtended;

    fn supports_probe(&self) -> bool {
        false
    }

    fn process(&self, langs: Langs, entry: &WordEntry, irs: &mut Self::I) {
        process_glossary_extended(langs.edition, langs.source, langs.target, entry, irs);
    }

    // TODO: change type "I" to not have to merge lemmas here
    fn postprocess(&self, irs: &mut Self::I) {
        let mut map = Map::default();

        for (lemma, pos, edition, translations) in irs.drain(..) {
            map.entry(lemma)
                .or_insert_with(|| (pos, edition, Set::default()))
                .2
                .extend(translations);
        }

        irs.extend(map.into_iter().map(|(lemma, (pos, edition, set))| {
            (lemma, pos, edition, set.into_iter().collect::<Vec<_>>())
        }));
    }

    fn to_yomitan(&self, langs: LangSpecs, irs: Self::I) -> Vec<LabelledYomitanEntry> {
        vec![LabelledYomitanEntry::new(
            "term",
            to_yomitan_glossary_extended(langs.target, irs),
        )]
    }
}

impl Dictionary for DIpa {
    type A = IpaArgs;
    type I = IIpa;

    fn process(&self, langs: Langs, entry: &WordEntry, irs: &mut Self::I) {
        process_ipa(langs.edition, langs.source, langs.target, entry, irs);
    }

    fn to_yomitan(&self, _: LangSpecs, irs: Self::I) -> Vec<LabelledYomitanEntry> {
        vec![LabelledYomitanEntry::new("term", to_yomitan_ipa(irs))]
    }
}

impl Dictionary for DIpaMerged {
    type A = IpaMergedArgs;
    type I = IIpa;

    fn process(&self, langs: Langs, entry: &WordEntry, irs: &mut Self::I) {
        process_ipa(langs.edition, langs.source, langs.target, entry, irs);
    }

    fn postprocess(&self, irs: &mut Self::I) {
        // Sorting is not needed ~ just for visibility
        irs.sort_unstable_keys();
    }

    fn to_yomitan(&self, _: LangSpecs, tidy: Self::I) -> Vec<LabelledYomitanEntry> {
        vec![LabelledYomitanEntry::new("term", to_yomitan_ipa(tidy))]
    }
}

fn process_glossary(source: Edition, target: Lang, entry: &WordEntry, irs: &mut Vec<YomitanEntry>) {
    let mut translations: Map<&str, Vec<String>> = Map::default();
    for translation in entry.non_trivial_translations() {
        if translation.lang_code == target.iso() {
            translations
                .entry(&translation.sense)
                .or_default()
                .push(translation.word.clone());
        }
    }

    if translations.is_empty() {
        return;
    }

    let mut definitions = Vec::new();
    for (sense, translations) in translations {
        if sense.is_empty() {
            definitions.extend(translations.into_iter().map(DetailedDefinition::Text));
            continue;
        }

        definitions.push(DetailedDefinition::structured(wrap(
            NTag::Div,
            "",
            Node::Array(vec![
                wrap(NTag::Span, "", Node::Text(sense.to_string())),
                wrap(
                    NTag::Ul,
                    "",
                    Node::Array(
                        translations
                            .into_iter()
                            .map(|translation| wrap(NTag::Li, "", Node::Text(translation)))
                            .collect(),
                    ),
                ),
            ]),
        )));
    }

    let reading = get_reading(source, target, entry).unwrap_or_else(|| entry.word.clone());
    let short_pos = find_short_pos_or_default(&entry.pos);
    let loc_short_pos = match localize_tag(target, short_pos) {
        Some((short, _)) => short,
        None => short_pos,
    };

    irs.push(YomitanEntry::TermBank(TermBank(
        entry.word.clone(),
        reading,
        loc_short_pos.to_string(),
        short_pos.to_string(),
        definitions,
    )));
}

/// (lemma, pos, edition, translations)
type IGlossaryExtended = Vec<(String, String, Edition, Vec<String>)>;

fn process_glossary_extended(
    edition: Edition,
    source: Lang,
    target: Lang,
    entry: &WordEntry,
    irs: &mut IGlossaryExtended,
) {
    let mut translations: Map<&str, (Vec<&str>, Vec<&str>)> = Map::default();

    for translation in entry.non_trivial_translations() {
        if translation.lang_code == target.iso() {
            translations
                .entry(&translation.sense)
                .or_default()
                .0
                .push(&translation.word);
        }

        if translation.lang_code == source.iso() {
            translations
                .entry(&translation.sense)
                .or_default()
                .1
                .push(&translation.word);
        }
    }

    // We only keep translations with matches in both languages (source and target)
    translations.retain(|_, (targets, sources)| !targets.is_empty() && !sources.is_empty());

    if translations.is_empty() {
        return;
    }

    // A "semi" cartesian product. See the test below.
    irs.extend(translations.iter().flat_map(|(_, (targets, sources))| {
        sources.iter().map(|lemma| {
            (
                (*lemma).to_string(),
                entry.pos.clone(),
                edition,
                targets.iter().map(|def| (*def).to_string()).collect(),
            )
        })
    }));
}

fn to_yomitan_glossary_extended(target: Lang, irs: IGlossaryExtended) -> Vec<YomitanEntry> {
    irs.into_iter()
        .map(|(lemma, pos, _, translations)| {
            let short_pos = find_short_pos_or_default(&pos);
            let loc_short_pos = match localize_tag(target, short_pos) {
                Some((short, _)) => short,
                None => short_pos,
            };

            YomitanEntry::TermBank(TermBank(
                lemma,
                String::new(),
                loc_short_pos.to_string(),
                short_pos.to_string(),
                translations
                    .into_iter()
                    .map(DetailedDefinition::Text)
                    .collect(),
            ))
        })
        .collect()
}

/// Normalize IPA notation: convert `\X\`, bare `X` to `/X/`, preserve `[X]`
///
/// `\X\` is used in the French edition for some reason, and `X` is just laziness.
///
/// ## Difference between `/X/` and `[X]`
///
/// - `/ ... /` is for phonemic transcription
/// - `[ ... ]` is for phonetic transcription
///
/// For example, the English words pin and spin become:
/// - `/pɪn/` and `/spɪn/`
/// - `[pʰɪn]` and `[spɪn]`.
///
/// See <https://en.wiktionary.org/wiki/Wiktionary:International_Phonetic_Alphabet>
fn normalize_ipa(text: &str) -> String {
    let fst = text.chars().next();
    let lst = text.chars().last();

    match (fst, lst) {
        (Some('['), Some(']')) => text.to_string(),
        (Some('/'), Some('/')) => text.to_string(),
        (Some('\\'), Some('\\')) if text.len() > 1 => format!("/{}/", &text[1..text.len() - 1]),
        _ => format!("/{}/", text),
    }
}

/// Extract inner IPA content without delimiters for comparison
fn ipa_inner(text: &str) -> &str {
    let fst = text.chars().next();
    let lst = text.chars().last();

    match (fst, lst) {
        (Some('['), Some(']')) => &text[1..text.len() - 1],
        (Some('/'), Some('/')) | (Some('\\'), Some('\\')) if text.len() > 1 => {
            &text[1..text.len() - 1]
        }
        _ => text,
    }
}

fn is_phonetic(text: &str) -> bool {
    text.starts_with('[') && text.ends_with(']')
}

// Grouping by ipa is done at process_ipa
fn get_ipas(entry: &WordEntry) -> Vec<Ipa> {
    entry
        .sounds
        .iter()
        .filter_map(|sound| {
            if sound.ipa.is_empty() {
                return None;
            }
            let mut tags = sound.tags.clone();
            if !sound.note.is_empty() {
                tags.push(sound.note.clone());
            }
            Some(Ipa {
                ipa: normalize_ipa(&sound.ipa),
                tags,
            })
        })
        .collect()
}

/// ((lemma, reading), transcription)
type IIpa = Map<(String, String), Vec<Ipa>>;

fn process_ipa(edition: Edition, source: Lang, target: Lang, entry: &WordEntry, irs: &mut IIpa) {
    let mut ipas = get_ipas(entry);

    if ipas.is_empty() {
        return;
    }

    // This replacing with the short tag will still show the long version on hover,
    // even though they are not really top-level tags (in the sense of the main dict)
    for ipa in &mut ipas {
        for tag in &mut ipa.tags {
            // tracing::warn!("tag {tag} @ {} (ed: {edition})", &entry.word);
            if let Some(tag_info) = find_tag_in_bank(tag) {
                // tracing::warn!("found tag {tag_info:?}");
                *tag = match localize_tag(target, &tag_info.short_tag) {
                    Some((short, _)) => {
                        // tracing::warn!("localized short tag {short:?}");
                        short.to_string()
                    }
                    None => (*tag_info.short_tag).to_string(),
                }
            }
        }
    }

    let reading = get_reading(edition, source, entry).unwrap_or_else(|| entry.word.clone());
    let existing = irs.entry((entry.word.clone(), reading)).or_default();
    for ipa in ipas {
        let inner = ipa_inner(&ipa.ipa);
        if let Some(existing_ipa) = existing.iter_mut().find(|e| ipa_inner(&e.ipa) == inner) {
            // Prefer phonetic [X] over phonemic /X/ (more specific)
            if is_phonetic(&ipa.ipa) {
                existing_ipa.ipa = ipa.ipa;
            }
            for tag in ipa.tags {
                if !existing_ipa.tags.contains(&tag) {
                    existing_ipa.tags.push(tag);
                }
            }
        } else {
            existing.push(ipa);
        }
    }
}

fn to_yomitan_ipa(irs: IIpa) -> Vec<YomitanEntry> {
    irs.into_iter()
        .map(|((lemma, reading), transcriptions)| {
            // NOTE: sorting is tricky because the order in Wiktionary may matter, with the first
            // result being the most relevant (not always, remains to be tested).
            // This is relevant for X-Y-ipa dicts, for merged X-ipa dicts the order is completely
            // random, as in it depends on edition iteration order, and therefore sorting is much
            // more justified.
            //
            // transcriptions.sort_unstable_by(|a, b| ipa_inner(&a.ipa).cmp(ipa_inner(&b.ipa)));

            YomitanEntry::TermBankMeta(TermBankMeta::TermPhoneticTranscription(
                TermPhoneticTranscription(
                    lemma,
                    PhoneticTranscription {
                        reading,
                        transcriptions,
                    },
                ),
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::models::kaikki::{Sound, Translation};

    impl Translation {
        fn new(lang_code: &str, sense: &str, word: &str) -> Self {
            Self {
                lang_code: lang_code.into(),
                sense: sense.into(),
                word: word.into(),
            }
        }
    }

    // cf. https://en.wiktionary.org/wiki/Gibraltar
    // {
    //     English (sense):       "British overseas territory"
    //     Albanian (sh):         ["Gjibraltar", "Gjibraltari"]
    //     Greek, Ancient (grc):  ["Ἡράκλειαι στῆλαι", "Κάλπη"]
    // }
    //
    //     source                            target (what we search)
    // >>> ["Gjibraltar", "Gjibraltari"]  <> "Ἡράκλειαι στῆλαι"
    // >>> ["Gjibraltar", "Gjibraltari"]  <> "Κάλπη"
    #[test]
    fn process_glossary_extended_basic() {
        let dict = DGlossaryExtended;
        let langs = Langs::new(Edition::En, Lang::Grc, Lang::Sh);
        let mut entry = WordEntry::default();
        entry.pos = "noun".to_string();
        entry.translations = vec![
            Translation::new("grc", "British overseas territory", "Ἡράκλειαι στῆλαι"),
            Translation::new("grc", "British overseas territory", "Ἡράκλειαι στῆλαι"),
            Translation::new("grc", "British overseas territory", "Κάλπη"),
            Translation::new("sh", "British overseas territory", "Gibraltar"),
            Translation::new("sh", "British overseas territory", "Gjibraltari"),
            Translation::new("sh", "Different sense", "Foo"),
        ];

        let mut irs = Vec::new();
        dict.process(langs, &entry, &mut irs);

        // Empty translations should not change anything
        let entry = WordEntry::default();
        dict.process(langs, &entry, &mut irs);

        assert_eq!(irs.len(), 3);

        let (lemma1, pos, _, defs1) = &irs[0];
        let (lemma2, _, _, defs2) = &irs[1];
        let (lemma3, _, _, defs3) = &irs[2];

        assert_eq!(pos, "noun");
        assert_eq!(lemma1, "Ἡράκλειαι στῆλαι");
        assert_eq!(lemma2, "Ἡράκλειαι στῆλαι");
        assert_eq!(lemma3, "Κάλπη");

        let expected = vec!["Gibraltar".to_string(), "Gjibraltari".to_string()];
        assert_eq!(defs1, &expected);
        assert_eq!(defs2, &expected);
        assert_eq!(defs3, &expected);

        dict.postprocess(&mut irs);
        assert_eq!(irs.len(), 2);

        let yomitan_entries = to_yomitan_glossary_extended(Lang::Grc, irs);
        assert_eq!(yomitan_entries.len(), 2);
        match yomitan_entries.first().unwrap() {
            YomitanEntry::TermBank(term_bank) => {
                // Should use the short pos here (noun > n)
                assert_eq!(term_bank.2, "n")
            }
            _ => panic!(), // We know that this dict only produces TermBank
        }
    }

    #[test]
    fn process_glossary_extended_pos_localization() {
        let dict = DGlossaryExtended;
        // Japanese as target, source irrelevant
        let langs = Langs::new(Edition::En, Lang::Ja, Lang::En);
        let mut entry = WordEntry::default();
        entry.pos = "noun".to_string();
        entry.translations = vec![
            Translation::new("ja", "some sense", "日本語"),
            Translation::new("en", "some sense", "english"),
        ];

        let mut irs = IGlossaryExtended::new();
        dict.process(langs, &entry, &mut irs);

        assert_eq!(irs.len(), 1);
        let (_, pos, _, _) = &irs[0];

        assert_eq!(pos, "noun");

        let yomitan_entries = to_yomitan_glossary_extended(Lang::Ja, irs);
        match &yomitan_entries[0] {
            YomitanEntry::TermBank(term_bank) => assert_eq!(term_bank.2, "名"),
            _ => panic!(), // We know that this dict only produces TermBank
        }
    }

    impl Sound {
        fn new(ipa: &str) -> Self {
            Self {
                ipa: ipa.into(),
                ..Default::default()
            }
        }

        fn with_tag(ipa: &str, tag: &str) -> Self {
            Self {
                ipa: ipa.into(),
                tags: vec![tag.into()],
                ..Default::default()
            }
        }
    }

    #[test]
    fn process_ipa_merged_basic() {
        let dict = DIpaMerged;
        let langs = Langs::new(Edition::En, Lang::Grc, Lang::Sh);
        let mut entry = WordEntry::default();
        entry.sounds = vec![
            Sound::new("/ipa1/"), // '/' phonemic gets replaced by '[]' phonetic...
            Sound::new("[ipa1]"),
            Sound::new("/ipa1/"), // ...independently of order
            Sound::new("[ipa2]"),
        ];

        let mut irs = IIpa::default();
        dict.process(langs, &entry, &mut irs);

        assert_eq!(irs.len(), 1);

        let transcriptions = irs.values().next().unwrap();
        assert_eq!(transcriptions.len(), 2);
        assert_eq!(&transcriptions[0].ipa, "[ipa1]");
        assert_eq!(&transcriptions[1].ipa, "[ipa2]");
    }

    #[test]
    fn process_ipa_tag() {
        let dict = DIpa;
        let langs = Langs::new(Edition::En, Lang::La, Lang::La); // irrelevant
        let mut entry = WordEntry::default();
        entry.sounds = vec![
            Sound::with_tag("[ipa1]", "tag1"),
            Sound::with_tag("[ipa2]", "modern Italianate Ecclesiastical"),
        ];

        let mut irs = IIpa::default();
        dict.process(langs, &entry, &mut irs);

        assert_eq!(irs.len(), 1);

        let transcriptions = irs.values().next().unwrap();
        assert_eq!(transcriptions.len(), 2);

        assert_eq!(&transcriptions[0].ipa, "[ipa1]");
        assert_eq!(&transcriptions[1].ipa, "[ipa2]");

        // Check that tags are properly simplified
        assert_eq!(&transcriptions[0].tags[0], "tag1");
        assert_eq!(&transcriptions[1].tags[0], "⛪");
    }

    #[test]
    fn process_ipa_merged_tag_merge() {
        let dict = DIpaMerged;
        let (source, target) = (Lang::La, Lang::La);

        let edition = Edition::En;
        let langs = Langs::new(edition, source, target);
        let mut entry = WordEntry::default();
        entry.sounds = vec![Sound::with_tag("ipa1", "tag1")];

        let mut irs = IIpa::default();
        dict.process(langs, &entry, &mut irs);

        let edition = Edition::De;
        let langs = Langs::new(edition, source, target);
        let mut entry = WordEntry::default();
        // same ipa, different tag (coming from another edition)
        entry.sounds = vec![Sound::with_tag("ipa1", "tag2")];
        dict.process(langs, &entry, &mut irs);

        let transcriptions = irs.values().next().unwrap();
        assert_eq!(transcriptions.len(), 1);
        // Both tags should be present after merging
        assert!(transcriptions[0].tags.contains(&"tag1".to_string()));
        assert!(transcriptions[0].tags.contains(&"tag2".to_string()));
    }

    #[test]
    fn process_ipa_merged_postprocess_order() {
        let dict = DIpaMerged;
        let (source, target) = (Lang::La, Lang::La);
        let langs = Langs::new(Edition::En, source, target);

        let mut irs = IIpa::default();

        let mut entry = WordEntry::default();
        entry.word = "zebra".to_string();
        entry.sounds = vec![Sound::new("ipa1")];
        dict.process(langs, &entry, &mut irs);

        let mut entry = WordEntry::default();
        entry.word = "apple".to_string();
        entry.sounds = vec![Sound::new("ipa2")];
        dict.process(langs, &entry, &mut irs);

        dict.postprocess(&mut irs);

        let keys: Vec<&String> = irs.keys().map(|(word, _)| word).collect();
        assert_eq!(keys, vec!["apple", "zebra"]);
    }
}
