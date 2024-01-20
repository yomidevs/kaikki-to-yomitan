#!/bin/bash

source .env

export MAX_SENTENCES
export DEBUG_WORD
export OPENSUBS_PATH
export DICT_NAME

# Check for the source_language and target_language arguments
if [ -z "$1" ] || [ -z "$2" ]; then
  echo "Usage: $0 <source_language> <target_language> [flags]"
  exit 1
fi

# Parse flags
source_all=false
target_all=false
redownload=false
force_tidy=false
force_freq=false
force_yez=false
force=false

flags=('S' 'T' 'd' 't' 'f' 'y' 'F')
for flag in "${flags[@]}"; do
  case "$3" in 
    *"$flag"*) 
      case "$flag" in
        'S') source_all=true ;;
        'T') target_all=true ;;
        'd') redownload=true ;;
        't') force_tidy=true ;;
        'f') force_freq=true ;;
        'y') force_yez=true ;;
        'F') force=true ;;
      esac
      ;;
  esac
done

if [ "$force" = true ]; then
  force_tidy=true
  force_freq=true
  force_yez=true
fi

if [ "$force_tidy" = true ] || [ "$force_freq" = true ]; then
  force_yez=true
fi

echo "[S] source_all: $source_all"
echo "[T] target_all: $target_all"
echo "[d] redownload: $redownload"
echo "[F] force: $force"
echo "[t] force_tidy: $force_tidy"
echo "[f] force_freq: $force_freq"
echo "[y] force_yez: $force_yez"

# Step 1: Install dependencies
npm i

# Step 2: Run create-folder.js
node 1-create-folders.js

languages=$(jq '.' ../ext/js/language/languages.json)

source_language="$1"
target_language="$2"

declare -a entries="($(
  jq -r '.[] | @json | @sh' ../ext/js/language/languages.json
))"

