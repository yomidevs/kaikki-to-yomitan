pub mod tags_constants;
pub mod tags_localization;

use std::cmp::Ordering;

use indexmap::IndexMap;
use tags_constants::{POSES, TAG_BANK, TAG_ORDER};

use crate::lang::Lang;
use crate::models::kaikki::Tag;
use crate::models::yomitan::TagInformation;
use crate::tags::tags_localization::{has_locale, localize_tag};

/// Tags that are blacklisted if they happen at *some* expanded form @ tidy
pub const BLACKLISTED_FORM_TAGS: [&str; 14] = [
    "inflection-template",
    "table-tags",
    "canonical",
    "class",
    "error-unknown-tag",
    "error-unrecognized-form",
    "includes-article",
    "obsolete",
    "archaic",
    "used-in-the-form",
    "romanization",
    "dated",
    "auxiliary",
    // multiword-construction was in REDUNDANT_TAGS in the original.
    // Yet it only seems to give noise for the fr-en edition (@ prendre):
    // * Form: 'present indicative of avoir + past participle' ???
    // * Tags: ["indicative", "multiword-construction", "perfect", "present"]
    //
    // It also removes valid german forms that are nonetheless most useless:
    // * werde gepflogen haben (for pflegen)
    // (note that gepflogen is already added)
    // This was considered ok. To revisit if it is more intrusive in other languages.
    "multiword-construction",
];
/// Tags that are blacklisted if they happen at *every* expanded form @ tidy
pub const IDENTITY_FORM_TAGS: [&str; 3] = ["nominative", "singular", "infinitive"];
/// Tags that we just remove from forms @ tidy
pub const REDUNDANT_FORM_TAGS: [&str; 1] = ["combined-form"];

/// Sort tags by their position in the tag_order.json file.
///
/// Expects (but does not check) tags WITHOUT spaces.
pub fn sort_tags(tags: &mut [&str]) {
    debug_assert!(tags.iter().all(|tag| !tag.contains(' ')));

    tags.sort_by(|a, b| {
        let index_a = TAG_ORDER.iter().position(|&x| x == *a);
        let index_b = TAG_ORDER.iter().position(|&x| x == *b);

        match (index_a, index_b) {
            (Some(i), Some(j)) => i.cmp(&j),   // both found → compare positions
            (Some(_), None) => Ordering::Less, // found beats not-found
            (None, Some(_)) => Ordering::Greater,
            // This seems better but it's different from the original
            // (None, None) => a.cmp(b),        // neither found → alphabetical fallback
            (None, None) => Ordering::Equal, // neither found → do nothing
        }
    });
}

/// Sort tags by word-by-word lexicographical similarity, grouping tags that
/// share the same leading words (shorter prefix-only tags sort first).
///
/// Expects (but does not check) tags WITH spaces.
pub fn sort_tags_by_similar(tags: &mut [Tag]) {
    tags.sort_by(|a, b| {
        let mut a_iter = a.split(' ');
        let mut b_iter = b.split(' ');

        loop {
            match (a_iter.next(), b_iter.next()) {
                (Some(a_word), Some(b_word)) => match a_word.cmp(b_word) {
                    Ordering::Equal => continue,
                    non_eq => return non_eq,
                },
                (Some(_), None) => return Ordering::Greater,
                (None, Some(_)) => return Ordering::Less,
                (None, None) => return Ordering::Equal,
            }
        }
    });
}

/// Remove tag1 if there is a tag2 such that tag1 <= tag2.
///
/// Expects (but does not check) tags WITH spaces.
///
/// Examples:
/// * ["a b", "b a"] -> ["b a"]
/// * ["s no ne", "ne s no", "ne", "no"] -> ["ne s no"]
pub fn remove_redundant_tags(tags: &mut Vec<Tag>) {
    let mut keep = vec![true; tags.len()];

    for i in 0..tags.len() {
        if !keep[i] {
            continue;
        }
        for j in (i + 1)..tags.len() {
            if !keep[j] {
                continue;
            }

            if tags_are_subset(&tags[i], &tags[j]) {
                keep[i] = false;
                break;
            } else if tags_are_subset(&tags[j], &tags[i]) {
                keep[j] = false;
            }
        }
    }

    let mut idx = 0;
    tags.retain(|_| {
        let k = keep[idx];
        idx += 1;
        k
    });
}

/// Check if all words in `a` are present in `b`, f.e. "foo bar" is subset of "bar foo baz".
fn tags_are_subset(a: &str, b: &str) -> bool {
    a.split(' ')
        .all(|a_word| b.split(' ').any(|b_word| b_word == a_word))
}

