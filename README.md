## wiktionary to yomitan (wty)

[![Build status](https://github.com/yomidedvs/wiktionary-to-yomitan/workflows/ci/badge.svg)](https://github.com/yomidedvs/wiktionary-to-yomitan/actions)

Converts wiktionary data from [kaikki](https://kaikki.org/) ([wiktextract](https://github.com/tatuylonen/wiktextract)) to [yomitan](https://github.com/yomidevs/yomitan)-compatible dictionaries.

This is a port of [kaikki-to-yomitan](https://github.com/yomidevs/wiktionary-to-yomitan/tree/old-master).

Converted dictionaries can be found on the [downloads](https://yomidevs.github.io/wiktionary-to-yomitan/download/) page.

## Usage

This example use German (de) to English (en).

```console
$ cargo install --git https://github.com/yomidevs/wiktionary-to-yomitan
$ wty main de en
...
✓ Wrote yomitan dict @ data/dict/de/en/wty-de-en.zip (20.94 MB)
```

A list of supported languages isos can be found [here](https://yomidevs.github.io/wiktionary-to-yomitan/language/).

For more information, see the [documentation](https://yomidevs.github.io/wiktionary-to-yomitan).
