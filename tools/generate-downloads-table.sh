declare -a languages="($(
  jq -r '.[] | @json | @sh' languages.json
))"

echo "| | English (en) | Chinese (zh) | French (fr) | German (de) | Russian (ru) | Spanish (es) | Merged IPA |" > temp.md
echo "|---|---|---|---|---|---|---|---|" >> temp.md
columns=("en" "zh" "fr" "de" "ru" "es" "ipa")
for source_lang in "${languages[@]}"; do
    source_iso=$(echo "${source_lang}" | jq -r '.iso')
    source_language_name=$(echo "${source_lang}" | jq -r '.language')
    flag=$(echo "${source_lang}" | jq -r '.flag')
        
    row="| $flag </br> $source_language_name ($source_iso)"

    for column in "${columns[@]}"; do

        if [ "$column" = "ipa" ]; then
            expected_filenames=("${source_iso}-ipa")
        else
            expected_filenames=("${source_iso}-${column}" "${source_iso}-${column}-ipa")
        fi

        cell=""
        for expected_filename in "${expected_filenames[@]}"; do
            if [[ "$expected_filename" == *"-ipa" ]]; then
                display_filename="IPA for ${expected_filename%-ipa}"
            else
                display_filename="$expected_filename"
            fi

            cell="$cell [$display_filename](https://github.com/themoeway/kaikki-to-yomitan/releases/latest/download/kty-$expected_filename.zip) </br>"                
        done
        row="$row | $cell"
    done
    echo "$row" >> temp.md
done

cat temp.md
rm temp.md
