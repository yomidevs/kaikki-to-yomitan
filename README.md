[DOWNLOADS](./downloads.md)

Converts wiktionary data from https://kaikki.org/ to yomitan-compatible dictionaries. Converted dictionaries can be found in the [Downloads](./downloads.md) section.

# Instructions

(examples use German (de) to English (en))

## Basic Run

1. Create a `.env` file based on `.env.example`.

2. If your language is not in `languages.json`, add it.

3. Run `./auto.sh ? German English`.

4. Dictionaries should be in `data/language/de/en`.

## Script Parameters

The first parameter is the wiktionary edition. For some language pairs, like German-English, the main dictionary (`kty-de-en`) is made from the English wiktionary edition, but there is also the supplementary`kty-de-en-gloss`, which is made from the German wiktionary edition. In most cases you will want to use the same edition as your target language (3rd param) or `?` to use any edition.

The second and third parameters are the source and target languages.

- `./auto.sh ? ? English` will run for any language to English (using data from any edition).
- `./auto.sh ? German ?` will run for German to any language (using data from any edition).

The `auto.sh` script also accepts a 4th parameter, which is a combination of the following flags:

- k: keep files (by default, the script deletes the downloaded files after running),
- d: redownload (by default, the script skips downloading if the file already exists),
- t: force_tidy (run tidy script again, even if its output already exists. useful when the tidy script is updated),
- y: force_ymt (run yomitan script again, even if its output already exists. useful when the yomitan script is updated),
- F: force = force_tidy + force_ymt,

When trying to fix something, the `k` flag is useful prevent redownloading of files, and the `F` or `t` or `y` flags can be used to overwrite already converted files, e.g. `./auto.sh ? German English kF`

After a run, `data/language/de/en` should contain files with skipped tags for IPA and terms. Adding some to `tag_bank_ipa.json` or `tag_bank_term.json` is an easy way to improve the conversion for your language pair. 

### Tests

Test inputs are in `data/test/kaikki`. Each line is a line from the corresponding kaikki file (from `data/kaikki`, after downloading). 

To fix something in the conversion of a word, add its line from `data/kaikki` to the corresponding test file in `data/test/kaikki`. 
Then run `npm run test-write` to add it to the expected test output, and commit the changes (e.g. `add baseline test for "word"`).
Now when you modify tidy-up or make-yomitan, you can run `npm run test-write` to see the changes you made.

If you are making a change that shouldn't change the output, just run `npm run test` to check if anything broke.