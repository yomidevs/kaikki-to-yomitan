//! This file was generated and should not be edited directly.
//! The source code can be found at scripts/build.py

use crate::lang::Lang;

pub fn has_locale(lang: Lang) -> bool {
    match lang {
        Lang::De => true,
        Lang::Ja => true,
        _ => false,
    }
}

pub fn localize_tag(lang: Lang, short_tag: &str) -> Option<(&'static str, &'static str)> {
    match lang {
        Lang::De => localize_tag_de(short_tag),
        Lang::Ja => localize_tag_ja(short_tag),
        _ => None,
    }
}

/// Coverage: 17/361 tags (4.7%)
fn localize_tag_de(short_tag: &str) -> Option<(&'static str, &'static str)> {
    match short_tag {
        "n" => Some(("S", "Substantiv")),
        "masc" => Some(("Mask", "Maskulinum")),
        "fem" => Some(("Fem", "Femininum")),
        "neut" => Some(("Neut", "Neutrum")),
        "v" => Some(("V", "Verb")),
        "adj" => Some(("Adj", "Adjektiv")),
        "vt" => Some(("Vt", "transitives Verb")),
        "vi" => Some(("Vi", "intransitives Verb")),
        "name" => Some(("Vorn", "Vorname")),
        "arch" => Some(("veraltet", "veraltet")),
        "dated" => Some(("altmod", "altmodisch")),
        "col" => Some(("ums", "umgangssprachlich")),
        "fig" => Some(("übertragen", "übertragen")),
        "vulg" => Some(("vulgär", "vulgär")),
        "literal" => Some(("wörtlich", "wörtlich")),
        "lit" => Some(("liter", "literarisch")),
        "sports" => Some(("Sport", "Sport")),
        _ => None,
    }
}
/// Coverage: 36/361 tags (10.0%)
fn localize_tag_ja(short_tag: &str) -> Option<(&'static str, &'static str)> {
    match short_tag {
        "v" => Some(("動", "動詞")),
        "n" => Some(("名", "名詞")),
        "adj" => Some(("形", "形容詞")),
        "dated" => Some(("古風", "古風")),
        "arch" => Some(("古語", "古語")),
        "fig" => Some(("比喩", "比喩")),
        "abbv" => Some(("略", "略語")),
        "name" => Some(("名前", "固有名詞")),
        "ptcl" => Some(("助", "助詞")),
        "sl" => Some(("俗", "俗語")),
        "math" => Some(("数学", "数学")),
        "vt" => Some(("他動", "他動詞")),
        "vi" => Some(("自動", "自動詞")),
        "inf" => Some(("非形式的", "非形式的")),
        "lit" => Some(("文語", "文語")),
        "pej" => Some(("軽蔑的", "軽蔑的")),
        "dialect" => Some(("方言", "方言")),
        "rare" => Some(("まれ", "まれ")),
        "adv" => Some(("副", "副詞")),
        "artic" => Some(("定", "定冠詞")),
        "aux-v" => Some(("助動", "助動詞")),
        "conj" => Some(("接続", "接続詞")),
        "contr" => Some(("縮約", "縮約形")),
        "intj" => Some(("間投", "間投詞")),
        "pron" => Some(("代", "代名詞")),
        "philos" => Some(("哲学", "哲学")),
        "suf" => Some(("接尾辞", "接尾辞")),
        "adj_noun" => Some(("形容動詞", "形容動詞")),
        "godan" => Some(("五段", "五段活用")),
        "ichidan" => Some(("一段", "一段活用")),
        "shimoichidan" => Some(("下一段", "下一段活用")),
        "onoma" => Some(("オノマ", "オノマトペ")),
        "adn" => Some(("連体詞", "連体詞")),
        "ling" => Some(("言語学", "言語学")),
        "edu" => Some(("教育", "教育")),
        "prep" => Some(("前", "前置詞")),
        _ => None,
    }
}
