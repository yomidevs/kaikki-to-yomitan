find data/language -type f -name '*.zip' > zip_files.txt

check_file_exists() {
  local filename=$1
  while IFS= read -r file; do
    # Extract the part of the path after 'kty-' and before the '.zip'
    extracted_name=$(basename "$file" .zip | sed 's/^kty-//')
    if [ "$extracted_name" = "$filename" ]; then
      return 0
    fi
  done < zip_files.txt
  return 1
}

declare -a languages="($(
  jq -r '.[] | @json | @sh' languages.json
))"

echo "| | English (en) | Chinese (zh) | French (fr) | German (de) | Russian (ru) | Spanish (es) | Merged IPA |" > temp.md
echo "|---|---|---|---|---|---|---|---|" >> temp.md
columns=("en" "zh" "fr" "de" "ru" "es" "ipa")
for source_lang in "${languages[@]}"; do
    source_iso=$(echo "${source_lang}" | jq -r '.iso')
    source_language_name=$(echo "${source_lang}" | jq -r '.language')
        
    row="| $source_language_name ($source_iso)"

    for column in "${columns[@]}"; do

        if [ "$column" = "ipa" ]; then
            expected_filenames=("${source_iso}-ipa")
        else
            expected_filenames=("${source_iso}-${column}" "${source_iso}-${column}-ipa")
        fi

        cell=""
        for expected_filename in "${expected_filenames[@]}"; do
            if check_file_exists "$expected_filename"; then
                cell="$cell [kty-$expected_filename.zip](https://github.com/themoeway/kaikki-to-yomitan/releases/latest/download/kty-$expected_filename.zip) </br>"                
            fi
        done
        row="$row | $cell"
    done
    echo "$row" >> temp.md
done

cat temp.md
rm temp.md
