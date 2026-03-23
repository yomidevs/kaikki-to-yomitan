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
    type I = Vec<YomitanEntry>;
    type A = GlossaryArgs;

    fn keep_if(&self, source: Lang, entry: &WordEntry) -> bool {
        entry.lang_code == source.iso()
    }

    fn process(&self, langs: Langs, entry: &WordEntry, irs: &mut Self::I) {
        process_glossary(langs.edition, langs.target, entry, irs);
    }

    fn to_yomitan(&self, _: LangSpecs, irs: Self::I) -> Vec<LabelledYomitanEntry> {
        vec![LabelledYomitanEntry::new("term", irs)]
    }
}

impl Dictionary for DGlossaryExtended {
    type I = Vec<IGlossaryExtended>;
    type A = GlossaryExtendedArgs;

    fn keep_if(&self, _: Lang, _: &WordEntry) -> bool {
        true
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

    fn to_yomitan(&self, _: LangSpecs, irs: Self::I) -> Vec<LabelledYomitanEntry> {
        vec![LabelledYomitanEntry::new(
            "term",
            to_yomitan_glossary_extended(irs),
        )]
    }
}

impl Dictionary for DIpa {
    type I = Vec<IIpa>;
    type A = IpaArgs;

    fn keep_if(&self, source: Lang, entry: &WordEntry) -> bool {
        entry.lang_code == source.iso()
    }

    fn supports_probe(&self) -> bool {
        true
    }

    fn process(&self, langs: Langs, entry: &WordEntry, irs: &mut Self::I) {
        process_ipa(langs.edition, langs.source, entry, irs);
    }

    fn to_yomitan(&self, _: LangSpecs, irs: Self::I) -> Vec<LabelledYomitanEntry> {
        vec![LabelledYomitanEntry::new("term", to_yomitan_ipa(irs))]
    }
}

impl Dictionary for DIpaMerged {
    type I = Vec<IIpa>;
    type A = IpaMergedArgs;

    fn keep_if(&self, source: Lang, entry: &WordEntry) -> bool {
        entry.lang_code == source.iso()
    }

    fn process(&self, langs: Langs, entry: &WordEntry, irs: &mut Self::I) {
        process_ipa(langs.edition, langs.source, entry, irs);
    }

    fn postprocess(&self, irs: &mut Self::I) {
        // Keep only unique entries
        *irs = Set::from_iter(irs.drain(..)).into_iter().collect();
        // Sorting is not needed ~ just for visibility
        irs.sort_by(|a, b| a.0.cmp(&b.0));
    }

