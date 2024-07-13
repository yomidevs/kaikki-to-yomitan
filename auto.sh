#!/bin/bash

convertMainDict(){
  export target_iso="$edition_iso"
  export target_language="$edition_name"
  export source_language="$language"
  export source_iso="$language_iso"

  echo "------------------------------- $source_language -> $target_language -------------------------------"

  # Step 4: Run tidy-up.js if the tidy files don't exist
  if \
    [ ! -f "data/tidy/$source_iso-$target_iso-forms-0.json" ] || \
    [ ! -f "data/tidy/$source_iso-$target_iso-lemmas.json" ] || \
    [ "$force_tidy" = true ]; then
      node --max-old-space-size="$max_memory_mb" 3-tidy-up.js
    else
      echo "Tidy file already exists. Skipping tidying."
  fi

  export temp_folder="data/temp"
  temp_dict_folder="$temp_folder/dict"
  temp_ipa_folder="$temp_folder/ipa"
  dict_file="${DICT_NAME}-$source_iso-$target_iso.zip"
  ipa_file="${DICT_NAME}-$source_iso-$target_iso-ipa.zip"

  # Step 5: Create Yomitan files
  if \
    [ ! -f "data/language/$source_iso/$target_iso/$dict_file" ] || \
    [ ! -f "data/language/$source_iso/$target_iso/$ipa_file" ] || \
    [ "$force_ymt" = true ]; then
    echo "Creating Yomitan dict and IPA files"
    if node --max-old-space-size="$max_memory_mb" 4-make-yomitan.js; then
      echo "Zipping Yomitan files"
      zip -qj "$dict_file" $temp_dict_folder/index.json $temp_dict_folder/styles.css $temp_dict_folder/tag_bank_1.json $temp_dict_folder/term_bank_*.json 
      zip -qj "$ipa_file" $temp_ipa_folder/index.json $temp_ipa_folder/tag_bank_1.json $temp_ipa_folder/term_meta_bank_*.json
    else
      echo "Error: Yomitan generation script failed."
    fi
  else
    echo "Yomitan dict already exists. Skipping Yomitan creation."
  fi

  if [ -f "$dict_file" ]; then
    mv "$dict_file" "data/language/$source_iso/$target_iso/"
  fi

  if [ -f "$ipa_file" ]; then
    mv "$ipa_file" "data/language/$source_iso/$target_iso/"
  fi

  echo "----------------------------------------------------------------------------------"
  return 0
}

convertGlossary(){
  export source_iso="$edition_iso"
  export source_language="$edition_name"
  export target_language="$gloss_lang_name"
  export target_iso="$gloss_iso"
  export temp_folder="data/temp"

  dict_file="${DICT_NAME}-$source_iso-$target_iso-gloss.zip"

  # Step 4: Create Yomitan files
  echo "Creating Yomitan dict files"
  if node --max-old-space-size="$max_memory_mb" make-glossary.js; then
    echo "Zipping Yomitan files"
    zip -qj "$dict_file" $temp_folder/dict/index.json $temp_folder/dict/tag_bank_1.json $temp_folder/dict/term_bank_*.json
  else
    echo "Error: Yomitan generation script failed."
  fi

  output_folder="data/language/$source_iso/$target_iso"
  if [ ! -d "$output_folder" ]; then
    mkdir -p "$output_folder"
  fi

  if [ -f "$dict_file" ]; then
    mv "$dict_file" "$output_folder"
  fi
}

source .env
export DEBUG_WORD
export DICT_NAME
max_memory_mb=${MAX_MEMORY_MB:-8192}


# Check for the source_language and target_language arguments
if [ -z "$1" ] || [ -z "$2" ] || [ -z "$3" ]; then
  echo "Usage: $0 <edition> <source_language> <target_language> [flags]"
  exit 1
fi

# Parse flags
redownload=false
force_tidy=false
force_ymt=false
force=false
keep_files=false
glossary_only=false

flags=('d' 't' 'y' 'F' 'k' 'g')
for flag in "${flags[@]}"; do
  case "$4" in 
    *"$flag"*) 
      case "$flag" in
        'd') redownload=true ;;
        't') force_tidy=true ;;
        'y') force_ymt=true ;;
        'F') force=true ;;
        'k') keep_files=true ;;
        'g') glossary_only=true ;;
      esac
      ;;
  esac
done

if [ "$force" = true ]; then
  force_tidy=true
  force_ymt=true
fi

if [ "$force_tidy" = true ]; then
  force_ymt=true
fi

echo "[d] redownload: $redownload"
echo "[F] force: $force"
echo "[t] force_tidy: $force_tidy"
echo "[y] force_ymt: $force_ymt"
echo "[k] keep_files: $keep_files"
echo "[g] glossary_only: $glossary_only"

