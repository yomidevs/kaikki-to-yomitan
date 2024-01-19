import json
import os
import re

source_iso = os.environ.get("source_iso")
target_iso = os.environ.get("target_iso")
kaikki_file = os.environ.get("kaikki_file")

def isInflectionGloss(glosses):
    if(target_iso == 'en'):
        return re.match(r".*inflection of.*", json.dumps(glosses))
    elif(target_iso == 'fr'):
        if re.match(r"(.*)du verbe\s+((?:(?!\bdu\b).)*)$", json.dumps(glosses)):
            return True
        if re.search(r"((?:(?:Masculin|Féminin)\s)?(?:(?:p|P)luriel|(?:s|S)ingulier)) de ([^\s]+)", json.dumps(glosses)):
            return True
    return False
    
def handle_level(nest, level):
    nest_defs = []
    def_index = 0

    for definition, children in nest.items():
        def_index += 1

        if children:
            next_level = level + 1
            child_defs = handle_level(children, next_level)

            list_type = "li" if level == 1 else "number"
            content = definition if level == 1 else [{"tag": "span", "data": {"listType": "number"}, "content": f"{def_index}. "}, definition]

            nest_defs.append([{"tag": "div", "data": {"listType": list_type}, "content": content},
                              {"tag": "div", "data": {"listType": "ol"}, "style": { "marginLeft": level + 1 }, "content": child_defs}])
        else:
            nest_defs.append({"tag": "div", "data": {"listType": "li"}, "content": [{"tag": "span", "data": {"listType": "number"}, "content": f"{def_index}. "}, definition]})

    return nest_defs

def handle_nest(nested_gloss_obj, sense):
    nested_gloss = handle_level(nested_gloss_obj, 1)

    if nested_gloss:
        for entry in nested_gloss:
            sense["glosses"].append({"type": "structured-content", "content": entry})


blacklisted_tags = [
    'inflection-template',
    'table-tags',
    'nominative',
    'canonical',
    'class',
    'error-unknown-tag',
    'error-unrecognized-form',
    'infinitive',
    'includes-article',
    'obsolete',
    'archaic',
    'used-in-the-form'
]

line_count = 0
print_interval = 1000

lemma_dict = {}
form_dict = {}
form_stuff = []
automated_forms = {}

def addDeinflections(form_dict, word, pos, lemma, inflections):
    if(target_iso == 'fr'):
        word = re.sub(r"(qu\')?(ils/elles|il/elle/on)\s*", '', word)
    form_dict[word] = form_dict.get(word, {})
    form_dict[word][lemma] = form_dict[word].get(lemma, {})
    form_dict[word][lemma][pos] = form_dict[word][lemma].get(pos, [])

    form_dict[word][lemma][pos].extend(inflections)

