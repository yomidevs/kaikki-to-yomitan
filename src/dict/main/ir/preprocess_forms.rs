use crate::{
    lang::{Edition, Lang},
    models::kaikki::WordEntry,
};

pub fn preprocess_forms(edition: Edition, source: Lang, entry: &mut WordEntry) {
    match (edition, source, entry.pos.as_str()) {
        (Edition::De, Lang::De, "verb") => preprocess_forms_de_de(entry),
        (Edition::En, Lang::Ga, _) => preprocess_forms_ga_en(entry),
        (Edition::Es, Lang::Es, "verb") => preprocess_forms_es_es(entry),
        (Edition::Fr, Lang::Fr, "verb") => preprocess_forms_fr_fr(entry),
        (Edition::It, Lang::It, "verb") => preprocess_forms_it_it(entry),
        (Edition::Pt, Lang::Pt, "verb") => preprocess_forms_pt_pt(entry),
        _ => (),
    }
}

fn strip_prefixes(entry: &mut WordEntry, prefixes: &[&str]) {
    debug_assert!(prefixes.iter().all(|pron| pron.ends_with(' ')));

    for form in &mut entry.forms {
        for &prefix in prefixes {
            if let Some(stripped) = form.form.strip_prefix(prefix) {
                form.form = stripped.to_string();
                break;
            }
        }
    }
}

// In German, verb forms come with personal pronouns which makes for poor results (worse
// search, deduplication, dictionary bloat etc.)
// This should probably be fixed at wiktextract at some point.
// Here we trim personal pronouns prefixes: "ich ", "du ", "er/sie/es " etc.
//
// See: https://kaikki.org/dewiktionary/Deutsch/meaning/a/au/ausmachen.html
fn preprocess_forms_de_de(entry: &mut WordEntry) {
    // 1. Trim personal pronouns from verb forms (this information is already in tags)
    const PRONOUNS: &[&str] = &["ich ", "du ", "er/sie/es ", "wir ", "ihr ", "sie "];
    strip_prefixes(entry, PRONOUNS);

    // Another possible simplification:
    //
    // Reflexive forms, i.e. "wir meldeten uns an"
    // Just skip: there should be an active table anyway, and they become redundant.
    //
    // Here they have the reflexive tag:
    // https://de.wiktionary.org/wiki/Flexion:anmelden
    // ...but in general they don't need to, if there is only the reflexive option:
    // https://de.wiktionary.org/wiki/Flexion:fortscheren

    // 2. Remove auxiliary verb constructions
    //
    // Working with tags is better than doing string replacement, because in that case we may
    // stumble into edge cases like "anhaben", where we don't want to confuse the auxiliary verb
    // with the actual verb.
    // See: https://www.verblisten.de/listen/verben/anfangsbuchstabe/ueberblick.html?i=haben
    entry.forms.retain(|form| {
        let is_compound = form.tags.iter().any(|tag| {
            matches!(
                tag.as_str(),
                "perfect"
                    | "pluperfect"
                    | "future-i"
                    | "future-ii"
                    | "processual-passive"
                    | "statal-passive"
            )
        });

        !is_compound && !form.form.ends_with(['…', '!'])
            // "Partizip II des Verbs sehen, nur unmittelbar nach einem Infinitiv" etc.
            && !form.form.contains(',')
    });

    // The above tag strategy "requires"* us to clean the "extended" forms.
    // *I'm not entirely sure this is needed, specially because the form obtained is an adjective
    // that most likely happens in some other page, but since it gives the same result as the
    // (previous) string replacement, we keep it as it is.
    for form in &mut entry.forms {
        if let Some(stripped) = form.form.strip_prefix("zu ") {
            form.form = stripped.to_string();
        }
        if form.tags.iter().any(|tag| tag == "extended")
            && let Some(stripped) = form.form.strip_suffix(" zu haben")
        {
            form.form = stripped.to_string();
        }
    }
}

fn preprocess_forms_ga_en(entry: &mut WordEntry) {
    // https://en.wiktionary.org/wiki/crodh#Irish
    const PREFIXES: &[&str] = &["a ", "an ", "na ", "leis an ", "don ", "leis na "];
    strip_prefixes(entry, PREFIXES);
}

fn preprocess_forms_es_es(entry: &mut WordEntry) {
    // The auxiliary haber is wrongly parsed as a form
    #[rustfmt::skip]
    const HABER_AUX: &[&str] = &[
        "he", "has", "ha", "hemos", "habéis", "han",
        "había", "habías", "había", "habíamos", "habíais", "habían",
        "habré", "habrás", "habrá", "habremos", "habréis", "habrán",
        "habría", "habrías", "habría", "habríamos", "habríais", "habrían",
        "haya", "hayas", "haya", "hayamos", "hayáis", "hayan",
        "hubiera", "hubieras", "hubiera", "hubiéramos", "hubierais", "hubieran",
        "hubiese", "hubieses", "hubiese", "hubiésemos", "hubieseis", "hubiesen",
    ];
    if entry.word != "haber" {
        entry.forms.retain(|form| {
            let is_auxiliary_form = HABER_AUX.contains(&form.form.as_str());

            !is_auxiliary_form
        });
    }
}

// This function is made based on preprocess_forms_de_de. See that function for more details.
fn preprocess_forms_fr_fr(entry: &mut WordEntry) {
    const PRONOUNS: &[&str] = &[
        "je ",
        "j' ",
        "tu ",
        "il/elle/on ",
        "nous ",
        "vous ",
        "ils/elles ",
    ];
    strip_prefixes(entry, PRONOUNS);

    entry.forms.retain(|form| {
        let is_compound = form
            .tags
            .iter()
            .any(|tag| matches!(tag.as_str(), "perfect" | "pluperfect" | "anterior"));
        let is_past_conditional = ["past", "conditional"]
            .iter()
            .all(|ctag| form.tags.iter().any(|tag| tag == ctag));
        let is_past_imperative = ["past", "imperative"]
            .iter()
            .all(|ctag| form.tags.iter().any(|tag| tag == ctag));

        !is_compound && !is_past_conditional && !is_past_imperative
    });
}

fn preprocess_forms_it_it(entry: &mut WordEntry) {
    const AVERE_AUX: &[&str] = &[
        "avrei ",
        "avresti ",
        "avrebbe ",
        "avremmo ",
        "avreste ",
        "avrebbero ",
        // These are actually perfect tense, but wiktionary doesn't parse them as such
        "abbia ",
        "abbiamo ",
        "abbiate ",
        "abbiano ",
        // Adding this here for simplicity: "non mangiare"
        "non ",
    ];
    strip_prefixes(entry, AVERE_AUX);

    entry.forms.retain(|form| {
        let is_compound = form
            .tags
            .iter()
            .any(|tag| matches!(tag.as_str(), "perfect" | "pluperfect" | "historic"));

        // mangiarsi (coniugazione)
        !is_compound && !form.form.ends_with(')')
    });
}

fn preprocess_forms_pt_pt(entry: &mut WordEntry) {
    strip_prefixes(entry, &["não "]);
}
