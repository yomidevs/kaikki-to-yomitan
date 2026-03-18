The mental model for tag extraction is as follows:
```
Wiktionary
  │
  │ (wiktextract)
  │
  ├─> raw_tags
  │     ↓
  └─> tags
        ↓
   tag filtering (which tags to keep, and their short forms)
        ↓
   tag formatting (global css)
        ↓
   (optional, custom css in yomitan)
        ↓
   shown tag in yomitan
```

---

There are (at least) three cases in which one may want to modify tags:

1 - **Tag order**: the order in which tags are displayed.  
2 - **Tag filtering**: abbreviations and which tags are displayed.  
3 - **Extraction logic**: where to extract tags from wiktionary data.

Tag postprocessing is done after building the whole intermediate representation, to only sort once with every extracted tag. The relevant function is `src/dict/main.rs::postprocess_forms`.

### Tag order

Tag order is recorded in `assets/tag_order.json`. While this file has categories (formatility, cases etc.), those are later strip and serve only as visual help. The sorting is done with the flattened list.

!!! warning "Run the build script after any modification to update the rust code: either `just build` or `python3 scripts/build.py`"

### Tag filtering

Tag filtering is recorded in `assets/tag_bank_term.json`. The items of this JSON list are a custom version of:

```typescript
type TagInformation = [
  tagName: string,
  category: string,
  sortingOrder: number,
  notes: string,
  popularityScore: number,
];
```

where `notes` is replaced with either a string, or a list of strings representing aliases, the **first** one being shown when hovering the tag.

For example, this tag information:

```json
[
    "abbv",
    "",
    0,
    [
        "abbreviation",
        "abbrev"
    ],
    0
]
```

will convert both the kaikki tags `abbreviation` and `abbrev` into `abbv`, and show `abbreviation` when hovered in yomitan.

Here is an example of a simple [commit](https://github.com/yomidevs/wiktionary-to-yomitan/commit/00c69daa89344d971978d905897aa19e7c1ae619) to add the "Buddhism" tag, that modifies the JSON, then runs the build script to update the rust code. Other example adding multiple tags [here](https://github.com/yomidevs/wiktionary-to-yomitan/commit/0b8013b0fe01f17a543a840200733a431bc1187b).

!!! warning "Run the build script after any modification to update the rust code: either `just build` or `python3 scripts/build.py`"

### Extraction logic

This requires some knowledge of kaikki internals and how they extract tags.

We only use the normalized, english `tags`, as opposed to `raw_tags`, which are the tags you may see in Wiktionary, in the edition language. Therefore, if kaikki hasn't gone through the work of translating the tag, it will not appear in the dictionary. If that is your case, see this [issue](https://github.com/yomidevs/wiktionary-to-yomitan/issues/84), and the associated [PR](https://github.com/tatuylonen/wiktextract/pull/997) in kaikki to have a grasp on how to request/add translations.

It can also happen that kaikki doesn't extract tags from certain templates. In that case, again, it may be worth reporting the issue to them.

TODO: explain tags at top level / sense level, why this can make some tags to not appear and the hack we do for Greek, Russian etc.