const PERSON_TAGS: [&str; 3] = ["first-person", "second-person", "third-person"];

fn person_sort(tags: &mut [&str]) {
    tags.sort_by_key(|x| PERSON_TAGS.iter().position(|p| p == x).unwrap_or(999));
}

/// Merge similar tags if the only difference is the person-tags.
///
/// F.e.
/// in:  ['first-person singular', 'third-person singular']
/// out: ['singular first/third-person ']
///
/// Note that this does not preserve logical tag order, and should be called before `sort_tag`.
pub fn merge_person_tags(tags: &mut Vec<Tag>) {
    let contains_person = tags
        .iter()
        .any(|tag| PERSON_TAGS.iter().any(|p| tag.contains(p)));

    if !contains_person {
        return;
    }

    // Leave tags with same capacity since we are going to repopulate it
    let mut old_tags = Vec::with_capacity(tags.capacity());
    std::mem::swap(&mut old_tags, tags);

    let mut grouped: IndexMap<Vec<&str>, Vec<&str>> = IndexMap::new();

    for tag in &old_tags {
        let (person_tags, other_tags): (Vec<_>, Vec<_>) =
            tag.split(' ').partition(|t| PERSON_TAGS.contains(t));

        match person_tags.as_slice() {
            [person] => grouped.entry(other_tags).or_default().push(person),
            _ => tags.push(tag.to_string()),
        }
    }

    for (other_tags, mut person_matches) in grouped {
        person_sort(&mut person_matches);

        // [first-person, third-person] > first/third-person
        let merged_person_tag = person_matches
            .iter()
            // SAFETY: PERSON_TAGS contains pmatch so it always ends in -person
            .map(|pmatch| pmatch.strip_suffix("-person").unwrap())
            .collect::<Vec<_>>()
            .join("/")
            + "-person";

        let tag = other_tags
            .into_iter()
            .chain(std::iter::once(merged_person_tag.as_ref()))
            .collect::<Vec<_>>()
            .join(" ");

        tags.push(tag);
    }
}

// Note that while target is an Edition for the main dictionary, it can be any Lang
// for the glossary dictionary, which also uses tags.
//
/// Return a Vec<TagInformation> from `TAG_BANK` (`tag_bank_terms.json`).
pub fn get_tag_bank_as_tag_info(target: Lang) -> Vec<TagInformation> {
    if has_locale(target) {
        TAG_BANK
            .iter()
            .map(
                |&(short_tag, category, sort_order, long_tag_aliases, popularity_score)| {
                    let (short_tag_loc, long_tag_loc) = match localize_tag(target, short_tag) {
                        Some((short, long)) => (short, long),
                        // This short_tag hasn't been localized yet, use the English version
                        None => (short_tag, long_tag_aliases[0]),
                    };

                    TagInformation {
                        short_tag: short_tag_loc.to_string(),
                        category: category.to_string(),
                        sort_order,
                        long_tag: long_tag_loc.to_string(),
                        popularity_score,
                    }
                },
            )
            .collect()
    } else {
        TAG_BANK.iter().map(TagInformation::new).collect()
    }
}

/// Find the tag in `TAG_BANK` (`tag_bank_terms.json`) and return the `TagInformation` if any.
pub fn find_tag_in_bank(tag: &str) -> Option<TagInformation> {
    TAG_BANK.iter().find_map(|entry| {
        if entry.3.contains(&tag) {
            Some(TagInformation::new(entry))
        } else {
            None
        }
    })
}

/// Find the short form in POSES (`tag_bank_terms.json` with category "partOfSpeech").
fn find_short_pos(pos: &str) -> Option<&'static str> {
    POSES
        .iter()
        .find_map(|(long, short)| if *long == pos { Some(*short) } else { None })
}

