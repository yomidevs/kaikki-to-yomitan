## Customize css

- The css used by all dictionaries can be found [here](https://raw.githubusercontent.com/yomidevs/wiktionary-to-yomitan/master/assets/styles.css).
- Yomitan documentation contains [themes](https://yomitan.wiki/tools-resources/?h=css#yomitan-colored-css) and how to set them up.
- The sunset Yomichan has a still mostly valid [guide](https://learnjapanese.moe/yomicss/) on how to customize css.

### Tags

The first thing to know, if you want the css to only affect a `wty` dictionary, is to preface every declaration with: 

```css
[data-dictionary="dictionary-name"]
```

For example, this will only show the "Hokkien" pronunciation in `wty-zh-en-ipa`:

```css
[data-dictionary="wty-zh-en-ipa"] .pronunciation {
    display: none;
}
[data-dictionary="wty-zh-en-ipa"] .pronunciation:has(.tag[data-details="Hokkien"]) {
    display: block;
}
```

!!! note "From now on, we will skip this declaration for simplicity."

#### Main dictionary

Here is a basic example on how to handle these tags with your custom css.

```css
/* Hide inflections (~book icon, inflection tags) */
.inflection-rule-chains { display: none; }

/* Hide dictionary name (top-level tag) */
[data-category="dictionary"] { display: none; }
/* Hide gender tags (top-level tag) */
[data-category="gender"] { display: none; }

/* Hide inner topic tags (note the -sc-) */
[data-sc-category="topic"] { display: none; }
```

#### Ipa dictionary

This css will only show "Hokkien" and "bopomofo" pronunciations:

```css
.pronunciation {
    display: none;
}
.pronunciation:has(.tag[data-details="Hokkien"]) {
    display: block;
}
.pronunciation:has(.tag[data-details="bopomofo"]) {
    display: block;
}
```
