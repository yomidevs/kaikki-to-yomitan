# Makes rust types from language.json

# Note that there is also the isolang.rs crate

import argparse
import json
import re
from dataclasses import dataclass, astuple
from pathlib import Path
from typing import Any


@dataclass
class Lang:
    iso: str
    language: str
    display_name: str
    flag: str
    # https://github.com/tatuylonen/wiktextract/tree/master/src/wiktextract/extractor
    has_edition: bool


@dataclass
class WhitelistedTag:
    short_tag: str
    category: str
    sort_order: int
    # if array, first element will be used, others are aliases
    long_tag_aliases: str | list[str]
    popularity_score: int

    def long_tag(self) -> str:
        if isinstance(self.long_tag_aliases, str):
            return self.long_tag_aliases
        return self.long_tag_aliases[0]


type Locale = dict[
    str,  # iso
    dict[
        str,  # English long tag
        tuple[
            str,  # Localized short tag
            str,  # Localized long tag
        ],
    ],
]


def write_warning(f) -> None:
    f.write("//! This file was generated and should not be edited directly.\n")
    f.write("//! The source code can be found at scripts/build.py\n\n")


def generate_tags_rs(
    tag_order: list[str],
    whitelisted_tags: list[WhitelistedTag],
    f,
) -> None:
    # SAFETY: because the IPA dictionary uses only short tags and relies on yomitan
    # to complete the long version on hover based on tag_bank_*.json, it is important to
    # not have short tag duplicates, unless the meaning of the long version are the same.
    seen = {}
    for wt in whitelisted_tags:
        st = wt.short_tag
        if st in ("fig", "dialect"):
            # These are ok, since the long versions are the same (figurative, figuratively)
            continue
        if st in seen:
            old = seen[st]
            print(f"WARN: duplicated short tag\n{wt}\n{old}")
        else:
            seen[st] = wt

    idt = " " * 4
    w = f.write  # shorthand

    write_warning(f)

    w(f"pub const TAG_ORDER: [&str; {len(tag_order)}] = [\n")
    for tag in tag_order:
        w(f'{idt}"{tag}",\n')
    w("];\n\n")

    # Not sure why all of this was done in the original, it makes almost no sense

    w("#[rustfmt::skip]\n")
    w(
        f"pub const TAG_BANK: [(&str, &str, i32, &[&str], i32); {len(whitelisted_tags)}] = [\n"
    )
    for wt in whitelisted_tags:
        longs_as_list = (
            wt.long_tag_aliases
            if isinstance(wt.long_tag_aliases, list)
            else [wt.long_tag_aliases]
        )
        longs_str = str(longs_as_list).replace("'", '"')
        w(
            f'{idt}("{wt.short_tag}", "{wt.category}", {wt.sort_order}, &{longs_str}, {wt.popularity_score}),\n'
        )
    w("];\n\n")

    wts_pos = []
    for wt in whitelisted_tags:
        if wt.category == "partOfSpeech":
            if isinstance(wt.long_tag_aliases, list):
                for alias in wt.long_tag_aliases:
                    wts_pos.append((alias, wt.short_tag))
            else:
                wts_pos.append((wt.long_tag_aliases, wt.short_tag))

    w(f"pub const POSES: [(&str, &str); {len(wts_pos)}] = [\n")
    for long, short in wts_pos:
        w(f'{idt}("{long}", "{short}"),\n')
    w("];\n")