/// Find the short form in POSES (`tag_bank_terms.json` with category "partOfSpeech"), or default
/// to input.
pub fn find_short_pos_or_default(pos: &str) -> &str {
    find_short_pos(pos).unwrap_or(pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_string_vec(str_vec: &[&str]) -> Vec<String> {
        str_vec.iter().map(|s| (*s).to_string()).collect()
    }

    fn to_str_vec<'a>(str_vec: &[&'a str]) -> Vec<&'a str> {
        str_vec.to_vec()
    }

    // This imitates the original. Can be removed if sort_tags logic changes.
    #[test]
    fn sort_tags_base() {
        let tag_not_found = "__sentinel";
        assert!(!TAG_ORDER.contains(&tag_not_found));
        let mut received = to_str_vec(&[tag_not_found, "Gheg"]);
        let expected = to_string_vec(&[tag_not_found, "Gheg"]);
        sort_tags(&mut received);
        assert_eq!(received, expected);
    }

    fn make_test_sort_tags_by_similar(received: &[&str], expected: &[&str]) {
        let mut vreceived: Vec<String> = to_string_vec(received);
        let vexpected: Vec<String> = to_string_vec(expected);
        sort_tags_by_similar(&mut vreceived);
        assert_eq!(vreceived, vexpected);
    }

    #[test]
    fn sort_tags_by_similar1() {
        make_test_sort_tags_by_similar(&["singular", "accusative"], &["accusative", "singular"]);
    }

    #[test]
    fn sort_tags_by_similar2() {
        make_test_sort_tags_by_similar(
            &["accusative", "singular", "neuter", "nominative", "vocative"],
            &["accusative", "neuter", "nominative", "singular", "vocative"],
        );
    }

    #[test]
    fn sort_tags_by_similar3() {
        make_test_sort_tags_by_similar(
            &["dual nominative", "accusative dual", "dual vocative"],
            &["accusative dual", "dual nominative", "dual vocative"],
        );
    }

    fn make_test_merge_person_tags(received: &[&str], expected: &[&str]) {
        let mut vreceived: Vec<String> = to_string_vec(received);
        let vexpected: Vec<String> = to_string_vec(expected);
        merge_person_tags(&mut vreceived);
        assert_eq!(vreceived, vexpected);
    }

    #[test]
    fn merge_person_tags1() {
        make_test_merge_person_tags(
            &[
                "first-person singular present",
                "third-person singular present",
            ],
            &["singular present first/third-person"],
        );
    }

    // Improvement over the original that would return:
    // "first/second-person singular past",
    // "third-person singular past",
    #[test]
    fn merge_person_tags2() {
        make_test_merge_person_tags(
            &[
                "first-person singular past",
                "second-person singular past",
                "third-person singular past",
            ],
            &["singular past first/second/third-person"],
        );
    }

    #[test]
    fn remove_redundant_tags1() {
        let mut received = to_string_vec(&["foo", "bar", "foo bar", "foo bar zee"]);
        let expected = to_string_vec(&["foo bar zee"]);
        remove_redundant_tags(&mut received);
        assert_eq!(received, expected);
    }

    #[test]
    fn remove_redundant_tags2() {
        let mut received = to_string_vec(&[
            "first-person singular indicative preterite",
            "first-person singular preterite",
        ]);
        let expected = to_string_vec(&["first-person singular indicative preterite"]);
        remove_redundant_tags(&mut received);
        assert_eq!(received, expected);
    }

    #[test]
    fn remove_redundant_tags_duplicates1() {
        let mut received = to_string_vec(&["a b", "a b"]);
        let expected = to_string_vec(&["a b"]);
        remove_redundant_tags(&mut received);
        assert_eq!(received, expected);
    }

    #[test]
    fn remove_redundant_tags_duplicates2() {
        let mut received = to_string_vec(&["a b", "b a"]);
        let expected = to_string_vec(&["b a"]);
        remove_redundant_tags(&mut received);
        assert_eq!(received, expected);
    }

    #[test]
    fn remove_redundant_tags_duplicates3() {
        let mut received = to_string_vec(&["a b", "c a b", "b a", "b a c", "c b a"]);
        let expected = to_string_vec(&["c b a"]);
        remove_redundant_tags(&mut received);
        assert_eq!(received, expected);
    }

    #[test]
    fn remove_redundant_tags_duplicates4() {
        let mut received = to_string_vec(&["s no ne", "ne s no", "ne", "no"]);
        let expected = to_string_vec(&["ne s no"]);
        remove_redundant_tags(&mut received);
        assert_eq!(received, expected);
    }

    #[test]
    fn tags_subsets() {
        assert!(tags_are_subset("foo bar", "bar foo baz"));
        assert!(!tags_are_subset("foo qux", "foo bar baz"));
    }

    use crate::{lang::Lang, models::yomitan::TagInformation};

    #[test]
    fn locale_ja_tag_bank() {
        let tag_bank = get_tag_bank_as_tag_info(Lang::Ja);
        let entry = ("動", "partOfSpeech", -2, &["動詞"][..], 2);
        let loc_tag_info = TagInformation::new(&entry);
        assert!(tag_bank.contains(&loc_tag_info))
    }

    #[test]
    fn locale_ja_translate_tag() {
        let loc_tag = localize_tag(Lang::Ja, "v");
        assert_eq!(loc_tag, Some(("動", "動詞")));
    }
}