# Step 1: Install dependencies
npm i

# Step 2: Run create-folder.js
node 1-create-folders.js

requested_edition="$1"
requested_source="$2"
requested_target="$3"

declare -a languages="($(
  jq -r '.[] | @json | @sh' languages.json
))"

supported_editions="de en es fr ru zh"

#Iterate over every edition language
for edition_lang in "${languages[@]}"; do
  export edition_iso=$(echo "${edition_lang}" | jq -r '.iso')
  edition_name=$(echo "${edition_lang}" | jq -r '.language')

  if [[ ! "$supported_editions" == *"$edition_iso"* ]]; then
    continue
  fi

  if [ "$edition_name" != "$requested_edition" ] && [ "$requested_edition" != "?" ]; then
    continue
  fi

  downloaded_edition_extract=false

  #Iterate over every language
  for some_lang in "${languages[@]}"; do
    export language_iso=$(echo "${some_lang}" | jq -r '.iso')
    language=$(echo "${some_lang}" | jq -r '.language')
    
    convert_main=true
    convert_glossary=false

    if [ "$edition_name" != "$requested_target" ] && [ "$requested_target" != "?" ]; then
      convert_main=false
    fi
    
    if [ "$language" != "$requested_source" ] && [ "$requested_source" != "?" ]; then
      convert_main=false
    fi

    if [ "$edition_name" = "$language" ]; then
      convert_glossary=true
    fi

    if [ "$edition_name" != "$requested_source" ] && [ "$requested_source" != "?" ]; then
      convert_glossary=false
    fi

    if [ "$glossary_only" = true ]; then
      convert_main=false
    fi

    if [ "$convert_main" = false ] && [ "$convert_glossary" = false ]; then
      continue
    fi

    download_language="$language"
    download_iso="$language_iso"

    export download_language
    export download_iso

    # Step 3: Download JSON data if it doesn't exist
    if [ "$edition_name" = "English" ]; then
      language_no_special_chars=$(echo "$download_language" | tr -d '[:space:]-') #Serbo-Croatian, Ancient Greek and such cases
      filename="kaikki.org-dictionary-$language_no_special_chars.jsonl"
      filepath="data/kaikki/$filename"


      if [ ! -f "$filepath" ] || [ "$redownload" = true ]; then
        url="https://kaikki.org/dictionary/$download_language/$filename"
        echo "Downloading $filename from $url"
        wget -nv "$url" -O "$filepath" 
      else
        echo "Kaikki dict already exists. Skipping download."
      fi
    else
      edition_extract="$edition_iso-extract.jsonl"
      edition_extract_path="data/kaikki/$edition_extract"

      if [ ! -f "$edition_extract_path" ] || [ "$redownload" = true ] && [ "$downloaded_edition_extract" = false ]; then
        url="https://kaikki.org/dictionary/downloads/$edition_iso/$edition_extract.gz"
        echo "Downloading $edition_extract from $url"
        wget -nv "$url" -O "$edition_extract_path".gz 
        echo "Extracting $edition_extract"
        gunzip -f "$edition_extract_path".gz
        downloaded_edition_extract=true
      else
        echo "Kaikki dict already exists. Skipping download."
      fi

      filename="$download_iso-$edition_iso-extract.jsonl"
      filepath="data/kaikki/$filename"

      if [ ! -f "$filepath" ]; then
        echo "Extracting $filename"
        python3 2-extract-language.py
      else
        echo "Extracted file already exists. Skipping extraction."
      fi
    fi

    export kaikki_file="data/kaikki/$filename"

    if [ "$convert_main" = true ]; then
      export tidy_folder="data/tidy"
      convertMainDict
    fi

    if [ "$convert_glossary" = true ]; then
      for gloss_lang in "${languages[@]}"; do
        
        export gloss_iso=$(echo "${gloss_lang}" | jq -r '.iso')
        gloss_lang_name=$(echo "${gloss_lang}" | jq -r '.language')

        if [ "$gloss_lang" = "$edition_name" ]; then
          continue
        fi

        if [ "$gloss_lang_name" != "$requested_target" ] && [ "$requested_target" != "?" ]; then
          continue
        fi

        convertGlossary
      done
    fi

    if [ "$keep_files" = false ]; then
      rm -rf "$kaikki_file"
    fi
  done

  if [ "$keep_files" = false ]; then
    edition_extract="$edition_iso-extract.jsonl"
    edition_extract_path="data/kaikki/$edition_extract"
    if [ -f "$edition_extract_path" ]; then
      rm -rf "$edition_extract_path"
    fi
  fi
done
echo "All done!"