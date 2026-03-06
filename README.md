## wiktionary to yomitan (wty)

[![Build status](https://github.com/daxida/wty/workflows/ci/badge.svg)](https://github.com/daxida/wty/actions)

Converts wiktionary data from [kaikki](https://kaikki.org/) ([wiktextract](https://github.com/tatuylonen/wiktextract)) to [yomitan](https://github.com/yomidevs/yomitan)-compatible dictionaries.

This is a port of [kaikki-to-yomitan](https://github.com/yomidevs/wiktionary-to-yomitan/tree/old-master).

Converted dictionaries can be found on the [downloads](https://daxida.github.io/wty/download) page.

## Usage

This example use German (de) to English (en).

```console
$ cargo install --git https://github.com/daxida/wty
$ wty main de en
...
✓ Wrote yomitan dict @ data/dict/de/en/wty-de-en.zip (20.94 MB)
```

A list of supported languages isos can be found [here](https://daxida.github.io/wty/language/).

For more information, see the [documentation](https://daxida.github.io/wty).