    fn to_yomitan(&self, _: LangSpecs, tidy: Self::I) -> Vec<LabelledYomitanEntry> {
        vec![LabelledYomitanEntry::new("term", to_yomitan_ipa(tidy))]
    }
}

// rg: process translations processtranslations
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
    let loc_short_pos = match localize_tag(target, &short_pos) {
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

type IGlossaryExtended = (String, String, Edition, Vec<String>);

fn process_glossary_extended(
    edition: Edition,
    source: Lang,
    target: Lang,
    entry: &WordEntry,
    irs: &mut Vec<IGlossaryExtended>,
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

    let short_pos = find_short_pos_or_default(&entry.pos);

    // A "semi" cartesian product. See the test below.
    irs.extend(translations.iter().flat_map(|(_, (targets, sources))| {
        sources.iter().map(|lemma| {
            (
                (*lemma).to_string(),
                short_pos.to_string(),
                edition,
                targets.iter().map(|def| (*def).to_string()).collect(),
            )
        })
    }));
}

fn to_yomitan_glossary_extended(irs: Vec<IGlossaryExtended>) -> Vec<YomitanEntry> {
    irs.into_iter()
        .map(|(lemma, found_pos, _, translations)| {
            YomitanEntry::TermBank(TermBank(
                lemma,
                String::new(),
                found_pos.clone(),
                found_pos,
                translations
                    .into_iter()
                    .map(DetailedDefinition::Text)
                    .collect(),
            ))
        })
        .collect()
}

// default version getphonetictranscription
pub fn get_ipas(entry: &WordEntry) -> Vec<Ipa> {
    let ipas_iter = entry.sounds.iter().filter_map(|sound| {
        if sound.ipa.is_empty() {
            return None;
        }
        let ipa = sound.ipa.clone();
        let mut tags = sound.tags.clone();
        if !sound.note.is_empty() {
            tags.push(sound.note.clone());
        }
        Some(Ipa { ipa, tags })
    });

    // rg: saveIpaResult - Group by ipa
    let mut ipas_grouped: Vec<Ipa> = Vec::new();
    for ipa in ipas_iter {
        if let Some(existing) = ipas_grouped.iter_mut().find(|e| e.ipa == ipa.ipa) {
            for tag in ipa.tags {
                if !existing.tags.contains(&tag) {
                    existing.tags.push(tag);
                }
            }
        } else {
            ipas_grouped.push(ipa);
        }
    }

    ipas_grouped
}

type IIpa = (String, PhoneticTranscription);

fn process_ipa(edition: Edition, source: Lang, entry: &WordEntry, irs: &mut Vec<IIpa>) {
    let mut ipas = get_ipas(entry);

    if ipas.is_empty() {
        return;
    }

    // This replacing with the short tag will still show the long version on hover.
    for ipa in &mut ipas {
        for tag in &mut ipa.tags {
            if let Some(tag_info) = find_tag_in_bank(tag) {
                *tag = (*tag_info.short_tag).to_string();
            }
        }
    }

    let phonetic_transcription = PhoneticTranscription {
        reading: get_reading(edition, source, entry).unwrap_or_else(|| entry.word.clone()),
        transcriptions: ipas,
    };

    irs.push((entry.word.clone(), phonetic_transcription));
}

fn to_yomitan_ipa(irs: Vec<IIpa>) -> Vec<YomitanEntry> {
    irs.into_iter()
        .map(|(lemma, transcription)| {
            YomitanEntry::TermBankMeta(TermBankMeta::TermPhoneticTranscription(
                TermPhoneticTranscription(lemma, "ipa".to_string(), transcription),
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

        let (lemma1, _, _, defs1) = &irs[0];
        let (lemma2, _, _, defs2) = &irs[1];
        let (lemma3, _, _, defs3) = &irs[2];

        assert_eq!(lemma1, "Ἡράκλειαι στῆλαι");
        assert_eq!(lemma2, "Ἡράκλειαι στῆλαι");
        assert_eq!(lemma3, "Κάλπη");

        let expected = vec!["Gibraltar".to_string(), "Gjibraltari".to_string()];
        assert_eq!(defs1, &expected);
        assert_eq!(defs2, &expected);
        assert_eq!(defs3, &expected);

        dict.postprocess(&mut irs);
        assert_eq!(irs.len(), 2);
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
        entry.sounds = vec![Sound::new("ipa1"), Sound::new("ipa1"), Sound::new("ipa2")];

        let mut irs = Vec::new();
        dict.process(langs, &entry, &mut irs);

        assert_eq!(irs.len(), 1);

        let transcriptions = &irs[0].1.transcriptions;
        assert_eq!(transcriptions.len(), 2);

        assert_eq!(&transcriptions[0].ipa, "ipa1");
        assert_eq!(&transcriptions[1].ipa, "ipa2");

        dict.postprocess(&mut irs);
        assert_eq!(irs[0].1.transcriptions.len(), 2);
    }

    #[test]
    fn process_ipa_tag() {
        let dict = DIpa;
        let langs = Langs::new(Edition::En, Lang::La, Lang::La); // irrelevant
        let mut entry = WordEntry::default();
        entry.sounds = vec![
            Sound::with_tag("ipa1", "tag1"),
            Sound::with_tag("ipa2", "modern Italianate Ecclesiastical"),
        ];

        let mut irs = Vec::new();
        dict.process(langs, &entry, &mut irs);

        assert_eq!(irs.len(), 1);

        let transcriptions = &irs[0].1.transcriptions;
        assert_eq!(transcriptions.len(), 2);

        assert_eq!(&transcriptions[0].ipa, "ipa1");
        assert_eq!(&transcriptions[1].ipa, "ipa2");

        // Check that tags are properly simplified
        assert_eq!(&transcriptions[0].tags[0], "tag1");
        assert_eq!(&transcriptions[1].tags[0], "⛪");

        dict.postprocess(&mut irs);
        assert_eq!(irs[0].1.transcriptions.len(), 2);
    }
}
