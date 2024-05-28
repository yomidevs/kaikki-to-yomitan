#!/bin/bash

source .env
export DEBUG_WORD
export DICT_NAME
max_memory_mb=${MAX_MEMORY_MB:-8192}

# Check for the source_language and target_language arguments
if [ -z "$1" ] || [ -z "$2" ]; then
  echo "Usage: $0 <source_language> <target_language> [flags]"
  exit 1
fi

# Parse flags
redownload=false
keep_files=false

flags=('d' 'k')
for flag in "${flags[@]}"; do
  case "$3" in 
    *"$flag"*) 
      case "$flag" in
        'd') redownload=true ;;
        'k') keep_files=true ;;
      esac
      ;;
  esac
done

echo "[d] redownload: $redownload"
echo "[k] keep_files: $keep_files"

# Step 1: Install dependencies
npm i

# Step 2: Run create-folder.js
node 1-create-folders.js

language_source="$1"
language_target="$2"

declare -a languages="($(
  jq -r '.[] | @json | @sh' languages.json
))"

for target_lang in "${languages[@]}"; do
  target_iso=$(echo "${target_lang}" | jq -r '.iso')
  target_language_name=$(echo "${target_lang}" | jq -r '.language')
    
  if [ "$target_language_name" != "$language_target" ] && [ "$language_target" != "?" ]; then
      continue
  fi

  export target_iso="$target_iso"
  export target_language="$target_language_name"

  for source_lang in "${languages[@]}"; do
    iso=$(echo "${source_lang}" | jq -r '.iso')
    language=$(echo "${source_lang}" | jq -r '.language')
    flag=$(echo "${source_lang}" | jq -r '.flag')
    
    if [ "$language" != "$language_source" ] && [ "$language_source" != "?" ]; then
      continue
    fi

    export source_language="$language"
    export source_iso="$iso"

    echo "------------------------------- $source_language -> $target_language -------------------------------"

    # Step 3: Download JSON data if it doesn't exist
    language_no_special_chars=$(echo "$language" | tr -d '[:space:]-') #Serbo-Croatian, Ancient Greek and such cases
    filename="kaikki.org-dictionary-$language_no_special_chars.json"
    filepath="data/kaikki/$filename"
    

    if [ ! -f "$filepath" ] || [ "$redownload" = true ]; then
      url="kaikki.org/dictionary/$language/$filename"
      echo "Downloading $filename from $url"
      wget "$url" -O "$filepath"
    else
      echo "Kaikki dict already exists. Skipping download."
    fi

    export kaikki_file="data/kaikki/$filename"
    export temp_folder="data/temp"

    dict_file="${DICT_NAME}-$source_iso-$target_iso-gloss.zip"

    # Step 5: Create Yomitan files
    echo "Creating Yomitan dict files"
    if node --max-old-space-size="$max_memory_mb" make-glossary.js; then
      echo "Zipping Yomitan files"
      zip -qj "$dict_file" $temp_folder/dict/index.json $temp_folder/dict/tag_bank_1.json $temp_folder/dict/term_bank_*.json
    else
      echo "Error: Yomitan generation script failed."
    fi

    if [ "$keep_files" = false ]; then
      rm -f "$kaikki_file"
    fi

    if [ -f "$dict_file" ]; then
      mv "$dict_file" "data/language/$source_iso/$target_iso/"
    fi

    echo "----------------------------------------------------------------------------------"
  done
done
echo "All done!"