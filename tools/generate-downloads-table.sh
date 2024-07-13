first_text="# Downloads

Currently, [Kaikki](https://kaikki.org/dictionary/rawdata.html) supports 6 wiktionary editions (English, Chinese, French, German, Russian and Spanish), so only dictionaries including these languages are available.

If the language you want isn't here, or you would like to see an improvement to a dictionary, please [open an issue](https://github.com/themoeway/kaikki-to-yomitan/issues/new).

Some of the dictionaries listed here are small; rather than decide on a lower bound for usefulness they are all included here. 

<sub><sup> Languages are referred to by their shortest [ISO code](https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes) (ISO 639-1 where available, [ISO 639-3](https://en.wikipedia.org/wiki/List_of_ISO_639-3_codes) where not)</sup></sub>

## Main Dictionaries
This table contains the main dictionaries:

1. Bilingual dictionaries - \`en-de\` for example has English headwords and their definitions/translations in German.
2. Monolingual dictionaries - \`en-en\` and such. These have good coverage, but tend to be verbose.
"

{
  echo "$first_text"
} > downloads.md


declare -a languages="($(
  jq -r '.[] | @json | @sh' languages.json
))"

columns=()
header="| |"
divider="|---|"
for language in "${languages[@]}"; do
    language_name=$(echo "${language}" | jq -r '.language')
    iso=$(echo "${language}" | jq -r '.iso')
    hasEdition=$(echo "${language}" | jq -r '.hasEdition')
    if [ "$hasEdition" = "true" ]; then
        header="$header $language_name ($iso) |"
        divider="$divider---|"
        columns+=("$iso")
    fi
done

echo "$header" > main-table.md
echo "$divider" >> main-table.md

for source_lang in "${languages[@]}"; do
    source_iso=$(echo "${source_lang}" | jq -r '.iso')
    source_language_name=$(echo "${source_lang}" | jq -r '.language')
    flag=$(echo "${source_lang}" | jq -r '.flag')
        
    row="| $flag </br> $source_language_name ($source_iso)"

    for column in "${columns[@]}"; do
        cell=""
        expected_filename="${source_iso}-${column}"

        cell="$cell [$expected_filename](https://github.com/themoeway/kaikki-to-yomitan/releases/latest/download/kty-$expected_filename.zip) </br>"

        row="$row | $cell"
    done
    echo "$row" >> main-table.md
done


cat main-table.md >> downloads.md
rm main-table.md

second_text="## IPA Dictionaries
These dictionaries contain the International Phonetic Alphabet (IPA) transcriptions for the headwords. There are two types of IPA dictionaries:
1. IPA dictionaries from a single wiktionary edition - e.g. \`en-de\` contains IPA transcriptions for English headwords from the German wiktionary edition.
2. Merged IPA dictionaries from all 6 supported editions - e.g. \`en merged\`. These have more terms covered but not all the entries might be formatted the same way.
"

{
  echo "$second_text"
} >> downloads.md

ipa_header="$header Merged |"
ipa_divider="$divider---|"  
ipa_columns=("${columns[@]}" "merged")

echo "$ipa_header" > ipa-table.md
echo "$ipa_divider" >> ipa-table.md

for source_lang in "${languages[@]}"; do
    source_iso=$(echo "${source_lang}" | jq -r '.iso')
    source_language_name=$(echo "${source_lang}" | jq -r '.language')
    flag=$(echo "${source_lang}" | jq -r '.flag')
        
    row="| $flag </br> $source_language_name ($source_iso)"

    for column in "${ipa_columns[@]}"; do
        cell=""
        display_filename="${source_iso}-${column}"
        expected_filename="${display_filename}-ipa"
        if [ "$column" = "merged" ]; then
            expected_filename="${source_iso}-ipa"
            display_filename="${source_iso} merged"
        fi
        cell="$cell [$display_filename](https://github.com/themoeway/kaikki-to-yomitan/releases/latest/download/kty-$expected_filename.zip) </br>"

        row="$row | $cell"
    done
    echo "$row" >> ipa-table.md
done

cat ipa-table.md >> downloads.md
rm ipa-table.md

third_text="## Extra Dictionaries / Glossaries
These dictionaries are made from the "Translations" section in a Wiktionary entry. The entries are shorter and there is fewer of them compared to the main dictionaries, but they are available in some unique language pairs.

⚠️ This table is orientated opposite to the main dictionaries, with the source language in the columns and the target language in the rows.
"
    
{
echo "$third_text"
} >> downloads.md

echo "$header" > glossary-table.md
echo "$divider" >> glossary-table.md

for target_lang in "${languages[@]}"; do
    target_iso=$(echo "${target_lang}" | jq -r '.iso')
    target_language_name=$(echo "${target_lang}" | jq -r '.language')
    flag=$(echo "${target_lang}" | jq -r '.flag')
        
    row="| $flag </br> $target_language_name ($target_iso)"

    for column in "${columns[@]}"; do
        cell=""
        if [ "$column" != "$target_iso" ]; then
            display_filename="${column}"-"${target_iso}"
            expected_filename="${display_filename}-gloss"
            cell="$cell [$display_filename](https://github.com/themoeway/kaikki-to-yomitan/releases/latest/download/kty-$expected_filename.zip) </br>"
        fi
        row="$row | $cell"
    done
    echo "$row" >> glossary-table.md
done

cat glossary-table.md >> downloads.md
rm glossary-table.md
