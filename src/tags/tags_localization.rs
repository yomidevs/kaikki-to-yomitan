//! This file was generated and should not be edited directly.
//! The source code can be found at scripts/build.py

use crate::lang::Lang;

pub fn has_locale(lang: Lang) -> bool {
    match lang {
        Lang::Ja => true,
        _ => false,
    }
}

pub fn localize_tag(lang: Lang, short_tag: &str) -> Option<(&'static str, &'static str)> {
    match lang {
        Lang::Ja => localize_tag_ja(short_tag),
        _ => None,
    }
}

/// Coverage: 9/258 tags (3.5%)
fn localize_tag_ja(short_tag: &str) -> Option<(&'static str, &'static str)> {
    match short_tag {
        "v" => Some(("動", "動詞")),
        "n" => Some(("名", "名詞")),
        "adj" => Some(("形", "形容詞")),
        "dated" => Some(("旧", "古風")),
        "arch" => Some(("古", "古語")),
        "name" => Some(("名前", "固有名詞")),
        "ptcl" => Some(("助", "助詞")),
        "sl" => Some(("俗", "俗語")),
        "math" => Some(("数", "数学")),
        _ => None,
    }
}