for entry in "${entries[@]}"; do
  target_iso=$(echo "${entry}" | jq -r '.iso')
  target_language_name=$(echo "${entry}" | jq -r '.language')
    
  if [ "$target_language_name" != "$target_language" ] && [ "$target_all" = false ]; then
      continue
  fi

    target_languages="es de en fr ru zh"
    if [[ ! "$target_languages" == *"$target_iso"* ]]; then
      echo "Unsupported target language: $target_iso"
      continue
    fi

  export target_iso="$target_iso"
  export target_language="$target_language_name"

  for entry in "${entries[@]}"; do
    iso=$(echo "${entry}" | jq -r '.iso')
    language=$(echo "${entry}" | jq -r '.language')
    flag=$(echo "${entry}" | jq -r '.flag')
    
    if [ "$language" != "$source_language" ] && [ "$source_all" = false ]; then
      continue
    fi

    export source_language="$language"
    export source_iso="$iso"

    echo "------------------------------- $source_language -> $target_language -------------------------------"

    # Step 3: Download JSON data if it doesn't exist
    if [ "$target_language" = "English" ]; then
      language_no_special_chars=$(echo "$language" | tr -d '[:space:]-') #Serbo-Croatian, Ancient Greek and such cases
      filename="kaikki.org-dictionary-$language_no_special_chars.json"
      filepath="data/kaikki/$filename"
      

      if [ ! -f "$filepath" ] || [ "$redownload" = true ]; then
        url="https://kaikki.org/dictionary/$language/$filename"
        echo "Downloading $filename from $url"
        wget "$url" -O "$filepath"
      else
        echo "Kaikki dict already exists. Skipping download."
      fi
    else
      filename="$target_iso-extract.json"
      filepath="data/kaikki/$filename"

      if [ ! -f "$filepath" ] || [ "$redownload" = true ]; then
        url="https://kaikki.org/dictionary/downloads/$target_iso/$filename.gz"
        echo "Downloading $filename from $url"
        wget "$url" -O "$filepath".gz
        echo "Extracting $filename"
        gunzip "$filepath".gz  # Use 'gunzip' to extract the compressed file
      else
        echo "Kaikki dict already exists. Skipping download."
      fi

      filename="$source_iso-$target_iso-extract.json"
      filepath="data/kaikki/$filename"

      if [ ! -f "$filepath" ]; then
        echo "Extracting $filename"
        python3 2-extract-language.py
      else
        echo "Extracted file already exists. Skipping extraction."
      fi
    fi
    export kaikki_file="$filename"


    # Step 4: Run tidy-up.js if the tidy files don't exist
    if \
      [ ! -f "data/tidy/$source_iso-$target_iso-forms.json" ] || \
      [ ! -f "data/tidy/$source_iso-$target_iso-lemmas.json" ] || \
      [ "$force_tidy" = true ]; then
      echo "Tidying up $filename"
      node --max-old-space-size=4096 2-tidy-up.js
    else
      echo "Tidy file already exists. Skipping tidying."
    fi

    # Step 5 (optional): Create an array of sentences
    if \
      [ ! -f "data/sentences/$source_iso-sentences.json" ] || \
      [ "$force_freq" = true ]; then
      if [ -d "$OPENSUBS_PATH" ]; then
        echo "Creating sentences file"
        python3 3-opensubs-to-freq.py
      else
        echo "OpenSubtitles path not found. Skipping sentence creation."
      fi
    else
      echo "Sentences file already exists. Skipping sentence creation."
    fi

    # Step 6: Create a frequency list
    if \
      [ ! -f "data/freq/$source_iso-freq.json" ] || \
      [ "$force_freq" = true ]; then
      echo "Creating frequency file"
      node 4-create-freq.js
    else
      echo "Freq file already exists. Skipping freq creation."
    fi

    dict_file="${DICT_NAME}W-$source_iso-$target_iso.zip"
    ipa_file="${DICT_NAME}W-$source_iso-$target_iso-ipa.zip"
    freq_file="${DICT_NAME}-$source_iso-freq.zip"

    # Step 7: Create Yezichak files
    if \
      [ ! -f "data/language/$source_iso/$target_iso/$dict_file" ] || \
      [ ! -f "data/language/$source_iso/$target_iso/$ipa_file" ] || \
      [ "$force_yez" = true ]; then
      echo "Creating Yezichak dict and IPA files"
      if node --max-old-space-size=8192 5-make-yezichak.js; then
        zip -j "$dict_file" data/temp/dict/index.json data/temp/dict/tag_bank_1.json data/temp/dict/term_bank_*.json
        zip -j "$ipa_file" data/temp/ipa/index.json data/temp/ipa/tag_bank_1.json data/temp/ipa/term_meta_bank_*.json
      else
        echo "Error: Yezichak generation script failed."
      fi
    else
      echo "Yezichak dict already exists. Skipping Yezichak creation."
    fi

    # Step 8: Convert frequency list to rank-based Yezichak format
    if \
      [ ! -f "data/language/$source_iso/$freq_file" ] || \
      [ "$force_yez" = true ]; then
      echo "Creating Yezichak freq files"
      if python3 6-freq-to-rank.py; then
        zip -j "$freq_file" data/temp/freq/index.json data/temp/freq/term_meta_bank_*.json
      else
        echo "Error: Frequency to rank conversion script failed."
      fi
    else
      echo "Yezichak freq already exists. Skipping Yezichak creation."
    fi

    if [ -f "$dict_file" ]; then
      mv "$dict_file" "data/language/$source_iso/$target_iso/"
    fi

    if [ -f "$ipa_file" ]; then
      mv "$ipa_file" "data/language/$source_iso/$target_iso/"
    fi

    if [ -f "$freq_file" ]; then
      mv "$freq_file" "data/language/$target_iso/"
    fi

    # Step 9: Run tests
    echo "Running tests..."
    npm run test

    echo "----------------------------------------------------------------------------------"
  done
done
echo "All done!"