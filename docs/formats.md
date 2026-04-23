# Formats

The full list of available formats can be seen in the CLI passing the `--help` flag.

| Format | CLI | Requires extra tools | Downloads | Used in |
|--------|-----|----------------------|-----------|---------|
| `yomitan` | ✅ | ❌ | ✅ | Yomitan |
| `mdict-text` | ✅ | ✅ | ❌ | GoldenDict-ng |
| `stardict` | ✅ | ❌ | ❌ | KOReader |

---

To make a dictionary in a certain format:

```console
// Defaults to yomitan
$ wty main ja en

// Glossary dictionary in mdict-text
$ wty glossary de en --format=mdict-text

// Ipa dictionary in stardict
$ wty ipa el el --format=stardict
```

---

## `yomitan` *(default)*

Produces a zip archive importable into yomitan. Downloads are available [here](download.md).

## `mdict-text`

Produces a plain-text file that can be consumed by any MDict conversion tool (f.e. [MDictUtils](https://github.com/daxida/MDictUtils) or [mdict-utils](https://github.com/liuyug/mdict-utils)), to make a `*.mdx` file. See [this](https://github.com/yomidevs/wiktionary-to-yomitan/issues/340#issuecomment-4276798146) for more information.

## `stardict`

Produces a folder with `*.ifo`, `*.dict` etc. files that can be directly imported.

KOReader dictionary install [guide](https://github.com/koreader/koreader/wiki/Dictionary-support).
