use crate::{lang::Lang, path::DictionaryType};

const BASE_URL: &str = "https://huggingface.co/datasets/daxida/wty-release/resolve/main/latest";

// Helper function to sync index with the file tree.
// It is sort of a kludge due to the fact that write_yomitan expects a source: Lang
fn source_str(dict_ty: DictionaryType, source: &Lang) -> &str {
    match dict_ty {
        DictionaryType::Main
        | DictionaryType::Ipa
        | DictionaryType::Glossary
        | DictionaryType::GlossaryExtended => source.as_ref(),
        DictionaryType::IpaMerged => "all",
    }
}

/// The url to download this dictionary.
///
/// See: docs/javascripts/download.js (keep in sync)
fn download_url(
    dict_ty: DictionaryType,
    dict_name_expanded: &str,
    source: Lang,
    target: Lang,
) -> String {
    let source_str = source_str(dict_ty, &source);
    format!("{BASE_URL}/dict/{source_str}/{target}/{dict_name_expanded}.zip?download=true")
}

/// The url of the cloned index of this dictionary.
///
/// See: scripts/release.py (keep in sync)
fn index_url(dict_name_expanded: &str) -> String {
    format!("{BASE_URL}/index/{dict_name_expanded}-index.json?download=true")
}

// Original index attributes:
// https://github.com/yomidevs/kaikki-to-yomitan/blob/7b5bd7f922c9003b09f253f361b8a2e4ff26e13a/4-make-yomitan.js#L19
// https://github.com/yomidevs/kaikki-to-yomitan/blob/7b5bd7f922c9003b09f253f361b8a2e4ff26e13a/4-make-yomitan.js#L809
//
// How updating works:
// * check for updates
// https://github.com/yomidevs/yomitan/blob/d82684d9b746da60adb0e28dec5f4a4914da68c1/ext/js/pages/settings/dictionary-controller.js#L174
// 1. Fetch the new index from indexUrl
// 2. Compare revisions to see if the one from the new index comes *after* our current index
// 3. If so, store the downloadUrl of the new index
// 4. Show a button notifying that an update is available
// 5. If the user decides to update, download downloadUrl
//
/// Dictionary index.
///
/// indexUrl points to a separate copy of the index in the download repository.
/// downloadUrl points to the download link in the download repository.
///
/// <https://github.com/yomidevs/yomitan/blob/master/ext/data/schemas/dictionary-index-schema.json>
pub fn get_index(
    dict_ty: DictionaryType,
    dict_name_expanded: &str,
    source: Lang,
    target: Lang,
) -> String {
    let current_date = chrono::Utc::now().format("%Y.%m.%d"); // needs to be dot separated
    let index_url = index_url(dict_name_expanded);
    let download_url = download_url(dict_ty, dict_name_expanded, source, target);
    let source_str = source_str(dict_ty, &source);

    format!(
        r#"{{
  "title": "{dict_name_expanded}",
  "format": 3,
  "revision": "{current_date}",
  "sequenced": true,
  "author": "wty contributors",
  "url": "https://github.com/daxida/wty",
  "description": "Dictionaries for various language pairs generated from Wiktionary data, via Kaikki and wty.",
  "attribution": "https://kaikki.org/",
  "sourceLanguage": "{source_str}",
  "targetLanguage": "{target}",
  "isUpdatable": true,
  "indexUrl": "{index_url}",
  "downloadUrl": "{download_url}"
}}"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_download_ipa() {
        assert_eq!(
            download_url(DictionaryType::Ipa, "wty-afb-en-ipa", Lang::Afb, Lang::En),
            "https://huggingface.co/datasets/daxida/wty-release/resolve/main/latest/dict/afb/en/wty-afb-en-ipa.zip?download=true"
        );
    }

    #[test]
    fn url_download_ipa_merged() {
        assert_eq!(
            download_url(DictionaryType::IpaMerged, "wty-en-ipa", Lang::Afb, Lang::En),
            "https://huggingface.co/datasets/daxida/wty-release/resolve/main/latest/dict/all/en/wty-en-ipa.zip?download=true"
        );
    }

    #[test]
    fn url_index() {
        assert_eq!(
            index_url("wty-afb-en-ipa"),
            "https://huggingface.co/datasets/daxida/wty-release/resolve/main/latest/index/wty-afb-en-ipa-index.json?download=true"
        );
    }
}