def generate_lang_rs(langs: list[Lang], f) -> None:
    idt = " " * 4
    w = f.write  # shorthand

    write_warning(f)

    # w("#![rustfmt::skip]\n")

    w("""use std::{
    fmt::{Debug, Display},
    hash::Hash,
    str::FromStr,
};\n\n""")

    w("use serde::{Deserialize, Serialize};\n\n")

    ### Trait

    shared_traits = [
        "Clone",
        "Debug",
        "Display",
        "FromStr",
        "AsRef<str>",
        "PartialEq",
        "Eq",
        "Hash",
    ]
    w("// The idea is from https://github.com/johnstonskj/rust-codes/tree/main\n")
    w(f"pub trait Code: {' + '.join(shared_traits)} {{}}\n\n")
    w("impl Code for Lang {}\n")
    w("impl Code for EditionSpec {}\n")
    w("impl Code for Edition {}\n")
    w("\n")

    ### Lang start

    # Lang
    w("#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]\n")
    w("pub enum Lang {\n")
    for lang in langs:
        w(f"{idt}/// {lang.language}\n")  # doc
        w(f"{idt}{lang.iso.title()},\n")
    w("}\n\n")

    # Lang: From<Edition>
    w("impl From<Edition> for Lang {\n")
    w(f"{idt}fn from(value: Edition) -> Self {{\n")
    w(f"{idt * 2}match value {{\n")
    for lang in langs:
        if lang.has_edition:
            w(f"{idt * 3}Edition::{lang.iso.title()} => Self::{lang.iso.title()},\n")
    w(f"{idt * 2}}}\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    w("impl Lang {\n")

    # Lang: help_messages
    is_supported = " | ".join(lang.iso for lang in langs)
    fn_name = "help_isos"
    w(f"{idt}pub const fn {fn_name}() -> &'static str {{\n")
    w(f'{idt * 2}"Supported isos: {is_supported}"\n')
    w(f"{idt}}}\n\n")

    coloured_parts = [
        f"\x1b[32m{lang.iso}\x1b[0m" if lang.has_edition else lang.iso for lang in langs
    ]
    isos_colored = " | ".join(coloured_parts)
    w(f"{idt}pub const fn help_isos_coloured() -> &'static str {{\n")
    w(f'{idt * 2}"Supported isos: {isos_colored}"\n')
    w(f"{idt}}}\n\n")

    with_edition = " | ".join(lang.iso for lang in langs if lang.has_edition)
    w(f"{idt}pub const fn help_editions() -> &'static str {{\n")
    w(f'{idt * 2}"Supported editions: {with_edition}"\n')
    w(f"{idt}}}\n\n")

    # Lang: long. long: Lang::El => "Greek"
    w(f"{idt}pub const fn long(&self) -> &'static str {{\n")
    w(f"{idt * 2}match self {{\n")
    for lang in langs:
        w(f'{idt * 3}Self::{lang.iso.title()} => "{lang.language}",\n')
    w(f"{idt * 2}}}\n")
    w(f"{idt}}}\n\n")

    # Lang: all (iteration)
    w(f"{idt}pub fn all() -> Vec<Self> {{\n")
    w(f"{idt * 2}vec![\n")
    for lang in langs:
        w(f"{idt * 3}Self::{lang.iso.title()},\n")
    w(f"{idt * 2}]\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    # Lang: TryInto<Edition>
    w("impl TryInto<Edition> for Lang {\n")
    w(f"{idt}type Error = &'static str;\n\n")
    w(f"{idt}fn try_into(self) -> Result<Edition, Self::Error> {{\n")
    w(f"{idt * 2}match self {{\n")
    for lang in langs:
        if lang.has_edition:
            w(
                f"{idt * 3}Self::{lang.iso.title()} => Ok(Edition::{lang.iso.title()}),\n"
            )
    w(f'{idt * 3}_ => Err("language has no edition"),\n')
    w(f"{idt * 2}}}\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    # Lang: FromStr
    w("impl FromStr for Lang {\n")
    w(f"{idt}type Err = String;\n\n")
    w(f"{idt}fn from_str(s: &str) -> Result<Self, Self::Err> {{\n")
    w(f"{idt * 2}match s.to_lowercase().as_str() {{\n")
    for lang in langs:
        w(f'{idt * 3}"{lang.iso.lower()}" => Ok(Self::{lang.iso.title()}),\n')
    w(
        f"{idt * 3}_ => Err(format!(\"unsupported iso code '{{s}}'\\n{{}}\", Self::{fn_name}())),\n"
    )
    w(f"{idt * 2}}}\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    # Lang: AsRef<&str>
    w("impl AsRef<str> for Lang {\n")
    w(f"{idt}fn as_ref(&self) -> &str {{\n")
    w(f"{idt * 2}match self {{\n")
    for lang in langs:
        w(f'{idt * 3}Self::{lang.iso.title()} => "{lang.iso.lower()}",\n')
    w(f"{idt * 2}}}\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    # Lang: iso - to separate lang_code filtering from json path fetching logic
    w("impl Lang {\n")
    w(f"{idt}pub fn iso(&self) -> &str {{\n")
    w(f"{idt * 2}match self {{\n")
    w(f'{idt * 3}Self::Simple => "en",\n')
    w(f"{idt * 3}_ => self.as_ref(),\n")
    w(f"{idt * 2}}}\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    # Lang: Display
    w("impl Display for Lang {\n")
    w(f"{idt}fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{\n")
    w(f"{idt * 2}f.write_str(self.as_ref())\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    ### EditionSpec start

    # EditionSpec
    w("#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]\n")
    w("pub enum EditionSpec {\n")
    w(f"{idt}/// All editions\n")
    w(f"{idt}All,\n")
    w(f"{idt}/// An `Edition`\n")
    w(f"{idt}One(Edition),\n")
    w("}\n\n")

    # EditionSpec: variants (iteration)
    w("impl EditionSpec {\n")
    w(f"{idt}pub fn variants(&self) -> Vec<Edition> {{\n")
    w(f"{idt * 2}match self {{\n")
    w(f"{idt * 3}Self::All => Edition::all(),\n")
    w(f"{idt * 3}Self::One(lang) => vec![*lang],\n")
    w(f"{idt * 2}}}\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    # EditionSpec: From<Edition>
    w("impl From<Edition> for EditionSpec {\n")
    w(f"{idt}fn from(val: Edition) -> Self {{\n")
    w(f"{idt * 2}Self::One(val)\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    # EditionSpec: TryInto<Edition>
    w("impl TryInto<Edition> for EditionSpec {\n")
    w(f"{idt}type Error = &'static str;\n\n")
    w(f"{idt}fn try_into(self) -> Result<Edition, Self::Error> {{\n")
    w(f"{idt * 2}match self {{\n")
    w(f'{idt * 3}Self::All => Err("cannot convert from All"),\n')
    w(f"{idt * 3}Self::One(lang) => Ok(lang),\n")
    w(f"{idt * 2}}}\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    # EditionSpec: FromStr
    w("impl FromStr for EditionSpec {\n")
    w(f"{idt}type Err = String;\n\n")
    w(f"{idt}fn from_str(s: &str) -> Result<Self, Self::Err> {{\n")
    w(f"{idt * 2}match s {{\n")
    w(f'{idt * 3}"all" => Ok(Self::All),\n')
    w(f"{idt * 3}other => Ok(Self::One(Edition::from_str(other)?)),\n")
    w(f"{idt * 2}}}\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    # EditionSpec: AsRef<&str>
    w("impl AsRef<str> for EditionSpec {\n")
    w(f"{idt}fn as_ref(&self) -> &str {{\n")
    w(f"{idt * 2}match self {{\n")
    w(f'{idt * 3}Self::All => "all",\n')
    w(f"{idt * 3}Self::One(lang) => lang.as_ref(),\n")
    w(f"{idt * 2}}}\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    # EditionSpec: Display
    w("impl Display for EditionSpec {\n")
    w(f"{idt}fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{\n")
    w(f"{idt * 2}f.write_str(self.as_ref())\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    ### Edition start

    # Edition
    w("#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]\n")
    w("pub enum Edition {\n")
    for lang in langs:
        if lang.has_edition:
            w(f"{idt}/// {lang.language}\n")  # doc
            w(f"{idt}{lang.iso.title()},\n")
    w("}\n\n")

    # Edition: all (iteration)
    w("impl Edition {\n")
    w(f"{idt}pub fn all() -> Vec<Self> {{\n")
    w(f"{idt * 2}vec![\n")
    for lang in langs:
        if lang.has_edition:
            w(f"{idt * 3}Self::{lang.iso.title()},\n")
    w(f"{idt * 2}]\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    # Edition: FromStr
    w("impl FromStr for Edition {\n")
    w(f"{idt}type Err = String;\n\n")
    w(f"{idt}fn from_str(s: &str) -> Result<Self, Self::Err> {{\n")
    w(f"{idt * 2}match s.to_lowercase().as_str() {{\n")
    for lang in langs:
        if lang.has_edition:
            w(f'{idt * 3}"{lang.iso.lower()}" => Ok(Self::{lang.iso.title()}),\n')
    w(f"{idt * 3}_ => Err(format!(\"invalid edition '{{s}}'\")),\n")
    w(f"{idt * 2}}}\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    # Edition: AsRef<&str>
    w("impl AsRef<str> for Edition {\n")
    w(f"{idt}fn as_ref(&self) -> &str {{\n")
    w(f"{idt * 2}match self {{\n")
    for lang in langs:
        if lang.has_edition:
            w(f'{idt * 3}Self::{lang.iso.title()} => "{lang.iso.lower()}",\n')
    w(f"{idt * 2}}}\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    # Edition: iso - to separate lang_code filtering from json path fetching logic
    w("impl Edition {\n")
    w(f"{idt}pub fn iso(&self) -> &str {{\n")
    w(f"{idt * 2}match self {{\n")
    w(f'{idt * 3}Self::Simple => "en",\n')
    w(f"{idt * 3}_ => self.as_ref(),\n")
    w(f"{idt * 2}}}\n")
    w(f"{idt}}}\n")
    w("}\n\n")

    # Edition: Display
    w("impl Display for Edition {\n")
    w(f"{idt}fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{\n")
    w(f"{idt * 2}f.write_str(self.as_ref())\n")
    w(f"{idt}}}\n")
    w("}\n")


def generate_tags_localization(
    locale: Locale, whitelisted_tags: list[WhitelistedTag], f
) -> None:
    w = f.write

    # SAFETY: check that all locale keys match a known long tag
    known_long_tags = {wt.long_tag() for wt in whitelisted_tags}
    for iso, iso_translations in locale.items():
        for key in iso_translations:
            if key not in known_long_tags:
                print(
                    f"[{iso}] WARN: locale key '{key}' has no matching tag bank entry"
                )

    write_warning(f)

    w("use crate::lang::Lang;\n\n")

    w("pub fn has_locale(lang: Lang) -> bool {\n")
    w("    match lang {\n")
    for iso in locale:
        w(f"        Lang::{iso.title()} => true,\n")
    w("        _ => false,\n")
    w("    }\n")
    w("}\n\n")

    w(
        "pub fn localize_tag(lang: Lang, short_tag: &str) -> Option<(&'static str, &'static str)> {\n"
    )
    w("    match lang {\n")
    for iso in locale:
        w(f"        Lang::{iso.title()} => localize_tag_{iso}(short_tag),\n")
    w("        _ => None,\n")
    w("    }\n")
    w("}\n\n")

    long_to_short = {wt.long_tag(): wt.short_tag for wt in whitelisted_tags}

    for iso, translations in locale.items():
        w(
            f"fn localize_tag_{iso}(short_tag: &str) -> Option<(&'static str, &'static str)> {{\n"
        )
        w("    match short_tag {\n")
        for key, (short, long) in translations.items():
            short_key = long_to_short[key]
            w(f'        "{short_key}" => Some(("{short}", "{long}")),\n')
        w("        _ => None,\n")
        w("    }\n")
        w("}\n")


def load_lang(item: Any) -> Lang:
    return Lang(
        item["iso"],
        item["language"],
        item["displayName"],
        item["flag"],
        item.get("hasEdition", False),
    )


def load_langs(path: Path) -> list[Lang]:
    with path.open() as f:
        data = json.load(f)
    return [load_lang(item) for item in data]


def sort_languages_json(path: Path) -> None:
    with path.open() as f:
        text = f.read()

    lines = text.splitlines()
    langs = [
        (load_lang(json.loads(line.strip(","))), idx)
        for idx, line in enumerate(lines[1:-1])
    ]

    langs_sorted = sorted(langs, key=lambda pair: pair[0].display_name)
    if langs == langs_sorted:
        return

    with path.open("w") as f:
        f.write("[\n")
        for _, idx in langs_sorted:
            f.write(lines[idx + 1])
            f.write("\n")
        f.write("]\n")


def check_yomitan_langs(langs: list[Lang]) -> None:
    """Check if we support at least what is supported by yomitan.

    Since it sends a request, it is gated under the --check-yomitan flag.
    """
    import requests

    url = "https://raw.githubusercontent.com/yomidevs/yomitan/master/ext/js/language/language-descriptors.js"
    response = requests.get(url)
    response.raise_for_status()
    js_text = response.text

    # Get iso and names
    # ~ we assume that there is no inner lists [] in the descriptors.
    mch = re.search(r"const languageDescriptors\s*=\s*\[(.*?)\];", js_text, re.DOTALL)
    if not mch:
        print("Regex didn't match")
        return
    content = mch.group(1)

    # Quick and dirty regex to get iso/names
    iso_re = re.compile(r"iso: '(.*)',")
    name_re = re.compile(r"name: '(.*)',")
    isos = []
    names = []

    for line in content.splitlines():
        if iso_match := iso_re.search(line):
            isos.append(iso_match.group(1))
        if name_match := name_re.search(line):
            names.append(name_match.group(1))
    assert len(isos) == len(names)

    our_iso_map = {lang.iso: lang for lang in langs}
    missing_isos = []
    different_names = []

    for ymt_iso, ymt_name in zip(isos, names):
        if ymt_iso not in our_iso_map:
            # This iso is supported by yomitan but not us
            missing_iso = f"[missing iso] {ymt_iso} ({ymt_name})"
            missing_isos.append(missing_iso)
        else:
            our_lang = our_iso_map[ymt_iso]
            if ymt_name != our_lang.language:
                # For Arabic (and relatives), we have:
                # * yomitan: name='Arabic (MSA)'
                # * we:      language='Arabic', display_name='Arabic, MSA',
                #
                # In this case the name is different but it is fine.
                if ", " in our_lang.display_name:
                    main, variant = our_lang.display_name.split(", ")
                    rebuilt = f"{main} ({variant})"
                    if ymt_name == rebuilt:
                        continue

                # We have this iso, but the name is different
                different_name = f"[different name] {ymt_name=} but {our_lang=}"
                different_names.append(different_name)

    for logs, label in (
        (missing_isos, "missing_isos"),
        (different_names, "different_names"),
    ):
        if logs:
            for log in logs:
                print(log)
        else:
            print(f"✓ No {label}")


def check_kaikki_langs(langs: list[Lang]) -> None:
    """Check for unsupported (by us) languages in https://kaikki.org/dictionary/

    Since it sends a request, it is gated under the --check-kaikki flag.
    """
    import requests

    url = "https://kaikki.org/dictionary/"
    response = requests.get(url)
    response.raise_for_status()
    response.encoding = "utf-8"
    text = response.text

    supported = {lang.language for lang in langs}
    upto = 80

    # Get names (isos are not in the website)
    matches = re.findall(
        r"<li><a href=\"[^/]+/index\.html\">([^<]+) \((\d+) senses\)</a></li>",
        text,
        re.DOTALL,
    )

    for lang, num_senses in matches[:upto]:
        if lang in (
            "All languages combined",
            "Translingual",
            "Mandarin",  # We call it Chinese
        ):
            continue
        if lang not in supported:
            print(f"[missing from English kaikki ({upto})] {lang}, {num_senses}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check-yomitan", action="store_true")
    parser.add_argument("--check-kaikki", action="store_true")
    args = parser.parse_args()
    check_yomitan = args.check_yomitan
    check_kaikki = args.check_kaikki

    src = Path("src")
    path_lang_rs = src / "lang.rs"
    path_tags_rs = src / "tags" / "tags_constants.rs"
    path_tags_loc_rs = src / "tags" / "tags_localization.rs"
    jsons_root = Path("assets")
    path_languages_json = jsons_root / "languages.json"
    path_tag_order_json = jsons_root / "tag_order.json"
    path_tag_bank_json = jsons_root / "tag_bank_term.json"
    path_tag_locale_folder = jsons_root / "tags" / "locale"

    for path in (
        path_languages_json,
        path_tag_order_json,
        path_tag_bank_json,
        path_tag_locale_folder,
    ):
        if not path.exists:
            print(f"Path does not exist @ {path}")
            return

    sort_languages_json(path_languages_json)

    langs = load_langs(path_languages_json)

    if check_kaikki:
        check_kaikki_langs(langs)

    if check_yomitan:
        check_yomitan_langs(langs)

    tag_order: list[str] = []
    with path_tag_order_json.open() as f:
        data = json.load(f)
        for _, tags in data.items():
            tag_order.extend(tags)
    # Overwrite to ensure formatting
    with path_tag_order_json.open("w") as f:
        json.dump(data, f, indent=4, ensure_ascii=False)

    with path_tag_bank_json.open() as f:
        data = json.load(f)
    whitelisted_tags = [WhitelistedTag(*row) for row in data]
    for wtag in whitelisted_tags:
        if " " in wtag.short_tag:
            # This is an issue because yomitan will treat them as separate tags.
            print(
                f"WARN: space detected in short tag: {wtag.short_tag}. Use an hyphen instead."
            )
    # Overwrite to ensure formatting and sort
    whitelisted_tags.sort(
        key=lambda wt: (
            wt.category == "",  # No category goes at the bottom
            wt.category,
            wt.sort_order,
            wt.short_tag,
        )
    )
    with path_tag_bank_json.open("w") as f:
        json.dump(
            [astuple(wt) for wt in whitelisted_tags], f, indent=4, ensure_ascii=False
        )

    # read tags_X.json files inside localization folder, where X is the iso
    locale: Locale = {}
    for lang in langs:
        path_localization_json = path_tag_locale_folder / f"tags_{lang.iso}.json"
        if not path_localization_json.exists():
            continue
        # Overwrite to ensure formatting
        with path_localization_json.open() as f:
            data = json.load(f)
        with path_localization_json.open("w") as f:
            json.dump(data, f, indent=4, ensure_ascii=False)
        locale[lang.iso] = data

    # import sys
    # generate_lang_rs(langs, sys.stdout)
    # generate_tags_rs(tag_order, sys.stdout)

    with path_lang_rs.open("w") as f:
        generate_lang_rs(langs, f)
        print(f"Wrote rust code @ {path_lang_rs}")
    with path_tags_rs.open("w") as f:
        generate_tags_rs(tag_order, whitelisted_tags, f)
        print(f"Wrote rust code @ {path_tags_rs}")
    with path_tags_loc_rs.open("w") as f:
        generate_tags_localization(locale, whitelisted_tags, f)
        print(f"Wrote rust code @ {path_tags_loc_rs}")


if __name__ == "__main__":
    main()