with open(f'data/kaikki/{kaikki_file}') as file:
    for line in file:
        line_count += 1
        if line_count % print_interval == 0:
            print(f"Processed {line_count} lines...", end='\r')

        if line:
            data = json.loads(line)
            word, pos, senses, sounds, forms = data.get('word'), data.get('pos'), data.get('senses'), data.get('sounds', []), data.get('forms')

            if not (word and pos and senses):
                continue

            if forms:
                for form_data in forms:
                    form, tags = form_data.get('form'), form_data.get('tags')

                    if form and tags  and not any(value in tags for value in blacklisted_tags):
                        automated_forms[form] = automated_forms.get(form, {})
                        automated_forms[form][word] = automated_forms[form].get(word, {})
                        automated_forms[form][word][pos] = automated_forms[form][word].get(pos, [])
                        
                        automated_forms[form][word][pos].append(' '.join(tags))

            
            ipa = [{'ipa': sound['ipa'], 'tags': sound.get('tags', [])} for sound in sounds if sound and sound.get('ipa')]

            if(word == 'akull'):
                print(sounds)
                print(ipa)

            nested_gloss_obj = {}
            sense_index = 0

            for sense in senses:
                glosses = sense.get('raw_glosses') or sense.get('raw_gloss') or sense.get('glosses')
                glosses = [glosses] if isinstance(glosses, str) else glosses

                form_of = sense.get('form_of')
                tags = sense.get('tags', [])

                if glosses:
                    if form_of:
                        form_stuff.append([word, sense, pos])
                    else:
                        if not isInflectionGloss(glosses):
                            lemma_dict[word] = lemma_dict.get(word, {})
                            lemma_dict[word][pos] = lemma_dict[word].get(pos, {})
                            lemma_dict[word][pos]['ipa'] = lemma_dict[word][pos].get('ipa', [])
                            for ipa_obj in ipa:
                                if ipa_obj['ipa'] not in [obj['ipa'] for obj in lemma_dict[word][pos]['ipa']]:
                                    lemma_dict[word][pos]['ipa'].append(ipa_obj)

                            lemma_dict[word][pos]['senses'] = lemma_dict[word][pos].get('senses', [])

                            curr_sense = {'glosses': [], 'tags': tags}

                            if len(glosses) > 1:
                                nested_obj = nested_gloss_obj
                                for level in glosses:
                                    nested_obj[level] = nested_obj.get(level, {})
                                    nested_obj = nested_obj[level]

                                if sense_index == len(senses) - 1 and nested_gloss_obj:
                                    try:
                                        handle_nest(nested_gloss_obj, curr_sense)
                                    except RecursionError:
                                        print(f"Recursion error on word '{word}', pos '{pos}'")
                                        continue
                                    nested_gloss_obj = {}
                            elif len(glosses) == 1:
                                if nested_gloss_obj:
                                    handle_nest(nested_gloss_obj, curr_sense)
                                    nested_gloss_obj = {}
                                    
                                gloss = glosses[0]

                                if gloss not in json.dumps(curr_sense['glosses']):
                                    curr_sense['glosses'].append(gloss)

                            if curr_sense['glosses']:
                                lemma_dict[word][pos]['senses'].append(curr_sense)
                        else:
                            if(target_iso == 'en'):
                                lemma = re.sub(r'.+(?=inflection of)', '', sense['glosses'][0])
                                lemma = re.sub(r' \(.+?\)', '', lemma)
                                lemma = re.sub(r':$', '', lemma)
                                lemma = re.sub(r':\n.+', '', lemma)
                                lemma = re.sub(r'inflection of ', '', lemma)
                                lemma = re.sub(r':.+', '', lemma)
                                lemma = lemma.strip()
                                
                                inflection = sense['glosses'][1] if len(sense['glosses']) > 1 else ''

                                if inflection and 'inflection of ' not in inflection and word != lemma:
                                    addDeinflections(form_dict, word, pos, lemma, [inflection])

                            elif(target_iso == 'fr'):
                                inflection, lemma = None, None
                                
                                if regexMatch := re.match(r"(.*)du verbe\s+((?:(?!\bdu\b).)*)$", sense['glosses'][0]):
                                    inflection, lemma = regexMatch.group(1), regexMatch.group(2)
                                              
                                elif regexMatch := re.match(r"^((?:(?:Masculin|Féminin)\s)?(?:(?:p|P)luriel|(?:s|S)ingulier)) de ([^\s]*)$", sense['glosses'][0].strip()):
                                    inflection, lemma = regexMatch.group(1), regexMatch.group(2)

                                if inflection and lemma:
                                    inflection = inflection.strip()
                                    lemma = re.sub(r'\.$', '', lemma).strip()

                                    if inflection and word != lemma:
                                        addDeinflections(form_dict, word, pos, lemma, [inflection])
                sense_index += 1
print(f"Processed {line_count} lines...")

for form, info, pos in form_stuff:
    glosses = info['glosses']
    form_of = info['form_of']
    lemma = form_of[0]['word']

    if form != lemma and glosses:
        if not "##" in glosses[0]:
            addDeinflections(form_dict, form, pos, lemma, [glosses[0]])
        elif len(glosses) > 1:
            addDeinflections(form_dict, form, pos, lemma, [glosses[1]])

missing_forms = 0

for form, info in automated_forms.items():
    if form not in form_dict:
        missing_forms += 1

        if len(info) < 5:
            for lemma, parts in info.items():
                for pos, glosses in parts.items():
                    if form != lemma:
                        inflections = [f"-automated- {gloss}" for gloss in glosses]
                        addDeinflections(form_dict, form, pos, lemma, inflections)

print(f"There were {missing_forms} missing forms that have now been automatically populated.")

print(f"Writing lemma dict to data/tidy/{source_iso}-{target_iso}-lemmas.json...")
with open(f"data/tidy/{source_iso}-{target_iso}-lemmas.json", "w") as f:
    json.dump(lemma_dict, f)

print(f"Writing form dict to data/tidy/{source_iso}-{target_iso}-forms.json...")
with open(f"data/tidy/{source_iso}-{target_iso}-forms.json", "w") as f:
    json.dump(form_dict, f)

print('2-tidy-up.py finished.')