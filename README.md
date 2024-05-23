Converts wiktionary data from https://kaikki.org/ to yomitan-compatible dictionaries. Converted dictionaries can be found in the [Releases](https://github.com/themoeway/kaikki-to-yomitan/releases) section.

# Instructions

(examples use German (de) to English (en))

## Basic Run

1. Create a `.env` file based on `.env.example`.

2. If your language is not in `languages.json`, add it.

3. Run `./auto.sh German English`.

4. Dictionaries should be in `data/language/de/en`.

## Contributing

The `auto.sh` script can also be run with flags:

- k: keep files (by default, the script deletes the downloaded files after running),
- d: redownload (by default, the script skips downloading if the file already exists),
- t: force_tidy (run tidy script again, even if its output already exists. useful when the tidy script is updated),
- y: force_ymt (run yomitan script again, even if its output already exists. useful when the yomitan script is updated),
- F: force = force_tidy + force_ymt,
- S: run for all source languages (`./auto.sh German English S` is like `./auto.sh * English`),
- T: run for all target languages (`./auto.sh German English T` is like `./auto.sh German *`).

Most often, you will want to run `./auto.sh German English kty` to recreate the dictionaries, then load them in yomitan and test them.

After a run, `data/language/de/en` should contain files with skipped tags for IPA and terms. Adding some to `tag_bank_ipa.json` or `tag_bank_term.json` is an easy way to improve the conversion for your language pair. 

### Tests

Test inputs are in `data/test/kaikki`. Each line is a line from the corresponding kaikki file (from `data/kaikki`, after downloading). 

To fix something in the conversion of a word, add its line from `data/kaikki` to the corresponding test file in `data/test/kaikki`. 
Then run `npm run test-write` to add it to the expected test output, and commit the changes (e.g. `add baseline test for "word"`).
Now when you modify tidy-up or make-yomitan, you can run `npm run test-write` to see the changes you made.

If you are making a change that shouldn't change the output, just run `npm run test` to check if anything broke.